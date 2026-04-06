use std::fmt::Display;

use abacat_common::{
    checked::{CheckedAnd, CheckedEq, CheckedOr},
    error::BasicFailure,
};
use abacat_parser::parser::parser::{BinaryOp, UnaryOp};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::{
    error::EvalError,
    value::{
        ActualType, Function,
        typed_value::{Number, NumberType, TypedValue},
    },
};

macro_rules! binary_maths {
    ($self:ident, $maths_func:ident, $right:ident, $op:expr) => {{
        let left_num = $self.as_number()?;
        let right_num = $right.as_number()?;

        if left_num.is_integer() && right_num.is_integer() {
            let left_num = left_num.as_integer(0..0)?;
            let right_num = right_num.as_integer(0..0)?;

            Ok(Value {
                val: TypedValue::Number(Number::Integer(left_num.$maths_func(right_num).ok_or(
                    EvalError::BinaryOperationFailure {
                        span: 0..0,
                        op: $op,
                    },
                )?)),
                display_hint: $self.display_hint,
            })
        } else {
            let left_num = left_num.as_decimal(0..0)?;
            let right_num = right_num.as_decimal(0..0)?;
            Ok(Value {
                val: TypedValue::Number(Number::Decimal(left_num.$maths_func(right_num).ok_or(
                    EvalError::BinaryOperationFailure {
                        span: 0..0,
                        op: $op,
                    },
                )?)),
                display_hint: DisplayHint::Auto,
            })
        }
    }};
}

#[derive(Debug, Clone)]
pub struct Value {
    pub val: TypedValue,
    pub display_hint: DisplayHint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayHint {
    Auto,
    Base16,
    Base10,
    Base8,
    Base2,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.val, &self.display_hint) {
            (TypedValue::Number(Number::Decimal(number)), DisplayHint::Base16)
                if number.is_integer() && number.to_i128().is_some() =>
            {
                write!(f, "{:#X}", number.to_i128().unwrap())
            }
            (TypedValue::Number(Number::Integer(number)), DisplayHint::Base16) => {
                write!(f, "{:#X}", number)
            }
            (TypedValue::Number(Number::Decimal(number)), DisplayHint::Base8)
                if number.is_integer() && number.to_i128().is_some() =>
            {
                write!(f, "{:#X}", number.to_i128().unwrap())
            }
            (TypedValue::Number(Number::Integer(number)), DisplayHint::Base8) => {
                write!(f, "0{:#o}", number)
            }
            (TypedValue::Number(Number::Decimal(number)), DisplayHint::Base2)
                if number.is_integer() && number.to_i128().is_some() =>
            {
                write!(f, "{:#X}", number.to_i128().unwrap())
            }
            (TypedValue::Number(Number::Integer(number)), DisplayHint::Base2) => {
                write!(f, "{:#b}", number)
            }
            (TypedValue::Number(Number::Integer(number)), _) => write!(f, "{}", number),
            (TypedValue::Number(Number::Decimal(number)), _) => write!(f, "{}", number),
            (TypedValue::Boolean(bool), _) => write!(f, "{}", bool),
            (TypedValue::Function(Function::Native(_)), _) => write!(f, "Native Function"),
            (
                TypedValue::Function(Function::UserFunction {
                    captures: _,
                    expr: _,
                }),
                _,
            ) => write!(f, "User Function"),
        }
    }
}

impl CheckedEq<&Value> for Value {
    fn checked_eq(&self, right: &Value) -> Option<bool> {
        let hint_equal = self.display_hint == right.display_hint;
        let val_equal = self.val.checked_eq(&right.val);
        val_equal.map(|val_equal| val_equal && hint_equal)
    }
}

impl Value {
    pub fn with_display_hint(&self, display_hint: DisplayHint) -> Value {
        Value::new(self.val.clone(), display_hint)
    }
    pub const fn decimal(val: Decimal) -> Value {
        Value::new(TypedValue::Number(Number::Decimal(val)), DisplayHint::Auto)
    }
    pub const fn function(val: Function) -> Value {
        Value::new(TypedValue::Function(val), DisplayHint::Auto)
    }
    pub const fn boolean(val: bool) -> Value {
        Value::new(TypedValue::Boolean(val), DisplayHint::Auto)
    }
    pub const fn integer(val: i128, display_hint: DisplayHint) -> Value {
        Value::new(TypedValue::Number(Number::Integer(val)), display_hint)
    }
    pub const fn new(val: TypedValue, display_hint: DisplayHint) -> Value {
        return Value {
            val: val,
            display_hint: display_hint,
        };
    }

    pub fn as_number(&self) -> Result<&Number, EvalError> {
        match &self.val {
            TypedValue::Number(num) => Ok(num),
            _ => Err(EvalError::UnableToConvert {
                span: 0..0,
                from: self.val.actual_type(),
                to: ActualType::Number,
            }),
        }
    }

