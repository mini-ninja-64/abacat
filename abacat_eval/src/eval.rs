use std::{collections::HashSet, ops::Deref};

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

#[derive(Debug, Clone)]
pub enum Eval {
    Value(Value),
    ValueAssignment(Ident, Value),
}

impl Eval {
    pub fn value(&self) -> &Value {
        match self {
            Eval::Value(value) => value,
            Eval::ValueAssignment(_, value) => value,
        }
    }
}

fn calculate_captures_func(
    func: &Spanned<FunctionExpr>,
    hash_set: HashSet<Ident>,
) -> HashSet<Ident> {
    let (FunctionExpr { args, body }, _) = &func;
    let mut hash_set = calculate_captures(&body, hash_set);
    for (ident, _) in args {
        _ = hash_set.remove(ident);
    }
    hash_set
}

fn calculate_captures(
    function_expr: &Spanned<Expr>,
    mut hash_set: HashSet<Ident>,
) -> HashSet<Ident> {
    let (body, _) = function_expr;
    match body {
        Expr::Binary(left, _, right) => {
            let vec = calculate_captures(left, hash_set);
            calculate_captures(right, vec)
        }
        Expr::Unary(_, right) => calculate_captures(right, hash_set),
        Expr::Call(_, items) => items
            .iter()
            .fold(hash_set, |vec, expr| calculate_captures(expr, vec)),
        Expr::Ident(ident) => {
            hash_set.insert(ident.0.clone());
            hash_set
        }
        Expr::Literal(_) => hash_set,
        Expr::Parenthesised(expr) => calculate_captures(expr, hash_set),
        Expr::AnonymousFunction(func) | Expr::NamedFunction(_, func) => {
            calculate_captures_func(func, hash_set)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // let expr = parse("(a) => (b) => a + b + z").unwrap();
        // let idents = calculate_captures(&expr, HashSet::new()).iter().collect();
        // assert_eq!(idents, vec!["z".to_string()])
    }
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

pub fn exec_func(
    func: &Function,
    arg_values: Vec<Value>,
    snapshot: &Snapshot,
) -> Result<Value, ()> {
    match func {
        Function::Native(func) => func(&arg_values),
        Function::UserFunction {
            captures,
            expr: FunctionExpr { args, body },
        } => {
            Args::exactly(args.len(), &arg_values)?;

            // TODO: Collecting before this just to convert to a vec again here
            let arg_changeset = args
                .into_iter()
                .zip(arg_values)
                // TODO: Unnessecary cloning blurgh
                .map(|((arg_ident, _), arg_value)| {
                    StateMutation::new(arg_ident.clone(), MutabilityGuard::Mutable(arg_value))
                })
                .chain(captures.into_iter().map(|x| x.clone()))
                .collect::<Vec<_>>();
            eval_value(&body, &snapshot, Some(&arg_changeset))
        }
    }
}

// TODO: Make eval funcs more generic
pub type Snapshot<'a> =
    StateSnapshot<'a, String, NativeChangeset, (Result<Spanned<Expr>, ()>, Result<Eval, ()>)>;

pub fn eval_value<'a, 'b: 'c, 'c>(
    (expr, _): &'a Spanned<Expr>,
    snapshot: &'b Snapshot<'b>,
    overrides: Option<&VecChangeset<String>>,
) -> Result<Value, ()> {
    match &expr {
        Expr::Binary(left, BinaryOp::Pipe, right) => {
            let left = eval_value(&*left, snapshot, overrides)?;
            if let Expr::Call(func_expr, args) = &right.0 {
                let func_val = eval_value(&*func_expr, snapshot, overrides)?;
                let func = func_val.as_function()?;
                let arg_values = args
                    .into_iter()
                    .map(|arg| eval_value(arg, snapshot, overrides));
                let args = [Ok(left)]
                    .into_iter()
                    .chain(arg_values)
                    .collect::<Result<Vec<_>, ()>>()?;
                exec_func(&*func, args, snapshot)
            } else {
                let right = eval_value(&*right, snapshot, overrides)?;
                let func = right.as_function()?;
                let args = vec![left];
                exec_func(func, args, snapshot)
            }
        }
        Expr::Binary(left, op, right) => {
            let left = eval_value(&*left, snapshot, overrides)?;
            let right = eval_value(&*right, snapshot, overrides)?;

            match op {
                BinaryOp::Equal => Err(()),
                BinaryOp::Plus => left.try_add(&right),
                BinaryOp::Minus => left.try_sub(&right),
                BinaryOp::Multiply => left.try_mul(&right),
                BinaryOp::Divide => left.try_div(&right),
                BinaryOp::IntDivide => left.try_int_div(&right),
                BinaryOp::EqualEqual => left.try_equal(&right),
                BinaryOp::AndAnd => left.try_and(&right),
                BinaryOp::OrOr => left.try_or(&right),
                BinaryOp::Pipe => Err(()), // TODO: bug in code
            }
        }
        Expr::Unary(unary_op, expr) => match unary_op {
            UnaryOp::ExclamationMark => eval_value(&*expr, snapshot, overrides)?.try_exclaim(),
            UnaryOp::Minus => eval_value(&*expr, snapshot, overrides)?.try_negate(),
        },
        Expr::Call(expr, args) => {
            let left = eval_value(expr.as_ref(), snapshot, overrides)?;
            let func = left.as_function()?;

            let arg_values: Vec<Value> = args
                .into_iter()
                .map(|arg| eval_value(arg, snapshot, overrides))
                .collect::<Result<Vec<_>, _>>()?;
            exec_func(func, arg_values, snapshot)
        }
        Expr::Ident((ident, _)) => snapshot
            .resolve_ident(ident, overrides)
            .map(|val| val.consume())
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
        Expr::Parenthesised(expr) => eval_value(&*expr, snapshot, overrides),
        Expr::NamedFunction(_, _) => Err(()),
        Expr::AnonymousFunction(func) => {
            let captures = calculate_captures_func(func, HashSet::new())
                .iter()
                .map(|ident| {
                    snapshot
                        .resolve_ident(ident, overrides)
                        .map(move |val| StateMutation::new(ident.clone(), val))
                })
                .collect::<Option<Vec<_>>>()
                .ok_or(())?;

            Ok(Value::function(Function::UserFunction {
                captures: captures,
                expr: func.0.clone(),
            }))
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
                    eval_value(right, snapshot, None)?,
                )),
                _ => Err(()), // Only idents can be assigned currently
            };
        }
        Expr::NamedFunction((name, _), func) => {
            let captures = calculate_captures_func(func, HashSet::new())
                .into_iter()
                .map(|ident| {
                    snapshot
                        .resolve_ident::<VecChangeset<_>>(&ident, None)
                        .map(move |val| StateMutation::new(ident, val))
                })
                .collect::<Option<Vec<_>>>()
                .ok_or(())?;
            return Ok(Eval::ValueAssignment(
                name.to_string(),
                Value::function(Function::UserFunction {
                    captures,
                    expr: func.0.to_owned(),
                }),
            ));
            //
        }
        _ => {}
    }
    eval_value(expr, snapshot, None).map(|val| Eval::Value(val))
}
