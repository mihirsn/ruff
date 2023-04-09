//! Struct used to efficiently slice source code at (row, column) Locations.

use ruff_text_size::{TextLen, TextRange, TextSize};
use std::ops::Add;

pub struct Locator<'a> {
    contents: &'a str,
}

impl<'a> Locator<'a> {
    pub const fn new(contents: &'a str) -> Self {
        Self { contents }
    }

    /// Computes the start position of the line of `offset`.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use ruff_text_size::TextSize;
    /// # use ruff_python_ast::source_code::Locator;
    ///
    /// let locator = Locator::new("First line\nsecond line\rthird line");
    ///
    /// assert_eq!(locator.line_start(TextSize::from(0)), TextSize::from(0));
    /// assert_eq!(locator.line_start(TextSize::from(4)), TextSize::from(0));
    ///
    /// assert_eq!(locator.line_start(TextSize::from(14)), TextSize::from(11));
    /// assert_eq!(locator.line_start(TextSize::from(28)), TextSize::from(23));
    /// ```
    ///
    /// ## Panics
    /// If `offset` is out of bounds.
    pub fn line_start(&self, offset: TextSize) -> TextSize {
        if let Some(index) = self.contents[TextRange::up_to(offset)].rfind(['\n', '\r']) {
            // SAFETY: Safe because `index < offset`
            TextSize::try_from(index).unwrap().add(TextSize::from(1))
        } else {
            TextSize::default()
        }
    }

