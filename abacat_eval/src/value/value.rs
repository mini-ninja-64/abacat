use std::{fmt::Display, ops::Range};

use abacat_common::checked::{CheckedAnd, CheckedEq, CheckedOr};
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
        $self.binary_with($right, &|l, r| {
            let left_num = l.as_number()?;
            let right_num = r.as_number()?;

            if left_num.is_integer() && right_num.is_integer() {
                let left_num = left_num.as_integer(&l.span)?;
                let right_num = right_num.as_integer(&r.span)?;

                Ok(Value {
                    span: None,
                    val: TypedValue::Number(Number::Integer(
                        left_num.$maths_func(right_num).ok_or(
                            EvalError::BinaryOperationFailure {
                                span: combined_span(l, r),
                                op: $op,
                            },
                        )?,
                    )),
                    display_hint: l.display_hint,
                })
            } else {
                let left_num = left_num.as_decimal(&l.span)?;
                let right_num = right_num.as_decimal(&r.span)?;
                Ok(Value {
                    span: None,
                    val: TypedValue::Number(Number::Decimal(
                        left_num.$maths_func(right_num).ok_or(
                            EvalError::BinaryOperationFailure {
                                span: combined_span(l, r),
                                op: $op,
                            },
                        )?,
                    )),
                    display_hint: DisplayHint::Auto,
                })
            }
        })
    }};
}

#[derive(Debug, Clone)]
pub struct Value {
    pub span: Option<Range<usize>>,
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
            (TypedValue::List(values), _) => {
                write!(f, "[")?;
                for (count, v) in values.iter().enumerate() {
                    if count != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
        }
    }
}

