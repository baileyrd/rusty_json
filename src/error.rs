use alloc::string::String;
use core::fmt;

/// Broad classification of what kind of problem an [`Error`] represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    /// The input wasn't syntactically valid JSON.
    Syntax,
    /// Input ended before a complete JSON value was parsed.
    Eof,
    /// JSON was well-formed but didn't match an expected data shape.
    /// Unreachable today (this crate has no typed deserialization yet);
    /// reserved for a future round.
    Data,
    /// An I/O error occurred while reading input. Unreachable today (this
    /// crate has no reader-based parsing yet); reserved for a future round.
    Io,
}

/// An error produced while parsing JSON, carrying the 1-based line/column
/// at which the problem was found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    msg: String,
    line: usize,
    column: usize,
    category: Category,
}

impl Error {
    pub(crate) fn new(msg: impl Into<String>, line: usize, column: usize) -> Self {
        Error {
            msg: msg.into(),
            line,
            column,
            category: Category::Syntax,
        }
    }

    pub(crate) fn eof(msg: impl Into<String>, line: usize, column: usize) -> Self {
        Error {
            msg: msg.into(),
            line,
            column,
            category: Category::Eof,
        }
    }

    /// A data-shape error: JSON parsed fine but didn't match what a
    /// `Deserialize` impl expected. Has no meaningful position, since it's
    /// raised after parsing succeeds; `line()`/`column()` are `0`.
    fn data(msg: impl Into<String>) -> Self {
        Error {
            msg: msg.into(),
            line: 0,
            column: 0,
            category: Category::Data,
        }
    }

    /// The 1-based line at which the error occurred.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The 1-based column at which the error occurred.
    pub fn column(&self) -> usize {
        self.column
    }

    /// A human-readable description of the error, without position info.
    pub fn message(&self) -> &str {
        &self.msg
    }

    /// This error's broad category.
    pub fn classify(&self) -> Category {
        self.category
    }

    /// True if the input wasn't syntactically valid JSON.
    pub fn is_syntax(&self) -> bool {
        self.category == Category::Syntax
    }

    /// True if the input ended before a complete JSON value was parsed.
    pub fn is_eof(&self) -> bool {
        self.category == Category::Eof
    }

    /// True if JSON was well-formed but didn't match an expected data
    /// shape. Always `false` today; reserved for a future round.
    pub fn is_data(&self) -> bool {
        self.category == Category::Data
    }

    /// True if an I/O error occurred while reading input. Always `false`
    /// today; reserved for a future round.
    pub fn is_io(&self) -> bool {
        self.category == Category::Io
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {} column {}",
            self.msg, self.line, self.column
        )
    }
}

// Unconditional (not gated behind the `std` feature): `core::error::Error`
// is itself no_std-compatible, and `serde::ser::Error` requires it as a
// supertrait regardless of serde's own `std` feature.
impl core::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::data(alloc::format!("{msg}"))
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::data(alloc::format!("{msg}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn display_includes_position() {
        let err = Error::new("unexpected token", 3, 7);
        assert_eq!(err.to_string(), "unexpected token at line 3 column 7");
        assert_eq!(err.line(), 3);
        assert_eq!(err.column(), 7);
        assert_eq!(err.message(), "unexpected token");
    }

    #[test]
    fn classification() {
        let syntax = Error::new("bad token", 1, 1);
        assert_eq!(syntax.classify(), Category::Syntax);
        assert!(syntax.is_syntax());
        assert!(!syntax.is_eof());
        assert!(!syntax.is_data());
        assert!(!syntax.is_io());

        let eof = Error::eof("unexpected end of input", 1, 1);
        assert_eq!(eof.classify(), Category::Eof);
        assert!(eof.is_eof());
        assert!(!eof.is_syntax());
    }

    #[test]
    fn serde_custom_errors_classify_as_data() {
        let de_err = <Error as serde::de::Error>::custom("bad shape");
        assert!(de_err.is_data());
        assert_eq!(de_err.line(), 0);
        assert_eq!(de_err.column(), 0);

        let ser_err = <Error as serde::ser::Error>::custom("cannot serialize");
        assert!(ser_err.is_data());
    }

    #[cfg(feature = "std")]
    #[test]
    fn composes_with_std_error() {
        fn assert_is_std_error<E: std::error::Error>(_: &E) {}
        let err = Error::new("bad token", 1, 1);
        assert_is_std_error(&err);
        let _boxed: alloc::boxed::Box<dyn std::error::Error> = alloc::boxed::Box::new(err);
    }
}
