use std::{
    ops::{Deref, Range},
    vec,
};

use abacat_common::{
    error::{Spanned, SpannedChumsky},
    mutability::MutabilityGuard,
};
use abacat_parser::parser::parser::{BinaryOp, Expr, Function as FunctionExpr, Literal, UnaryOp};

use crate::{
    document::ParserEvalPair,
    error::EvalError,
    state::{NativeChangeset, StateMutation, StateSnapshot, VecChangeset},
    value::{DisplayHint, Function, Number, TypedValue, Value, function::Args},
};
use abacat_parser::parser::parser::Ident;

#[derive(Debug, Clone)]
pub enum Eval {
    Value(Spanned<Value>),
    ValueAssignment(Spanned<Ident>, Spanned<Value>),
}

impl Eval {
    pub fn value(&self) -> &Value {
        match self {
            Eval::Value((value, _)) => value,
            Eval::ValueAssignment(_, (value, _)) => value,
        }
    }
}

pub struct IdentCollection<I> {
    idents: Vec<(I, Range<usize>)>,
}

impl<I> IdentCollection<I>
where
    I: PartialEq,
{
    pub fn new() -> IdentCollection<I> {
        IdentCollection { idents: vec![] }
    }
    pub fn insert(&mut self, ident: I, span: Range<usize>) {
        for (existing_ident, _) in &mut self.idents {
            if &ident == existing_ident {
                return;
            }
        }
        self.idents.push((ident, span));
    }

    pub fn to_vec(self) -> Vec<(I, Range<usize>)> {
        self.idents
    }
    pub fn remove(&mut self, ident: &I) {
        let mut to_remove: Option<usize> = None;
        for (index, (existing_ident, _)) in self.idents.iter().enumerate() {
            if ident == existing_ident {
                let _ = to_remove.insert(index);
                break;
            }
        }
        if let Some(to_remove) = to_remove {
            self.idents.remove(to_remove);
        }
    }
}

fn calculate_captures_func(
    func: &SpannedChumsky<FunctionExpr>,
    ident_collection: IdentCollection<Ident>,
) -> IdentCollection<Ident> {
    let FunctionExpr {
        args: (args, _),
        body,
    } = &func.0;
    let mut ident_collection = calculate_captures(&body, ident_collection);
    for (ident, _) in args {
        _ = ident_collection.remove(ident);
    }
    ident_collection
}

