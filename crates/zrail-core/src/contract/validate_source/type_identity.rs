//! Exact field types accept only the recursively rendered Rust subset.

mod cursor;

use cursor::Cursor;

pub(super) fn valid_exact_type(source: &str) -> bool {
    let mut parser = Parser::new(source);
    parser.parse_type() && parser.finish()
}

struct Parser<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
    const fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
        }
    }

    fn parse_type(&mut self) -> bool {
        self.cursor.whitespace();
        match self.cursor.peek() {
            Some(b'!') => {
                self.cursor.advance();
                true
            }
            Some(b'&') => self.parse_reference(),
            Some(b'*') => self.parse_pointer(),
            Some(b'[') => self.parse_brackets(),
            Some(b'(') => self.parse_parentheses(),
            _ => self.parse_path(),
        }
    }

    fn parse_reference(&mut self) -> bool {
        self.cursor.advance();
        self.cursor.whitespace();
        if self.cursor.peek() == Some(b'\'') && !self.cursor.parse_lifetime() {
            return false;
        }
        self.cursor.whitespace();
        self.cursor.consume_keyword("mut");
        self.parse_type()
    }

    fn parse_pointer(&mut self) -> bool {
        self.cursor.advance();
        self.cursor.whitespace();
        if !self.cursor.consume_keyword("const") && !self.cursor.consume_keyword("mut") {
            return false;
        }
        self.parse_type()
    }

    fn parse_brackets(&mut self) -> bool {
        self.cursor.advance();
        if !self.parse_type() {
            return false;
        }
        self.cursor.whitespace();
        if self.cursor.consume(b']') {
            return true;
        }
        self.cursor.consume(b';') && self.parse_const() && self.cursor.consume_after_space(b']')
    }

    fn parse_parentheses(&mut self) -> bool {
        self.cursor.advance();
        self.cursor.whitespace();
        if self.cursor.consume(b')') {
            return true;
        }
        if !self.parse_type() {
            return false;
        }
        self.cursor.whitespace();
        if self.cursor.consume(b')') {
            return true;
        }
        while self.cursor.consume(b',') {
            self.cursor.whitespace();
            if self.cursor.consume(b')') {
                return true;
            }
            if !self.parse_type() {
                return false;
            }
            self.cursor.whitespace();
            if self.cursor.consume(b')') {
                return true;
            }
        }
        false
    }

    fn parse_path(&mut self) -> bool {
        let Some(first) = self.cursor.identifier() else {
            return false;
        };
        if unsupported(first) {
            return false;
        }
        let mut segments = 1;
        loop {
            self.cursor.whitespace();
            if self.cursor.consume_pair(b':', b':') {
                self.cursor.whitespace();
                if self.cursor.identifier().is_none() {
                    return false;
                }
                segments += 1;
                continue;
            }
            break;
        }
        self.cursor.whitespace();
        if self.cursor.consume(b'<') && !self.parse_arguments() {
            return false;
        }
        (segments > 1 || primitive(first)) && !self.cursor.next_is_path_segment()
    }

    fn parse_arguments(&mut self) -> bool {
        self.cursor.whitespace();
        if self.cursor.consume(b'>') {
            return false;
        }
        loop {
            if !self.parse_argument() {
                return false;
            }
            self.cursor.whitespace();
            if self.cursor.consume(b'>') {
                return true;
            }
            if !self.cursor.consume(b',') {
                return false;
            }
            self.cursor.whitespace();
            if self.cursor.consume(b'>') {
                return true;
            }
        }
    }

    fn parse_argument(&mut self) -> bool {
        self.cursor.whitespace();
        if self.cursor.peek() == Some(b'\'') && !self.cursor.char_literal_ahead() {
            return self.cursor.parse_lifetime();
        }
        if self.cursor.literal_ahead() {
            return self.cursor.parse_literal();
        }
        self.parse_type()
    }

    fn parse_const(&mut self) -> bool {
        self.cursor.whitespace();
        if self.cursor.literal_ahead() {
            return self.cursor.parse_literal();
        }
        self.parse_path()
    }

    fn finish(&mut self) -> bool {
        self.cursor.finish()
    }
}

fn primitive(value: &str) -> bool {
    matches!(
        value,
        "bool"
            | "char"
            | "str"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
    )
}

fn unsupported(value: &str) -> bool {
    matches!(value, "_" | "dyn" | "fn" | "impl" | "for")
}
