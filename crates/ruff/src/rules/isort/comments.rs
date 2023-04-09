use ruff_text_size::TextRange;
use std::borrow::Cow;

use rustpython_parser::ast::Location;
use rustpython_parser::{lexer, Mode, Tok};

use ruff_python_ast::source_code::Locator;

#[derive(Debug)]
pub struct Comment<'a> {
    pub value: Cow<'a, str>,
    pub location: Location,
    pub end_location: Location,
}

/// Collect all comments in an import block.
pub fn collect_comments<'a>(range: TextRange, locator: &'a Locator) -> Vec<Comment<'a>> {
    let contents = locator.slice(range);
    lexer::lex_located(contents, Mode::Module, range.start())
        .flatten()
        .filter_map(|(start, tok, end)| {
            if let Tok::Comment(value) = tok {
                Some(Comment {
                    value: value.into(),
                    location: start,
                    end_location: end,
                })
            } else {
                None
            }
        })
        .collect()
}
