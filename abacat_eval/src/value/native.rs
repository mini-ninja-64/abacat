use phf::phf_map;
use rust_decimal::Decimal;

use crate::value::{DisplayHint, Value, function::Args};

pub type NativeFunctionPointer = fn(args: Vec<Value>) -> Result<Value, ()>;

pub enum NativeValue {
    Function(NativeFunctionPointer),
    Value(Value),
}

pub const NATIVE_VALUES: phf::Map<&str, NativeValue> = phf_map! {
    "testFuncTrue" => NativeValue::Function(|args| {
        Args::exactly(0, &args)?;
        Ok(Value::boolean(true))
    }),
    "testFuncFalse" => NativeValue::Function(|_| Ok(Value::boolean(false))),
    // Representaion
    "bin" => NativeValue::Function(|args| {
        Args::exactly(1, &args)?;
        Ok(args[0].with_display_hint(DisplayHint::Base2))
    }),
    "hex" => NativeValue::Function(|args| {
        Args::exactly(1, &args)?;
        Ok(args[0].with_display_hint(DisplayHint::Base16))
    }),
    "dec" => NativeValue::Function(|args| {
        Args::exactly(1, &args)?;
        Ok(args[0].with_display_hint(DisplayHint::Base10))
    }),
    "oct" => NativeValue::Function(|args| {
        Args::exactly(1, &args)?;
        Ok(args[0].with_display_hint(DisplayHint::Base8))
    }),
    // Note: Should probs make a macro for this, prevents need for below unit test
    "PI" => NativeValue::Value(Value::decimal(Decimal::from_parts(1102470953, 185874565, 1703060790, false, 28))),
    "E" => NativeValue::Value(Value::decimal(Decimal::from_parts(2239425882, 3958169141, 1473583531, false, 28))),
    "DECIMAL_MIN" => NativeValue::Value(Value::decimal(Decimal::MIN)),
    "DECIMAL_MAX" => NativeValue::Value(Value::decimal(Decimal::MAX)),
};

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[cfg(test)]
    impl NativeValue {
        pub fn as_decimal_unchecked(&self) -> Decimal {
            if let NativeValue::Value(val) = self {
                val.as_number().unwrap().as_decimal().unwrap()
            } else {
                unreachable!("")
            }
        }
    }

    #[test]
    fn native_decimals_arent_typoed() {
        assert_eq!(
            NATIVE_VALUES.get("PI").unwrap().as_decimal_unchecked(),
            Decimal::from_str("3.1415926535897932384626433833").unwrap(),
        );
        assert_eq!(
            NATIVE_VALUES.get("E").unwrap().as_decimal_unchecked(),
            Decimal::from_str("2.7182818284590452353602874714").unwrap(),
        );
    }
}