    pub fn as_function(&self) -> Result<&Function, EvalError> {
        match &self.val {
            TypedValue::Function(function) => Ok(function),
            _ => Err(EvalError::UnableToConvert {
                span: 0..0,
                from: self.val.actual_type(),
                to: ActualType::Function,
            }),
        }
    }

    pub fn try_add(&self, right: &Value) -> Result<Value, EvalError> {
        binary_maths!(self, checked_add, right, BinaryOp::Plus)
    }
    pub fn try_sub(&self, right: &Value) -> Result<Value, EvalError> {
        binary_maths!(self, checked_sub, right, BinaryOp::Minus)
    }
    pub fn try_mul(&self, right: &Value) -> Result<Value, EvalError> {
        binary_maths!(self, checked_mul, right, BinaryOp::Multiply)
    }
    pub fn try_div(&self, right: &Value) -> Result<Value, EvalError> {
        let left_num = self.as_number()?;
        let right_num = right.as_number()?;

        if left_num.is_integer()
            && right_num.is_integer()
            && (left_num.as_integer(0..0)? % right_num.as_integer(0..0)? == 0)
        {
            let left_num = left_num.as_integer(0..0)?;
            let right_num = right_num.as_integer(0..0)?;
            Ok(Value::integer(
                left_num
                    .checked_div(right_num)
                    .ok_or(EvalError::BinaryOperationFailure {
                        span: 0..0,
                        op: BinaryOp::Divide,
                    })?,
                self.display_hint,
            ))
        } else {
            let left_num = left_num.as_decimal(0..0)?;
            let right_num = right_num.as_decimal(0..0)?;
            let result =
                left_num
                    .checked_div(right_num)
                    .ok_or(EvalError::BinaryOperationFailure {
                        span: 0..0,
                        op: BinaryOp::Divide,
                    })?;
            if result.is_integer()
                && let Some(integer) = result.to_i128()
            {
                Ok(Value::integer(integer, self.display_hint))
            } else {
                Ok(Value::decimal(result))
            }
        }
    }

    // TODO: add comparison failure eval error
    pub fn try_equal(&self, right: &Value) -> Result<Value, EvalError> {
        self.val.checked_eq(&right.val).map(Value::boolean).ok_or(
            EvalError::BinaryOperationFailure {
                span: 0..0,
                op: BinaryOp::EqualEqual,
            },
        )
    }

    pub fn try_and(&self, right: &Value) -> Result<Value, EvalError> {
        self.val.checked_and(&right.val).map(Value::boolean).ok_or(
            EvalError::BinaryOperationFailure {
                span: 0..0,
                op: BinaryOp::EqualEqual,
            },
        )
    }
    pub fn try_or(&self, right: &Value) -> Result<Value, EvalError> {
        self.val.checked_or(&right.val).map(Value::boolean).ok_or(
            EvalError::BinaryOperationFailure {
                span: 0..0,
                op: BinaryOp::OrOr,
            },
        )
    }

    pub fn try_int_div(&self, right: &Value) -> Result<Value, EvalError> {
        let left_num = self.as_number()?;
        let right_num = right.as_number()?;
        if left_num.is_integer() && right_num.is_integer() {
            let left_num = left_num.as_integer(0..0)?;
            let right_num = right_num.as_integer(0..0)?;
            Ok(Value::integer(
                left_num
                    .checked_div(right_num)
                    .ok_or(EvalError::BinaryOperationFailure {
                        span: 0..0,
                        op: BinaryOp::Divide,
                    })?,
                self.display_hint,
            ))
        } else {
            let left_num = left_num.as_decimal(0..0)?;
            let right_num = right_num.as_decimal(0..0)?;
            let result = left_num
                .checked_div(right_num)
                .ok_or(EvalError::BinaryOperationFailure {
                    span: 0..0,
                    op: BinaryOp::Divide,
                })?
                .trunc()
                .to_i128()
                .ok_or(EvalError::NumberConversionFailure {
                    span: 0..0,
                    from: NumberType::Decimal,
                    to: NumberType::Integer,
                })?;
            Ok(Value::integer(result, self.display_hint))
        }
    }

    pub fn try_exclaim(&self) -> Result<Value, EvalError> {
        match &self.val {
            TypedValue::Boolean(bool) => Ok(Value::boolean(!*bool)),
            _ => Err(EvalError::UnaryOperationFailure {
                span: 0..0,
                op: UnaryOp::ExclamationMark,
            }),
        }
    }
    pub fn try_negate(&self) -> Result<Value, EvalError> {
        match &self.val {
            TypedValue::Number(number) => match number {
                Number::Integer(int) => Ok(Value::new(
                    TypedValue::Number(Number::Integer(-int)),
                    self.display_hint,
                )),
                Number::Decimal(decimal) => Ok(Value::new(
                    TypedValue::Number(Number::Decimal(-decimal)),
                    self.display_hint,
                )),
            },
            _ => Err(EvalError::UnaryOperationFailure {
                span: 0..0,
                op: UnaryOp::Minus,
            }),
        }
    }
}
