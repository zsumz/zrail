//! Byte-level cursor for the bounded exact-type grammar.

pub(super) struct Cursor<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> Cursor<'a> {
    pub(super) const fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    pub(super) fn advance(&mut self) {
        self.offset += 1;
    }

    pub(super) fn parse_literal(&mut self) -> bool {
        self.whitespace();
        let remaining = &self.source[self.offset..];
        if remaining.starts_with("true") || remaining.starts_with("false") {
            let keyword = if remaining.starts_with("true") {
                "true"
            } else {
                "false"
            };
            return self.consume_keyword(keyword);
        }
        if remaining.starts_with('\'') {
            return self.parse_char_literal();
        }
        let start = self.offset;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.offset += 1;
        }
        self.offset > start
            && self.source[start..self.offset]
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_digit())
    }

    fn parse_char_literal(&mut self) -> bool {
        if !self.consume(b'\'') {
            return false;
        }
        let mut escaped = false;
        while let Some(byte) = self.peek() {
            self.offset += 1;
            if byte == b'\'' && !escaped {
                return true;
            }
            escaped = byte == b'\\' && !escaped;
            if byte != b'\\' {
                escaped = false;
            }
        }
        false
    }

    pub(super) fn parse_lifetime(&mut self) -> bool {
        self.consume(b'\'') && self.identifier().is_some_and(|lifetime| lifetime != "_")
    }

    pub(super) fn identifier(&mut self) -> Option<&'a str> {
        self.whitespace();
        let start = self.offset;
        if self.source[self.offset..].starts_with("r#") {
            self.offset += 2;
        }
        let first = self.peek()?;
        if !first.is_ascii_alphabetic() && first != b'_' {
            self.offset = start;
            return None;
        }
        self.offset += 1;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.offset += 1;
        }
        Some(&self.source[start..self.offset])
    }

    pub(super) fn literal_ahead(&self) -> bool {
        self.peek().is_some_and(|byte| byte.is_ascii_digit())
            || self.char_literal_ahead()
            || self.source[self.offset..].starts_with("true")
            || self.source[self.offset..].starts_with("false")
    }

    pub(super) fn char_literal_ahead(&self) -> bool {
        let Some(rest) = self.source[self.offset..].strip_prefix('\'') else {
            return false;
        };
        let mut escaped = false;
        for byte in rest.bytes() {
            if byte == b'\'' && !escaped {
                return true;
            }
            if !escaped && (byte.is_ascii_whitespace() || matches!(byte, b',' | b'>')) {
                return false;
            }
            escaped = byte == b'\\' && !escaped;
            if byte != b'\\' {
                escaped = false;
            }
        }
        false
    }

    pub(super) fn next_is_path_segment(&mut self) -> bool {
        self.whitespace();
        self.source[self.offset..].starts_with("::")
    }

    pub(super) fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.whitespace();
        let remaining = &self.source[self.offset..];
        if !remaining.starts_with(keyword) {
            return false;
        }
        let end = self.offset + keyword.len();
        if self
            .source
            .as_bytes()
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            return false;
        }
        self.offset = end;
        true
    }

    pub(super) fn consume_after_space(&mut self, byte: u8) -> bool {
        self.whitespace();
        self.consume(byte)
    }

    pub(super) fn consume_pair(&mut self, first: u8, second: u8) -> bool {
        if self.source.as_bytes().get(self.offset..self.offset + 2) == Some(&[first, second]) {
            self.offset += 2;
            true
        } else {
            false
        }
    }

    pub(super) fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn whitespace(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.offset += 1;
        }
    }

    pub(super) fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }

    pub(super) fn finish(&mut self) -> bool {
        self.whitespace();
        self.offset == self.source.len()
    }
}
