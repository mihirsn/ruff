//! Struct used to index source code, to enable efficient lookup of tokens that
//! are omitted from the AST (e.g., commented lines).

use crate::source_code::Locator;
use ruff_text_size::{TextRange, TextSize};
use rustpython_parser::lexer::LexResult;
use rustpython_parser::Tok;

pub struct Indexer {
    commented_lines: Vec<usize>,
    // FIXME rewrite to store text ranges
    continuation_lines: Vec<usize>,
}

impl Indexer {
    pub fn from_tokens(tokens: &[LexResult], locator: &Locator) -> Indexer {
        let mut commented_lines = Vec::new();
        let mut continuation_lines = Vec::new();
        // Line, Token, end
        let mut prev: Option<(usize, &Tok, TextSize)> = None;
        let mut line = 1usize;

        for (start, tok, end) in tokens.iter().flatten() {
            let prev_end = prev.map(|(_, _, end)| end).unwrap_or_default();

            let trivia = &locator.contents()[TextRange::new(prev_end, *start)];

            for (index, text) in trivia.match_indices(['\n', '\r']) {
                if text == "\r" && trivia.as_bytes().get(index + 1) == Some(&b'\n') {
                    continue;
                }

                line += 1;
            }

            if let Some((prev_line, prev_tok, _)) = prev {
                if !matches!(
                    prev_tok,
                    Tok::Newline | Tok::NonLogicalNewline | Tok::Comment(..)
                ) {
                    for line in prev_line..line {
                        continuation_lines.push(line)
                    }
                }
            }

            match tok {
                Tok::Comment(..) => {
                    commented_lines.push(line);
                }
                Tok::Newline | Tok::NonLogicalNewline => {
                    line += 1;
                }
                _ => {}
            }

            prev = Some((line, tok, *end));
        }
        Self {
            commented_lines,
            continuation_lines,
        }
    }

    pub fn commented_lines(&self) -> &[usize] {
        &self.commented_lines
    }

    pub fn continuation_lines(&self) -> &[usize] {
        &self.continuation_lines
    }
}

#[cfg(test)]
mod tests {
    use rustpython_parser::lexer::LexResult;
    use rustpython_parser::{lexer, Mode};

    use crate::source_code::{Indexer, Locator};

    #[test]
    fn continuation() {
        //         let contents = r#"x = 1"#;
        //         let lxr: Vec<LexResult> = lexer::lex(contents, Mode::Module).collect();
        //         let indexer = Indexer::from_tokens(&lxr);
        //         assert_eq!(indexer.continuation_lines(), &[]);
        //
        //         let contents = r#"
        // # Hello, world!
        //
        // x = 1
        //
        // y = 2
        // "#
        //         .trim();
        //
        //         let lxr: Vec<LexResult> = lexer::lex(contents, Mode::Module).collect();
        //         let indexer = Indexer::from_tokens(&lxr);
        //         assert_eq!(indexer.continuation_lines(), &[]);
        //
        let contents = r#"
        x = \
            1

        if True:
            z = \
                \
                2

        (
            "abc" # Foo
            "def" \
            "ghi"
        )
        "#
        .trim();

        let lxr: Vec<LexResult> = lexer::lex(contents, Mode::Module).collect();
        let indexer = Indexer::from_tokens(&lxr.as_slice(), &Locator::new(contents));
        assert_eq!(indexer.continuation_lines(), [1, 5, 6, 11]);

        let contents = r#"
x = 1; import sys
import os

if True:
    x = 1; import sys
    import os

if True:
    x = 1; \
        import os

x = 1; \
import os
"#
        .trim();

        let lxr: Vec<LexResult> = lexer::lex(contents, Mode::Module).collect();
        let indexer = Indexer::from_tokens(&lxr.as_slice(), &Locator::new(contents));
        assert_eq!(indexer.continuation_lines(), [9, 12]);
    }
}
