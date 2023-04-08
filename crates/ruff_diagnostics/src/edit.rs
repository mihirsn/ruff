use ruff_text_size::{TextRange, TextSize};
use rustpython_parser::ast::Location;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A text edit to be applied to a source file. Inserts, deletes, or replaces
/// content at a given location.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Edit {
    /// The start location of the edit.
    range: TextRange,
    /// The replacement content to insert between the start and end locations.
    content: Option<Box<str>>,
}

impl Edit {
    /// Creates an edit that deletes the content in the `start` to `end` range.
    pub const fn deletion(start: TextSize, end: TextSize) -> Self {
        Self {
            content: None,
            range: TextRange::new(start, end),
        }
    }

    /// Creates an edit that replaces the content in the `start` to `end` range with `content`.
    pub fn replacement(content: String, start: TextSize, end: TextSize) -> Self {
        debug_assert!(!content.is_empty(), "Prefer `Fix::deletion`");

        Self {
            content: Some(Box::from(content)),
            range: TextRange::new(start, end),
        }
    }

    /// Creates an edit that inserts `content` at the [`Location`] `at`.
    pub fn insertion(content: String, at: TextSize) -> Self {
        debug_assert!(!content.is_empty(), "Insert content is empty");

        Self {
            content: Some(Box::from(content)),
            range: TextRange::new(at, at),
        }
    }

    /// Returns the new content for an insertion or deletion.
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    /// Returns the start location of the edit in the source document.
    pub const fn start(&self) -> Location {
        self.range.start()
    }

    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the edit's end location in the source document.
    pub const fn end(&self) -> TextSize {
        self.range.end()
    }

    fn kind(&self) -> EditOperationKind {
        if self.content.is_none() {
            EditOperationKind::Deletion
        } else if self.range.is_empty() {
            EditOperationKind::Insertion
        } else {
            EditOperationKind::Replacement
        }
    }

    /// Returns `true` if this edit deletes content from the source document.
    #[inline]
    pub fn is_deletion(&self) -> bool {
        self.kind().is_deletion()
    }

    /// Returns `true` if this edit inserts new content into the source document.
    #[inline]
    pub fn is_insertion(&self) -> bool {
        self.kind().is_insertion()
    }

    /// Returns `true` if this edit replaces some existing content with new content.
    #[inline]
    pub fn is_replacement(&self) -> bool {
        self.kind().is_replacement()
    }
}

impl Ord for Edit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start()
            .cmp(&other.start())
            .then_with(|| self.end().cmp(&other.end()))
            .then_with(|| self.content.cmp(&other.content))
    }
}

impl PartialOrd for Edit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum EditOperationKind {
    /// Edit that inserts new content into the source document.
    Insertion,

    /// Edit that deletes content from the source document.
    Deletion,

    /// Edit that replaces content from the source document.
    Replacement,
}

impl EditOperationKind {
    pub const fn is_insertion(self) -> bool {
        matches!(self, EditOperationKind::Insertion)
    }

    pub const fn is_deletion(self) -> bool {
        matches!(self, EditOperationKind::Deletion)
    }

    pub const fn is_replacement(self) -> bool {
        matches!(self, EditOperationKind::Replacement)
    }
}
