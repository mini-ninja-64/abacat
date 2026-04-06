pub mod document;
pub mod eval;
pub mod state;
pub mod value;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use abacat_common::checked::CheckedEq;
    use abacat_parser::parse;
    use rust_decimal::Decimal;

    use crate::{
        document::{Document, ParserEvalPair},
        eval::Eval,
        value::{DisplayHint, Value},
    };

    use rstest::rstest;

    #[rstest]
    #[case("-1 + 1.5", Eval::Value(Value::decimal(Decimal::from_str("0.5").unwrap())))]
    #[case("0.5 / 5", Eval::Value(Value::decimal(Decimal::from_str("0.1").unwrap())))]
    #[case("0.5 // 5", Eval::Value(Value::integer(0, DisplayHint::Auto)))]
    #[case("0xFF / 5.1", Eval::Value(Value::integer(50, DisplayHint::Base16)))]
    #[case("!true", Eval::Value(Value::boolean(false)))]
    #[case("true || false", Eval::Value(Value::boolean(true)))]
    #[case("true && false", Eval::Value(Value::boolean(false)))]
    #[case("true == false", Eval::Value(Value::boolean(false)))]
    #[case("0.1 == 1", Eval::Value(Value::boolean(false)))]
    #[case("1 == 1", Eval::Value(Value::boolean(true)))]
    #[case("1 == 1.0000000000000", Eval::Value(Value::boolean(true)))]
    #[case("PI", Eval::Value(Value::decimal(Decimal::from_str("3.1415926535897932384626433833").unwrap())))]
    #[case(
        "E == 2.7182818284590452353602874714",
        Eval::Value(Value::boolean(true))
    )]
    fn statements_are_evaluated(#[case] statement: &str, #[case] expected: Eval) {
        use crate::document::Document;

        let mut doc = Document::new_with_default_constants();
        let result = doc.next(parse(statement)).unwrap();
        assert!(result.checked_eq(&expected).unwrap());
    }

    #[rstest]
    fn ttttt() {
        // let negate = |b: bool| -> bool { !b };
        // let a = negate.before(|()| true);
        let _ = parse("def abc(x,y,z) = x + y + z").unwrap();
        // println!("{:?}", x);

        let _ = parse("a = (x,y,z) => x + y + z").unwrap();
        // println!("{:?}", y);
    }

    // TODO: Assertions
    #[rstest]
    fn xyz() {
        let mut doc = Document::new_with_default_constants();

        let v = vec!["x = 123", "x = x + 456", "ans"];
        for expr in v {
            doc.next(parse(expr)).unwrap();
        }
        doc.replace_at(0, parse("x = 1"));

        for ParserEvalPair(expr, eval) in doc.history() {
            let expr = expr.as_ref().unwrap();
            let e = eval.as_ref().unwrap();
            match e {
                Eval::Value(value) => println!("'{:?}' = {:?}", expr, value),
                _ => println!("'{:?}'", expr),
            }
        }
    }

    // TODO: Assertions
    #[rstest]
    fn fun_test() {
        let mut doc = Document::new_with_default_constants();

        let v = vec![
            "x = 123",
            "def testy() = x",
            "testy()",
            "x = 456",
            "testy()",
            "def testy() = x",
            "testy()",
            "def negate(a) = !a",
            "negate(true)",
        ];
        for expr in v {
            let eval = doc.next(parse(expr)).unwrap();
            match eval {
                Eval::Value(value) => println!("'{}' = {:?}", expr, value),
                _ => println!("'{}'", expr),
            }
        }
    }

    #[rstest]
    fn assignments_mutate_state() {
        let mut doc = Document::new_with_default_constants();

        let result = doc.next(parse("x = 0x05")).unwrap();
        assert!(
            result
                .checked_eq(&Eval::ValueAssignment(
                    "x".to_string(),
                    Value::integer(5, DisplayHint::Base16)
                ))
                .unwrap()
        );
        let result = doc.next(parse("x + 6")).unwrap();
        assert!(
            result
                .checked_eq(&Eval::Value(Value::integer(11, DisplayHint::Base16)))
                .unwrap()
        );
    }
}
