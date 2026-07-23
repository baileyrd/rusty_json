use crate::{Error, Value};
use alloc::string::String;

pub(crate) struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Parser {
            input: input.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Parses a complete JSON document: one value, optionally surrounded by
    /// whitespace, with nothing left over afterward.
    pub(crate) fn parse(input: &'a str) -> Result<Value, Error> {
        let mut parser = Parser::new(input);
        let value = parser.parse_value()?;
        parser.skip_whitespace();
        if parser.pos != parser.input.len() {
            return Err(parser.error("trailing characters after JSON value"));
        }
        Ok(value)
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        if byte == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(byte)
    }

    fn error(&self, msg: impl Into<String>) -> Error {
        Error::new(msg, self.line, self.column)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.bump();
        }
    }

    fn expect_literal(&mut self, literal: &str, value: Value) -> Result<Value, Error> {
        for expected in literal.bytes() {
            match self.bump() {
                Some(byte) if byte == expected => {}
                _ => {
                    return Err(self.error(alloc::format!("invalid literal, expected `{literal}`")))
                }
            }
        }
        Ok(value)
    }

    pub(crate) fn parse_value(&mut self) -> Result<Value, Error> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'n') => self.expect_literal("null", Value::Null),
            Some(b't') => self.expect_literal("true", Value::Bool(true)),
            Some(b'f') => self.expect_literal("false", Value::Bool(false)),
            Some(other) => {
                Err(self.error(alloc::format!("unexpected character `{}`", other as char)))
            }
            None => Err(self.error("unexpected end of input")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_literals() {
        assert_eq!(Parser::parse("null").unwrap(), Value::Null);
        assert_eq!(Parser::parse("true").unwrap(), Value::Bool(true));
        assert_eq!(Parser::parse("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn skips_surrounding_whitespace() {
        assert_eq!(Parser::parse("  \n\t null  \r\n").unwrap(), Value::Null);
    }

    #[test]
    fn rejects_invalid_literal() {
        assert!(Parser::parse("nul").is_err());
        assert!(Parser::parse("truee").is_err());
        assert!(Parser::parse("nulll").is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(Parser::parse("").is_err());
        assert!(Parser::parse("   ").is_err());
    }

    #[test]
    fn error_reports_line_and_column() {
        let err = Parser::parse("\n\n  bogus").unwrap_err();
        assert_eq!(err.line(), 3);
        assert_eq!(err.column(), 3);
    }
}
