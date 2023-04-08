use ruff_text_size::{TextLen, TextRange};
use rustpython_parser::ast::Stmt;
use std::ops::Add;

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::types::Range;

#[violation]
pub struct Assert;

impl Violation for Assert {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Use of `assert` detected")
    }
}

/// S101
pub fn assert_used(stmt: &Stmt) -> Diagnostic {
    Diagnostic::new(
        Assert,
        TextRange::new(stmt.start(), stmt.start().add("assert".text_len())),
    )
}
