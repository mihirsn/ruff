use rustpython_parser::ast::{Constant, Expr, ExprKind};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};

use crate::checkers::ast::Checker;

#[violation]
pub struct RaiseVanillaArgs;

impl Violation for RaiseVanillaArgs {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Avoid specifying long messages outside the exception class")
    }
}

fn any_string<F>(expr: &Expr, predicate: F) -> bool
where
    F: (Fn(&str) -> bool) + Copy,
{
    match &expr.node {
        ExprKind::JoinedStr { values } => {
            for value in values {
                if any_string(value, predicate) {
                    return true;
                }
            }
        }
        ExprKind::Constant {
            value: Constant::Str(val),
            ..
        } => {
            if predicate(val.as_str()) {
                return true;
            }
        }
        _ => {}
    }

    false
}

/// TRY003
pub fn raise_vanilla_args(checker: &mut Checker, expr: &Expr) {
    if let ExprKind::Call { args, .. } = &expr.node {
        if let Some(arg) = args.first() {
            if any_string(arg, |part| part.chars().any(char::is_whitespace)) {
                checker
                    .diagnostics
                    .push(Diagnostic::new(RaiseVanillaArgs, expr.range()));
            }
        }
    }
}
