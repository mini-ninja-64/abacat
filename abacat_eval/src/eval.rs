use std::ops::Deref;

use abacat_common::mutability::MutabilityGuard;
use abacat_parser::{
    Spanned,
    parser::parser::{BinaryOp, Expr, Function as FunctionExpr, Literal, UnaryOp},
};

use crate::{
    state::{NativeChangeset, StateMutation, StateSnapshot, VecChangeset},
    value::{DisplayHint, Function, Number, TypedValue, Value, function::Args},
};
use abacat_common::checked::CheckedEq;
use abacat_parser::parser::parser::Ident;

#[derive(Debug)]
pub enum Eval {
    Value(Value),
    ValueAssignment(Ident, Value),
}

impl<'a> CheckedEq<&Eval> for Eval {
    fn checked_eq(&self, right: &Eval) -> Option<bool> {
        match (self, right) {
            (Self::Value(left_value), Self::Value(right_value)) => {
                left_value.checked_eq(right_value)
            }
            (
                Self::ValueAssignment(left_ident, left_value),
                Self::ValueAssignment(right_ident, right_value),
            ) => {
                if left_ident != right_ident {
                    Some(false)
                } else {
                    left_value.checked_eq(right_value)
                }
            }
            _ => None,
        }
    }
}

// TODO: Make eval funcs more generic
pub type Snapshot<'a> = StateSnapshot<
    'a,
    String,
    Vec<StateMutation<String>>,
    NativeChangeset,
    Vec<StateMutation<String>>,
>;

pub fn eval_value((expr, _): &Spanned<Expr>, snapshot: &Snapshot) -> Result<Value, ()> {
    match &expr {
        Expr::Binary(left, op, right) => {
            let right = eval_value(&*right, snapshot)?;
            let left = eval_value(&*left, snapshot)?;

            match op {
                BinaryOp::Equal => Err(()),
                BinaryOp::Plus => left.try_add(right),
                BinaryOp::Minus => left.try_sub(right),
                BinaryOp::Multiply => left.try_mul(right),
                BinaryOp::Divide => left.try_div(right),
                BinaryOp::IntDivide => left.try_int_div(right),
                BinaryOp::EqualEqual => left.try_equal(&right),
                BinaryOp::AndAnd => left.try_and(&right),
                BinaryOp::OrOr => left.try_or(&right),
            }
        }
        Expr::Unary(unary_op, expr) => match unary_op {
            UnaryOp::ExclamationMark => eval_value(&*expr, snapshot)?.try_exclaim(),
            UnaryOp::Minus => eval_value(&*expr, snapshot)?.try_negate(),
        },
        Expr::Call((ident, _), args) => {
            let arg_values = args
                .into_iter()
                .map(|arg| eval_value(arg, snapshot))
                .collect::<Result<Vec<_>, _>>()?;

            if let Some((associated_snap, native_func)) = snapshot.resolve_ident_and_source(ident)
                && let TypedValue::Function(func) = native_func.consume().val
            {
                match func {
                    Function::Native(func) => func(arg_values),
                    Function::UserFunction(FunctionExpr { args, body }) => {
                        Args::exactly(args.len(), &arg_values)?;
                        let overrides = args
                            .into_iter()
                            .zip(arg_values)
                            .map(|((arg_ident, _), arg_value)| {
                                StateMutation::new(arg_ident, MutabilityGuard::Mutable(arg_value))
                            })
                            .collect::<VecChangeset<_>>();

                        return eval_value(&body, &associated_snap.with(&overrides));
                    }
                }
            } else {
                Err(())
            }
        }
        Expr::Ident((ident, _)) => snapshot
            .resolve_ident(ident)
            .map(|guard| guard.consume())
            .ok_or(()),
        Expr::Literal(literal) => Ok(match literal {
            Literal::Base10Decimal(decimal) => Value::new(
                TypedValue::Number(Number::Decimal(*decimal)),
                DisplayHint::Auto,
            ),
            Literal::Bool(boolean) => Value::new(TypedValue::Boolean(*boolean), DisplayHint::Auto),
            Literal::Base16Num(num) => Value::new(
                TypedValue::Number(Number::Integer((*num).into())),
                DisplayHint::Base16,
            ),
            Literal::Base10Num(num) => Value::new(
                TypedValue::Number(Number::Integer((*num).into())),
                DisplayHint::Base10,
            ),
            Literal::Base8Num(num) => Value::new(
                TypedValue::Number(Number::Integer((*num).into())),
                DisplayHint::Base8,
            ),
            Literal::Base2Num(num) => Value::new(
                TypedValue::Number(Number::Integer((*num).into())),
                DisplayHint::Base2,
            ),
        }),
        Expr::Parenthesised(expr) => eval_value(&*expr, snapshot),
        Expr::NamedFunction(_, _) => Err(()),
        Expr::AnonymousFunction((func, _)) => {
            Ok(Value::function(Function::UserFunction(func.clone())))
        }
    }
}

pub fn eval(expr: &Spanned<Expr>, snapshot: &Snapshot) -> Result<Eval, ()> {
    match &expr.0 {
        Expr::Binary(left, BinaryOp::Equal, right) => {
            //    Expr::Binary(, Bina)
            let (left, _) = left.deref();
            return match left {
                Expr::Ident((ident, _)) => Ok(Eval::ValueAssignment(
                    ident.to_string(),
                    eval_value(right, snapshot)?,
                )),
                _ => Err(()), // Only idents can be assigned currently
            };
            // let right = self.eval(&*right)?;
            // return;
        }
        Expr::NamedFunction((name, _), (func, _)) => {
            return Ok(Eval::ValueAssignment(
                name.to_string(),
                Value::function(Function::UserFunction(func.clone())),
            ));
            //
        }
        _ => {}
    }

    eval_value(expr, snapshot).map(Eval::Value)
}
