use abacat_common::checked::{CheckedAnd, CheckedEq, CheckedOr};
use rust_decimal::{
    Decimal,
    prelude::{FromPrimitive, ToPrimitive},
};

use abacat_parser::parser::parser::Function as FunctionExpr;

use crate::{state::VecChangeset, value::native::NativeFunctionPointer};

#[derive(Debug, Clone)]
pub enum Number {
    Integer(i128),
    Decimal(Decimal),
}
impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Decimal(l0), Self::Decimal(r0)) => l0 == r0,
            // Note: Safe to unwrap since decimal integer always fits in i128
            (Self::Decimal(l0), Self::Integer(r0)) => {
                l0.is_integer() && l0.to_i128().unwrap() == *r0
            }
            (Self::Integer(l0), Self::Decimal(r0)) => {
                r0.is_integer() && *l0 == r0.to_i128().unwrap()
            }
        }
    }
}
impl Eq for Number {}

impl Number {
    pub fn is_integer(&self) -> bool {
        match self {
            Number::Integer(_) => true,
            Number::Decimal(decimal) => decimal.is_integer(),
        }
    }
    pub fn as_integer(&self) -> Result<i128, ()> {
        match self {
            Number::Integer(int) => Ok(*int),
            Number::Decimal(decimal) => decimal.to_i128().ok_or(()),
        }
    }
    pub fn is_decimal(&self) -> bool {
        match self {
            Number::Integer(_) => false,
            Number::Decimal(decimal) => !decimal.is_integer(),
        }
    }
    pub fn as_decimal(&self) -> Result<Decimal, ()> {
        match self {
            Number::Integer(int) => Decimal::from_i128(*int).ok_or(()),
            Number::Decimal(decimal) => Ok(*decimal),
        }
    }
}

// TODO: User function interning, instead of holding an actual expr
#[derive(Debug, Clone)]
pub enum Function {
    Native(NativeFunctionPointer),
    UserFunction {
        captures: VecChangeset<String>,
        expr: FunctionExpr,
    },
}

#[derive(Debug, Clone)]
pub enum TypedValue {
    Number(Number),
    Boolean(bool),
    Function(Function),
}

impl CheckedEq<&TypedValue> for TypedValue {
    fn checked_eq(&self, right: &TypedValue) -> Option<bool> {
        match (self, right) {
            (TypedValue::Number(left), TypedValue::Number(right)) => Some(left == right),
            (TypedValue::Boolean(left), TypedValue::Boolean(right)) => Some(left == right),
            (TypedValue::Function(_), TypedValue::Function(_)) => Some(false),
            _ => None,
        }
    }
}

impl CheckedAnd<&TypedValue> for TypedValue {
    fn checked_and(&self, right: &TypedValue) -> Option<bool> {
        if let TypedValue::Boolean(left) = self
            && let TypedValue::Boolean(right) = right
        {
            return Some(*left && *right);
        }

        None
    }
}

impl CheckedOr<&TypedValue> for TypedValue {
    fn checked_or(&self, right: &TypedValue) -> Option<bool> {
        if let TypedValue::Boolean(left) = self
            && let TypedValue::Boolean(right) = right
        {
            return Some(*left || *right);
        }

        None
    }
}
