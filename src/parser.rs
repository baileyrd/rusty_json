use crate::{Error, Number, Value};
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
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(other) => {
                Err(self.error(alloc::format!("unexpected character `{}`", other as char)))
            }
            None => Err(self.error("unexpected end of input")),
        }
    }

    /// Parses a JSON number per RFC 8259 §6:
    /// `number = [ "-" ] int [ frac ] [ exp ]`.
    fn parse_number(&mut self) -> Result<Value, Error> {
        let start = self.pos;

        if self.peek() == Some(b'-') {
            self.bump();
        }

        match self.peek() {
            Some(b'0') => {
                self.bump();
            }
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.bump();
                }
            }
            _ => return Err(self.error("invalid number: expected a digit")),
        }

        let mut is_float = false;

        if self.peek() == Some(b'.') {
            is_float = true;
            self.bump();
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("invalid number: expected a digit after `.`"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.bump();
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.bump();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.bump();
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("invalid number: expected a digit in exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.bump();
            }
        }

        // Safe: we only ever consumed ASCII digits and `-`/`.`/`e`/`E`/`+`.
        let raw = core::str::from_utf8(&self.input[start..self.pos]).unwrap();

        if is_float {
            let f: f64 = raw.parse().map_err(|_| self.error("invalid number"))?;
            return Number::from_f64(f)
                .map(Value::Number)
                .ok_or_else(|| self.error("number is not finite"));
        }

        let negative = raw.starts_with('-');
        let magnitude = if negative { &raw[1..] } else { raw };

        if let Ok(mag) = magnitude.parse::<u64>() {
            if !negative {
                return Ok(Value::Number(Number::from(mag)));
            }
            if mag == 1u64 << 63 {
                return Ok(Value::Number(Number::from(i64::MIN)));
            }
            if let Ok(n) = i64::try_from(mag) {
                return Ok(Value::Number(Number::from(-n)));
            }
        }

        // Integer too large for i64/u64: fall back to a lossy f64, same as
        // serde_json does for out-of-range integers.
        let f: f64 = raw.parse().map_err(|_| self.error("invalid number"))?;
        Number::from_f64(f)
            .map(Value::Number)
            .ok_or_else(|| self.error("number is not finite"))
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

    fn num(s: &str) -> Value {
        Parser::parse(s).unwrap()
    }

    #[test]
    fn parses_integers() {
        assert_eq!(num("0"), Value::Number(Number::from(0u64)));
        assert_eq!(num("42"), Value::Number(Number::from(42u64)));
        assert_eq!(num("-42"), Value::Number(Number::from(-42i64)));
        assert_eq!(num("-0"), Value::Number(Number::from(0u64)));
    }

    #[test]
    fn parses_i64_min() {
        assert_eq!(
            num("-9223372036854775808"),
            Value::Number(Number::from(i64::MIN))
        );
    }

    #[test]
    fn parses_floats() {
        assert_eq!(num("1.5"), Value::Number(Number::from_f64(1.5).unwrap()));
        assert_eq!(num("1e10"), Value::Number(Number::from_f64(1e10).unwrap()));
        assert_eq!(
            num("1.5e-2"),
            Value::Number(Number::from_f64(1.5e-2).unwrap())
        );
        assert_eq!(
            num("-1.5E+2"),
            Value::Number(Number::from_f64(-1.5e2).unwrap())
        );
    }

    #[test]
    fn rejects_leading_zero() {
        assert!(Parser::parse("01").is_err());
        assert!(Parser::parse("-01").is_err());
    }

    #[test]
    fn rejects_malformed_numbers() {
        assert!(Parser::parse("-").is_err());
        assert!(Parser::parse(".5").is_err());
        assert!(Parser::parse("1.").is_err());
        assert!(Parser::parse("1e").is_err());
        assert!(Parser::parse("1e+").is_err());
        assert!(Parser::parse("+1").is_err());
    }

    #[test]
    fn overflow_falls_back_to_f64() {
        // Larger than u64::MAX; must not error, must not panic.
        let v = num("100000000000000000000000000000");
        match v {
            Value::Number(n) => assert!(n.is_f64()),
            _ => panic!("expected a number"),
        }
    }
}
