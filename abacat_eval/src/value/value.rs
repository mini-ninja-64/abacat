use std::fmt::Display;

use abacat_common::checked::{CheckedAnd, CheckedEq, CheckedOr};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::value::{
    Function,
    typed_value::{Number, TypedValue},
};

macro_rules! binary_maths {
    ($self:ident, $maths_func:ident, $right:ident) => {{
        let left_num = $self.as_number()?;
        let right_num = $right.as_number()?;

        if left_num.is_integer() && right_num.is_integer() {
            let left_num = left_num.as_integer()?;
            let right_num = right_num.as_integer()?;

            Ok(Value {
                val: TypedValue::Number(Number::Integer(
                    left_num.$maths_func(right_num).ok_or(())?,
                )),
                display_hint: $self.display_hint,
            })
        } else {
            let left_num = left_num.as_decimal()?;
            let right_num = right_num.as_decimal()?;
            Ok(Value {
                val: TypedValue::Number(Number::Decimal(
                    left_num.$maths_func(right_num).ok_or(())?,
                )),
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
            (TypedValue::Number(Number::Decimal(number)), _) => write!(f, "{}", number),
            (TypedValue::Number(Number::Integer(number)), DisplayHint::Base16) => {
                write!(f, "{:#X}", number)
            }
            (TypedValue::Number(Number::Integer(number)), DisplayHint::Base8) => {
                write!(f, "0{:#o}", number)
            }
            (TypedValue::Number(Number::Integer(number)), DisplayHint::Base2) => {
                write!(f, "{:#b}", number)
            }
            (TypedValue::Number(Number::Integer(number)), _) => write!(f, "{}", number),
            (TypedValue::Boolean(bool), _) => write!(f, "{}", bool),
            (TypedValue::Function(Function::Native(_)), _) => write!(f, "Native Function"),
            (TypedValue::Function(Function::UserFunction(_)), _) => write!(f, "User Function"),
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

    pub fn as_number(&self) -> Result<&Number, ()> {
        match &self.val {
            TypedValue::Number(num) => Ok(num),
            _ => Err(()),
        }
    }

    pub fn try_add(&self, right: Value) -> Result<Value, ()> {
        binary_maths!(self, checked_add, right)
    }
    pub fn try_sub(&self, right: Value) -> Result<Value, ()> {
        binary_maths!(self, checked_sub, right)
    }
    pub fn try_mul(&self, right: Value) -> Result<Value, ()> {
        binary_maths!(self, checked_mul, right)
    }
    pub fn try_div(&self, right: Value) -> Result<Value, ()> {
        let left_num = self.as_number()?;
        let right_num = right.as_number()?;

        if left_num.is_integer()
            && right_num.is_integer()
            && (left_num.as_integer()? % right_num.as_integer()? == 0)
        {
            let left_num = left_num.as_integer()?;
            let right_num = right_num.as_integer()?;
            Ok(Value::integer(
                left_num.checked_div(right_num).ok_or(())?,
                self.display_hint,
            ))
        } else {
            let left_num = left_num.as_decimal()?;
            let right_num = right_num.as_decimal()?;
            let result = left_num.checked_div(right_num).ok_or(())?;
            if result.is_integer() {
                Ok(Value::integer(
                    result.to_i128().ok_or(())?,
                    self.display_hint,
                ))
            } else {
                Ok(Value::decimal(result))
            }
        }
    }

    pub fn try_equal(&self, right: &Value) -> Result<Value, ()> {
        Ok(Value::boolean(self.val.checked_eq(&right.val).ok_or(())?))
    }

    pub fn try_and(&self, right: &Value) -> Result<Value, ()> {
        Ok(Value::boolean(self.val.checked_and(&right.val).ok_or(())?))
    }
    pub fn try_or(&self, right: &Value) -> Result<Value, ()> {
        Ok(Value::boolean(self.val.checked_or(&right.val).ok_or(())?))
    }

    pub fn try_int_div(&self, right: Value) -> Result<Value, ()> {
        let left_num = self.as_number()?;
        let right_num = right.as_number()?;

        if left_num.is_integer() && right_num.is_integer() {
            let left_num = left_num.as_integer()?;
            let right_num = right_num.as_integer()?;
            Ok(Value::integer(
                left_num.checked_div(right_num).ok_or(())?,
                self.display_hint,
            ))
        } else {
            let left_num = left_num.as_decimal()?;
            let right_num = right_num.as_decimal()?;
            let result = left_num
                .checked_div(right_num)
                .ok_or(())?
                .trunc()
                .to_i128()
                .ok_or(())?;
            Ok(Value::integer(result, self.display_hint))
        }
    }

    pub fn try_exclaim(&self) -> Result<Value, ()> {
        match &self.val {
            TypedValue::Boolean(bool) => Ok(Value::boolean(!*bool)),
            _ => Err(()),
        }
    }
    pub fn try_negate(&self) -> Result<Value, ()> {
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
            _ => Err(()),
        }
    }
}
