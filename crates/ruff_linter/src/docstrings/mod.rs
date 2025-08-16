use std::fmt::{Debug, Formatter};
use std::ops::Deref;

use ruff_python_ast::{self as ast, StringFlags};
use ruff_python_semantic::Definition;
use ruff_source_file::LineRanges;
use ruff_text_size::{Ranged, TextRange, TextSize};

pub mod extraction;
pub mod google;
pub mod numpy;
pub mod sections;
pub mod styles;

#[derive(Debug)]
pub struct Docstring<'a> {
    pub definition: &'a Definition<'a>,
    /// The literal AST node representing the docstring.
    pub expr: &'a ast::StringLiteral,
    /// The source file the docstring was defined in.
    pub source: &'a str,
}

impl<'a> Docstring<'a> {
    fn flags(&self) -> ast::StringLiteralFlags {
        self.expr.flags
    }

    /// The contents of the docstring, including the opening and closing quotes.
    pub fn contents(&self) -> &'a str {
        &self.source[self.range()]
    }

    /// The contents of the docstring, excluding the opening and closing quotes.
    pub fn body(&self) -> DocstringBody<'_> {
        DocstringBody { docstring: self }
    }

    /// Compute the start position of the docstring's opening line
    pub fn line_start(&self) -> TextSize {
        self.source.line_start(self.start())
    }

    /// Return the slice of source code that represents the indentation of the docstring's opening quotes.
    pub fn compute_indentation(&self) -> &'a str {
        &self.source[TextRange::new(self.line_start(), self.start())]
    }

    pub fn quote_style(&self) -> ast::str::Quote {
        self.flags().quote_style()
    }

    pub fn is_raw_string(&self) -> bool {
        self.flags().prefix().is_raw()
    }

    pub fn is_u_string(&self) -> bool {
        self.flags().prefix().is_unicode()
    }

    pub fn is_triple_quoted(&self) -> bool {
        self.flags().is_triple_quoted()
    }

    /// The docstring's prefixes as they exist in the original source code.
    pub fn prefix_str(&self) -> &'a str {
        // N.B. This will normally be exactly the same as what you might get from
        // `self.flags().prefix().as_str()`, but doing it this way has a few small advantages.
        // For example, the casing of the `u` prefix will be preserved if it's a u-string.
        &self.source[TextRange::new(
            self.start(),
            self.start() + self.flags().prefix().text_len(),
        )]
    }

    /// The docstring's "opener" (the string's prefix, if any, and its opening quotes).
    pub fn opener(&self) -> &'a str {
        &self.source[TextRange::new(self.start(), self.start() + self.flags().opener_len())]
    }

    /// The docstring's closing quotes.
    pub fn closer(&self) -> &'a str {
        &self.source[TextRange::new(self.end() - self.flags().closer_len(), self.end())]
    }
}

impl Ranged for Docstring<'_> {
    fn range(&self) -> TextRange {
        self.expr.range()
    }
}

#[derive(Copy, Clone)]
pub struct DocstringBody<'a> {
    docstring: &'a Docstring<'a>,
}

impl<'a> DocstringBody<'a> {
    pub fn as_str(self) -> &'a str {
        &self.docstring.source[self.range()]
    }
}

impl Ranged for DocstringBody<'_> {
    fn range(&self) -> TextRange {
        self.docstring.expr.content_range()
    }
}

impl Deref for DocstringBody<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Debug for DocstringBody<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocstringBody")
            .field("text", &self.as_str())
            .field("range", &self.range())
            .finish()
    }
}