fn calculate_captures(
    expr: &SpannedChumsky<Expr>,
    mut ident_collection: IdentCollection<Ident>,
) -> IdentCollection<Ident> {
    let (expr, expr_span) = expr;
    match expr {
        Expr::Binary(left, _, right) => {
            let vec = calculate_captures(left, ident_collection);
            calculate_captures(right, vec)
        }
        Expr::Unary(_, right) => calculate_captures(right, ident_collection),
        Expr::Call(_, (items, _)) => items
            .iter()
            .fold(ident_collection, |vec, expr| calculate_captures(expr, vec)),
        Expr::Ident(ident) => {
            ident_collection.insert(ident.clone(), expr_span.into_range());
            ident_collection
        }
        Expr::Literal(_) => ident_collection,
        Expr::Parenthesised(expr) => calculate_captures(expr, ident_collection),
        Expr::List(elements) => elements
            .iter()
            .fold(ident_collection, |vec, expr| calculate_captures(expr, vec)),
        Expr::AnonymousFunction(func) | Expr::NamedFunction(_, func) => {
            calculate_captures_func(func, ident_collection)
        }
        Expr::IndexedExpr(expr, index) => {
            let captured = calculate_captures(expr, ident_collection);
            calculate_captures(index, captured)
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

fn exec_func(
    func: &Function,
    arg_values: Spanned<Vec<Spanned<Value>>>,
    snapshot: &Snapshot,
) -> Result<Value, EvalError> {
    match func {
        Function::Native(func) => func(&arg_values),
        Function::UserFunction {
            captures,
            expr:
                FunctionExpr {
                    args: (args, _),
                    body,
                },
        } => {
            Args::exactly(args.len(), &arg_values)?;

            // TODO: Collecting before this just to convert to a vec again here
            let arg_changeset = args
                .into_iter()
                .zip(arg_values.0)
                // TODO: Unnessecary cloning blurgh
                .map(|((arg_ident, _), (arg_value, _))| {
                    StateMutation::new(arg_ident.clone(), MutabilityGuard::Mutable(arg_value))
                })
                .chain(captures.into_iter().map(|x| x.clone()))
                .collect::<Vec<_>>();
            eval_value(&body, &snapshot, Some(&arg_changeset)).map(|val| val.0)
        }
    }
}

// TODO: Make eval funcs more generic
pub type Snapshot<'a> = StateSnapshot<'a, String, NativeChangeset, ParserEvalPair>;

// TODO: I dont think this needs to return a spanned value anymore, since Value contain a span already?
pub fn eval_value<'a, 'b: 'c, 'c>(
    (expr, expr_span): &'a SpannedChumsky<Expr>,
    snapshot: &'b Snapshot<'b>,
    overrides: Option<&VecChangeset<String>>,
) -> Result<Spanned<Value>, EvalError> {
    match &expr {
        Expr::Binary(left_expr, BinaryOp::Pipe, right_expr) => {
            let left = eval_value(&*left_expr, snapshot, overrides)?;
            if let Expr::Call(func_expr, args) = &right_expr.0 {
                let (func_val, _) = eval_value(func_expr, snapshot, overrides)?;
                let func = func_val.as_function()?;
                let arg_values = args
                    .0
                    .iter()
                    .map(|arg| eval_value(arg, snapshot, overrides));
                let args = [Ok(left)]
                    .into_iter()
                    .chain(arg_values)
                    .collect::<Result<Vec<_>, EvalError>>()?;
                exec_func(func, (args, expr_span.into_range()), snapshot)
                    .map(|val| (val, expr_span.into_range()))
            } else {
                let right = eval_value(&*right_expr, snapshot, overrides)?;
                let func = right.0.as_function()?;
                let left_range = left_expr.1.into_range();
                let args = vec![left];
                exec_func(func, (args, left_range), snapshot)
                    .map(|val| (val, expr_span.into_range()))
            }
            .map_err(|err| err.replace_span(Some(right_expr.1.into_range())))
        }
        Expr::Binary(left, op, right) => {
            let (left, _) = eval_value(&*left, snapshot, overrides)?;
            let (right, _) = eval_value(&*right, snapshot, overrides)?;

            match op {
                BinaryOp::Plus => left.try_add(&right),
                BinaryOp::Minus => left.try_sub(&right),
                BinaryOp::Multiply => left.try_mul(&right),
                BinaryOp::Divide => left.try_div(&right),
                BinaryOp::IntDivide => left.try_int_div(&right),
                BinaryOp::EqualEqual => left.try_equal(&right),
                BinaryOp::AndAnd => left.try_and(&right),
                BinaryOp::OrOr => left.try_or(&right),
                BinaryOp::Equal | BinaryOp::Pipe => Err(EvalError::ImplementationBug),
            }
            .map(|val| (val, expr_span.into_range()))
        }
        Expr::Unary(unary_op, expr) => match unary_op {
            UnaryOp::ExclamationMark => eval_value(&*expr, snapshot, overrides)?
                .0
                .try_exclaim()
                .map(|val| (val, expr_span.into_range())),
            UnaryOp::Minus => eval_value(&*expr, snapshot, overrides)?
                .0
                .try_negate()
                .map(|val| (val, expr_span.into_range())),
        },
        Expr::Call(expr, (args, args_range)) => {
            let (left, _) = eval_value(expr.as_ref(), snapshot, overrides)?;
            let func = left.as_function()?;

            let arg_values = args
                .iter()
                .map(|arg| eval_value(arg, snapshot, overrides))
                .collect::<Result<Vec<_>, _>>()?;
            exec_func(func, (arg_values, args_range.into_range()), snapshot)
                .map_err(|val| val.replace_span(Some(expr.1.into_range())))
                .map(|val| (val, expr_span.into_range()))
        }
        Expr::Ident(ident) => snapshot
            .resolve_ident(ident, overrides)
            .map(|val| (val.consume(), expr_span.into_range()))
            .ok_or_else(|| EvalError::IdentNotFound {
                span: Some(expr_span.into_range()),
                ident: ident.clone(),
            }),
        Expr::Literal(literal) => Ok(match literal {
            Literal::Base10Decimal(decimal) => (
                Value::new(
                    Some(expr_span.into_range()),
                    TypedValue::Number(Number::Decimal(*decimal)),
                    DisplayHint::Auto,
                ),
                expr_span.into_range(),
            ),
            Literal::Bool(boolean) => (
                Value::new(
                    Some(expr_span.into_range()),
                    TypedValue::Boolean(*boolean),
                    DisplayHint::Auto,
                ),
                expr_span.into_range(),
            ),
            Literal::Base16Num(num) => (
                Value::new(
                    Some(expr_span.into_range()),
                    TypedValue::Number(Number::Integer((*num).into())),
                    DisplayHint::Base16,
                ),
                expr_span.into_range(),
            ),
            Literal::Base10Num(num) => (
                Value::new(
                    Some(expr_span.into_range()),
                    TypedValue::Number(Number::Integer((*num).into())),
                    DisplayHint::Base10,
                ),
                expr_span.into_range(),
            ),
            Literal::Base8Num(num) => (
                Value::new(
                    Some(expr_span.into_range()),
                    TypedValue::Number(Number::Integer((*num).into())),
                    DisplayHint::Base8,
                ),
                expr_span.into_range(),
            ),
            Literal::Base2Num(num) => (
                Value::new(
                    Some(expr_span.into_range()),
                    TypedValue::Number(Number::Integer((*num).into())),
                    DisplayHint::Base2,
                ),
                expr_span.into_range(),
            ),
        }),
        Expr::Parenthesised(expr) => {
            eval_value(&*expr, snapshot, overrides).map(|(val, _)| (val, expr_span.into_range()))
        }
        Expr::List(exprs) => exprs
            .iter()
            .map(|expr| eval_value(expr, snapshot, overrides).map(|(value, _)| value))
            .collect::<Result<Vec<_>, _>>()
            .map(|values| {
                (
                    Value::new(
                        Some(expr_span.into_range()),
                        TypedValue::List(values),
                        DisplayHint::Auto,
                    ),
                    expr_span.into_range(),
                )
            }),
        Expr::IndexedExpr(expr, index) => {
            let (list, _) = eval_value(&*expr, snapshot, overrides)?;
            let (index, _) = eval_value(&*index, snapshot, overrides)?;
            let (index, elements) = validate_index_for_list(&index, &list)?;
            Ok((elements[index].clone(), expr_span.into_range()))
        }
        Expr::NamedFunction(_, _) => Err(EvalError::ImplementationBug),
        Expr::AnonymousFunction(func) => {
            let captures = calculate_captures_func(func, IdentCollection::new())
                .to_vec()
                .into_iter()
                .map(|(ident, ident_span)| {
                    if let Some(val) = snapshot.resolve_ident::<VecChangeset<_>>(&ident, None) {
                        Ok(StateMutation::new(ident, val))
                    } else {
                        Err(EvalError::IdentNotFound {
                            span: Some(ident_span),
                            ident,
                        })
                    }
                })
                .collect::<Result<Vec<_>, EvalError>>()?;

            Ok((
                Value::function(Function::UserFunction {
                    captures: captures,
                    expr: func.0.clone(),
                }),
                expr_span.into_range(),
            ))
        }
    }
}

pub fn validate_index_for_list<'a, 'b>(
    index: &'a Value,
    list: &'b Value,
) -> Result<(usize, &'b Vec<Value>), EvalError> {
    let elements = list.as_list()?;
    let length = elements.len() as i128;
    let index_span = &index.span;
    let mut index = index.as_number()?.as_integer(index_span)?;
    if index < 0 {
        index = length + index;
    };
    if index < 0 || index >= length {
        Err(EvalError::IndexOutOfRangeError {
            span: index_span.clone(),
            index,
            length: length as usize,
        })
    } else {
        // We can safely cast to usize as we already know it must be smaller than max of usize
        // due to index >= length check
        Ok((index as usize, elements))
    }
}

pub fn eval(expr: &SpannedChumsky<Expr>, snapshot: &Snapshot) -> Result<Eval, EvalError> {
    match &expr.0 {
        Expr::Binary(left, BinaryOp::Equal, right) => {
            let (left, left_span) = left.deref();
            return match left {
                Expr::Ident(ident) => Ok(Eval::ValueAssignment(
                    (ident.to_string(), left_span.into_range()),
                    eval_value(right, snapshot, None)?,
                )),
                _ => Err(EvalError::NonIdentAssignment {
                    span: Some(left_span.into_range()),
                }),
            };
        }
        Expr::NamedFunction((name, name_span), func) => {
            let captures = calculate_captures_func(func, IdentCollection::new())
                .to_vec()
                .into_iter()
                .map(|(ident, ident_span)| {
                    if let Some(val) = snapshot.resolve_ident::<VecChangeset<_>>(&ident, None) {
                        Ok(StateMutation::new(ident, val))
                    } else {
                        Err(EvalError::IdentNotFound {
                            span: Some(ident_span),
                            ident,
                        })
                    }
                })
                .collect::<Result<Vec<_>, EvalError>>()?;
            return Ok(Eval::ValueAssignment(
                (name.to_string(), name_span.into_range()),
                (
                    Value::function(Function::UserFunction {
                        captures,
                        expr: func.0.to_owned(),
                    }),
                    func.1.into_range(),
                ),
            ));
            //
        }
        _ => {}
    }
    eval_value(expr, snapshot, None).map(|val| Eval::Value(val))
}