    /// Computes the offset that is right after the newline character that ends `offset`'s line.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use ruff_text_size::{TextRange, TextSize};
    /// # use ruff_python_ast::source_code::Locator;
    ///
    /// let locator = Locator::new("First line\nsecond line\r\nthird line");
    ///
    /// assert_eq!(locator.line_end(TextSize::from(3)), TextSize::from(11));
    /// assert_eq!(locator.line_end(TextSize::from(14)), TextSize::from(24));
    /// assert_eq!(locator.line_end(TextSize::from(28)), TextSize::from(34));
    /// ```
    ///
    /// ## Panics
    ///
    /// If `offset` is passed the end of the content.
    pub fn line_end(&self, offset: TextSize) -> TextSize {
        let slice = &self.contents[usize::from(offset)..];
        if let Some(index) = slice.find(['\n', '\r']) {
            let bytes = slice.as_bytes();

            // `\r\n`
            let relative_offset = if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                TextSize::try_from(index + 2).unwrap()
            }
            // `\r` or `\n`
            else {
                TextSize::try_from(index + 1).unwrap()
            };

            offset.add(relative_offset)
        } else {
            self.contents.text_len()
        }
    }

    /// Computes the range of this `offset`s line.
    ///
    /// The range starts at the beginning of the line and goes up to, and including, the new line character
    /// at the end of the line.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use ruff_text_size::{TextRange, TextSize};
    /// # use ruff_python_ast::source_code::Locator;
    ///
    /// let locator = Locator::new("First line\nsecond line\r\nthird line");
    ///
    /// assert_eq!(locator.line_range(TextSize::from(3)), TextRange::new(TextSize::from(0), TextSize::from(11)));
    /// assert_eq!(locator.line_range(TextSize::from(14)), TextRange::new(TextSize::from(11), TextSize::from(24)));
    /// assert_eq!(locator.line_range(TextSize::from(28)), TextRange::new(TextSize::from(24), TextSize::from(34)));
    /// ```
    ///
    /// ## Panics
    /// If `offset` is out of bounds.
    pub fn line_range(&self, offset: TextSize) -> TextRange {
        TextRange::new(self.line_start(offset), self.line_end(offset))
    }

    /// Returns the text of the `offset`'s line.
    ///
    /// The line includes the newline characters at the end of the line.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use ruff_text_size::{TextRange, TextSize};
    /// # use ruff_python_ast::source_code::Locator;
    ///
    /// let locator = Locator::new("First line\nsecond line\r\nthird line");
    ///
    /// assert_eq!(locator.line(TextSize::from(3)), "First line\n");
    /// assert_eq!(locator.line(TextSize::from(14)), "second line\r\n");
    /// assert_eq!(locator.line(TextSize::from(28)), "third line");
    /// ```
    ///
    /// ## Panics
    /// If `offset` is out of bounds.
    pub fn line(&self, offset: TextSize) -> &'a str {
        &self.contents[self.line_range(offset)]
    }

    /// Computes the range of all lines that this `range` covers.
    ///
    /// The range starts at the beginning of the line at `range.start()` and goes up to, and including, the new line character
    /// at the end of `range.ends()`'s line.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use ruff_text_size::{TextRange, TextSize};
    /// # use ruff_python_ast::source_code::Locator;
    ///
    /// let locator = Locator::new("First line\nsecond line\r\nthird line");
    ///
    /// assert_eq!(
    ///     locator.lines_range(TextRange::new(TextSize::from(3), TextSize::from(5))),
    ///     TextRange::new(TextSize::from(0), TextSize::from(11))
    /// );
    /// assert_eq!(
    ///     locator.lines_range(TextRange::new(TextSize::from(3), TextSize::from(14))),
    ///     TextRange::new(TextSize::from(0), TextSize::from(24))
    /// );
    /// ```
    ///
    /// ## Panics
    /// If the start or end of `range` is out of bounds.
    pub fn lines_range(&self, range: TextRange) -> TextRange {
        TextRange::new(self.line_start(range.start()), self.line_end(range.end()))
    }

    /// Returns true if the text of `range` contains any line break.
    ///
    /// ```
    /// # use ruff_text_size::{TextRange, TextSize};
    /// # use ruff_python_ast::source_code::Locator;
    ///
    /// let locator = Locator::new("First line\nsecond line\r\nthird line");
    ///
    /// assert!(
    ///     !locator.contains_line_break(TextRange::new(TextSize::from(3), TextSize::from(5))),
    /// );
    /// assert!(
    ///     locator.contains_line_break(TextRange::new(TextSize::from(3), TextSize::from(14))),
    /// );
    /// ```
    ///
    /// ## Panics
    /// If the `range` is out of bounds.
    pub fn contains_line_break(&self, range: TextRange) -> bool {
        let text = &self.contents[range];

        text.bytes().any(|b| matches!(b, b'\n' | b'\r'))
    }

    /// Returns the text of all lines that include `range`.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use ruff_text_size::{TextRange, TextSize};
    /// # use ruff_python_ast::source_code::Locator;
    ///
    /// let locator = Locator::new("First line\nsecond line\r\nthird line");
    ///
    /// assert_eq!(
    ///     locator.lines(TextRange::new(TextSize::from(3), TextSize::from(5))),
    ///     "First line\n"
    /// );
    /// assert_eq!(
    ///     locator.lines(TextRange::new(TextSize::from(3), TextSize::from(14))),
    ///     "First line\nsecond line\r\n"
    /// );
    /// ```
    ///
    /// ## Panics
    /// If the start or end of `range` is out of bounds.
    pub fn lines(&self, range: TextRange) -> &'a str {
        &self.contents[self.lines_range(range)]
    }

    // TODO remove
    /// Take the source code up to the given [`Location`].
    #[inline]
    pub fn up_to(&self, offset: TextSize) -> &'a str {
        &self.contents[TextRange::up_to(offset)]
    }

    /// Take the source code after the given [`Location`].
    #[inline]
    pub fn after(&self, offset: TextSize) -> &'a str {
        &self.contents[usize::from(offset)..]
    }

    /// Take the source code between the given [`Range`].
    #[inline]
    pub fn slice<R: Into<TextRange>>(&self, range: R) -> &'a str {
        &self.contents[range.into()]
    }

    /// Return the underlying source code.
    pub fn contents(&self) -> &'a str {
        self.contents
    }

    /// Return the number of bytes in the source code.
    pub const fn len(&self) -> usize {
        self.contents.len()
    }

    /// Return `true` if the source code is empty.
    pub const fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}
