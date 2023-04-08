use ruff_python_ast::newlines::StrExt;
use ruff_python_ast::source_code::Locator;
use ruff_text_size::TextRange;

/// Return `true` if the given string is a radix literal (e.g., `0b101`).
pub fn is_radix_literal(content: &str) -> bool {
    content.starts_with("0b")
        || content.starts_with("0o")
        || content.starts_with("0x")
        || content.starts_with("0B")
        || content.starts_with("0O")
        || content.starts_with("0X")
}

/// Find the first token in the given range that satisfies the given predicate.
pub fn find_tok(
    range: TextRange,
    locator: &Locator,
    f: impl Fn(rustpython_parser::Tok) -> bool,
) -> TextRange {
    for (start, tok, end) in rustpython_parser::lexer::lex_located(
        &locator.contents()[range],
        rustpython_parser::Mode::Module,
        range.start(),
    )
    .flatten()
    {
        if f(tok) {
            return TextRange::new(start, end);
        }
    }
    unreachable!("Failed to find token in range {:?}", range)
}

/// Expand the range of a compound statement.
///
/// `location` is the start of the compound statement (e.g., the `if` in `if x:`).
/// `end_location` is the end of the last statement in the body.
pub fn expand_indented_block(range: TextRange, locator: &Locator) -> TextRange {
    let contents = locator.contents();

    // Find the colon, which indicates the end of the header.
    let mut nesting = 0;
    let mut colon = None;
    for (start, tok, _end) in rustpython_parser::lexer::lex_located(
        &contents[range],
        rustpython_parser::Mode::Module,
        range.start(),
    )
    .flatten()
    {
        match tok {
            rustpython_parser::Tok::Colon if nesting == 0 => {
                colon = Some(start);
                break;
            }
            rustpython_parser::Tok::Lpar
            | rustpython_parser::Tok::Lsqb
            | rustpython_parser::Tok::Lbrace => nesting += 1,
            rustpython_parser::Tok::Rpar
            | rustpython_parser::Tok::Rsqb
            | rustpython_parser::Tok::Rbrace => nesting -= 1,
            _ => {}
        }
    }
    let colon_location = colon.unwrap();

    // From here, we have two options: simple statement or compound statement.
    let indent = rustpython_parser::lexer::lex_located(
        &contents[TextRange::new(colon_location, range.end())],
        rustpython_parser::Mode::Module,
        colon_location,
    )
    .flatten()
    .find_map(|(start, tok, _end)| match tok {
        rustpython_parser::Tok::Indent => Some(start),
        _ => None,
    });

    let line_end = locator.line_end(range.end());

    let Some(indent_location) = indent else {
        // Simple statement: from the colon to the end of the line.
        return TextRange::new(colon_location, line_end);
    };

    // Compound statement: from the colon to the end of the block.
    let mut offset = 0;
    for (index, line) in contents[usize::from(range.end())..]
        .universal_newlines()
        .skip(1)
        .enumerate()
    {
        if line.is_empty() {
            continue;
        }

        if line[TextRange::up_to(indent_location)]
            .chars()
            .all(char::is_whitespace)
        {
            offset = index + 1;
        } else {
            break;
        }
    }

    TextRange::new(colon_location, line_end)
}

/// Return true if the `orelse` block of an `if` statement is an `elif` statement.
pub fn is_elif(orelse: &[rustpython_parser::ast::Stmt], locator: &Locator) -> bool {
    if orelse.len() == 1 && matches!(orelse[0].node, rustpython_parser::ast::StmtKind::If { .. }) {
        let contents = locator.after(orelse[0].end());
        if contents.starts_with("elif") {
            return true;
        }
    }
    false
}
