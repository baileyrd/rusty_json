use alloc::string::String;
use core::fmt;

/// An error produced while parsing JSON, carrying the 1-based line/column
/// at which the problem was found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    msg: String,
    line: usize,
    column: usize,
}

impl Error {
    pub(crate) fn new(msg: impl Into<String>, line: usize, column: usize) -> Self {
        Error {
            msg: msg.into(),
            line,
            column,
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
}