pub fn combined_span(left: &Value, right: &Value) -> Option<Range<usize>> {
    left.span.as_ref().and_then(|left_span| {
        right
            .span
            .as_ref()
            .map(|right_span| left_span.start..right_span.end)
    })
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
        Value::new(None, self.val.clone(), display_hint)
    }
    pub const fn decimal(val: Decimal) -> Value {
        Value::new(
            None,
            TypedValue::Number(Number::Decimal(val)),
            DisplayHint::Auto,
        )
    }
    pub const fn list(values: Vec<Value>) -> Value {
        Value::new(None, TypedValue::List(values), DisplayHint::Auto)
    }
    pub const fn function(val: Function) -> Value {
        Value::new(None, TypedValue::Function(val), DisplayHint::Auto)
    }
    pub const fn boolean(val: bool) -> Value {
        Value::new(None, TypedValue::Boolean(val), DisplayHint::Auto)
    }
    pub const fn integer(val: i128, display_hint: DisplayHint) -> Value {
        Value::new(None, TypedValue::Number(Number::Integer(val)), display_hint)
    }
    pub const fn new(
        span: Option<Range<usize>>,
        val: TypedValue,
        display_hint: DisplayHint,
    ) -> Value {
        return Value {
            span: span,
            val: val,
            display_hint: display_hint,
        };
    }

    pub fn as_list(&self) -> Result<&Vec<Value>, EvalError> {
        match &self.val {
            TypedValue::List(values) => Ok(values),
            _ => Err(EvalError::UnableToConvert {
                span: self.span.clone(),
                from: self.val.actual_type(),
                to: ActualType::List,
            }),
        }
    }

    pub fn as_number(&self) -> Result<&Number, EvalError> {
        match &self.val {
            TypedValue::Number(num) => Ok(num),
            _ => Err(EvalError::UnableToConvert {
                span: self.span.clone(),
                from: self.val.actual_type(),
                to: ActualType::Number,
            }),
        }
    }

    pub fn as_function(&self) -> Result<&Function, EvalError> {
        match &self.val {
            TypedValue::Function(function) => Ok(function),
            _ => Err(EvalError::UnableToConvert {
                span: self.span.clone(),
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
        self.binary_with(right, &|l, r| {
            let left_num = l.as_number()?;
            let right_num = r.as_number()?;

            if let Ok(right_num) = right_num.as_integer(&r.span)
                && right_num == 0
            {
                return Err(EvalError::DivideByZero {
                    span: combined_span(l, r),
                });
            }

            if left_num.is_integer()
                && right_num.is_integer()
                && (left_num.as_integer(&l.span)? % right_num.as_integer(&r.span)? == 0)
            {
                let left_num = left_num.as_integer(&l.span)?;
                let right_num = right_num.as_integer(&r.span)?;
                Ok(Value::integer(
                    left_num
                        .checked_div(right_num)
                        .ok_or(EvalError::BinaryOperationFailure {
                            span: combined_span(l, r),
                            op: BinaryOp::Divide,
                        })?,
                    self.display_hint,
                ))
            } else {
                let left_num = left_num.as_decimal(&l.span)?;
                let right_num = right_num.as_decimal(&r.span)?;
                let result =
                    left_num
                        .checked_div(right_num)
                        .ok_or(EvalError::BinaryOperationFailure {
                            span: combined_span(l, r),
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
        })
    }
    pub fn try_int_div(&self, right: &Value) -> Result<Value, EvalError> {
        let left_num = self.as_number()?;
        let right_num = right.as_number()?;

        if let Ok(right_num) = right_num.as_integer(&right.span)
            && right_num == 0
        {
            return Err(EvalError::DivideByZero {
                span: combined_span(self, right),
            });
        }
        if left_num.is_integer() && right_num.is_integer() {
            let left_num = left_num.as_integer(&self.span)?;
            let right_num = right_num.as_integer(&right.span)?;
            Ok(Value::integer(
                left_num
                    .checked_div(right_num)
                    .ok_or(EvalError::BinaryOperationFailure {
                        span: combined_span(self, right),
                        op: BinaryOp::Divide,
                    })?,
                self.display_hint,
            ))
        } else {
            let left_num = left_num.as_decimal(&self.span)?;
            let right_num = right_num.as_decimal(&right.span)?;
            let result = left_num
                .checked_div(right_num)
                .ok_or(EvalError::BinaryOperationFailure {
                    span: combined_span(self, right),
                    op: BinaryOp::Divide,
                })?
                .trunc()
                .to_i128()
                .ok_or(EvalError::NumberConversionFailure {
                    span: combined_span(self, right),
                    from: NumberType::Decimal,
                    to: NumberType::Integer,
                })?;
            Ok(Value::integer(result, self.display_hint))
        }
    }

    // TODO: add comparison failure eval error
    pub fn try_equal(&self, right: &Value) -> Result<Value, EvalError> {
        self.binary_with(right, &|l, r| {
            l.val
                .checked_eq(&r.val)
                .map(Value::boolean)
                .ok_or(EvalError::BinaryOperationFailure {
                    span: combined_span(l, r),
                    op: BinaryOp::EqualEqual,
                })
        })
    }
    pub fn try_and(&self, right: &Value) -> Result<Value, EvalError> {
        self.binary_with(right, &|l, r| {
            l.val
                .checked_and(&r.val)
                .map(Value::boolean)
                .ok_or(EvalError::BinaryOperationFailure {
                    span: combined_span(l, r),
                    op: BinaryOp::AndAnd,
                })
        })
    }
    pub fn try_or(&self, right: &Value) -> Result<Value, EvalError> {
        self.binary_with(right, &|l, r| {
            l.val
                .checked_or(&r.val)
                .map(Value::boolean)
                .ok_or(EvalError::BinaryOperationFailure {
                    span: combined_span(l, r),
                    op: BinaryOp::OrOr,
                })
        })
    }

    pub fn try_exclaim(&self) -> Result<Value, EvalError> {
        self.map_inner(&|value| match &value.val {
            TypedValue::Boolean(bool) => Ok(Value::boolean(!*bool)),
            _ => Err(EvalError::UnaryOperationFailure {
                span: value.span.clone(),
                op: UnaryOp::ExclamationMark,
            }),
        })
    }
    pub fn try_negate(&self) -> Result<Value, EvalError> {
        self.map_inner(&|value| match &value.val {
            TypedValue::Number(number) => match number {
                Number::Integer(int) => Ok(Value::new(
                    None,
                    TypedValue::Number(Number::Integer(-int)),
                    value.display_hint,
                )),
                Number::Decimal(decimal) => Ok(Value::new(
                    None,
                    TypedValue::Number(Number::Decimal(-decimal)),
                    value.display_hint,
                )),
            },
            _ => Err(EvalError::UnaryOperationFailure {
                span: value.span.clone(),
                op: UnaryOp::Minus,
            }),
        })
    }

    fn map_inner(
        &self,
        f: &impl Fn(&Value) -> Result<Value, EvalError>,
    ) -> Result<Value, EvalError> {
        if let TypedValue::List(values) = &self.val {
            let mut mapped = Vec::with_capacity(values.len());
            for value in values {
                mapped.push(value.map_inner(f)?);
            }
            Ok(Value::list(mapped))
        } else {
            f(self)
        }
    }

    fn binary_with(
        &self,
        right: &Value,
        f: &impl Fn(&Value, &Value) -> Result<Value, EvalError>,
    ) -> Result<Value, EvalError> {
        match (&self.val, &right.val) {
            (TypedValue::List(left), TypedValue::List(right)) if left.len() == right.len() => {
                let mut mapped = Vec::with_capacity(left.len());
                for (left, right) in left.iter().zip(right) {
                    mapped.push(left.binary_with(right, f)?);
                }
                Ok(Value::list(mapped))
            }
            (TypedValue::List(_), TypedValue::List(_)) => Err(EvalError::MismatchArrayLength {
                left: self.span.clone(),
                right: right.span.clone(),
            }),
            (TypedValue::List(_), _) => self.map_inner(&|left| f(left, right)),
            (_, TypedValue::List(_)) => right.map_inner(&|right| f(self, right)),
            (_, _) => f(self, right),
        }
    }
}
