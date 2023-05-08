use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io::{BufReader, Read},
};

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

impl Token {
    pub fn new<T: ToString>(kind: TokenKind, text: T, span: Span) -> Self {
        Self {
            kind,
            text: text.to_string(),
            span,
        }
    }
}

#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub enum TokenKind {
    Doctype(String),       // <!DOCTYPE html>
    Comment(String),       // <!-- this is my comment -->
    TagOpen(TagData),      // <div class="container">
    TagClose(String),      // </div>
    TagSelfClose(TagData), // <img class="wrapper"/>
    Text(String),          // My text
    Eof,
}

#[derive(Debug)]
pub struct TagData {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

pub trait Lexer {
    fn next_char(&mut self) -> Option<char>;

    fn chop_char(&mut self) {
        assert!(self.next_char().is_some());
    }

    fn chop_char_expect(&mut self, expected: char) {
        let c = self
            .next_char()
            .expect("Tried chopping but no more characters");

        assert_eq!(c, expected, "Expected {} but got {}", expected, c);
    }

    fn peek_char(&self) -> Option<char>;
    fn peek_char_nth(&self, n: usize) -> Option<char>;

    fn get_position(&self) -> usize;

    fn has_next(&self) -> bool {
        self.peek_char().is_some()
    }
    fn has_next_nth(&self, n: usize) -> bool {
        self.peek_char_nth(n).is_some()
    }

    fn peek_matches(&self, chars: &str) -> bool {
        chars
            .chars()
            .enumerate()
            .all(|(i, c)| self.peek_char_nth(i).is_some_and(|peeked| peeked == c))
    }

    fn peek_matches_ignore_case(&self, chars: &str) -> bool {
        chars.chars().enumerate().all(|(i, c)| {
            self.peek_char_nth(i)
                .is_some_and(|peeked| peeked.eq_ignore_ascii_case(&c))
        })
    }

    fn next_token(&mut self) -> Option<Token> {
        if self.peek_char().is_none() {
            return Some(Token::new(
                TokenKind::Eof,
                "",
                Span::new(self.get_position(), self.get_position()),
            ));
        }

        while self.has_next() {
            if self.peek_matches("<!--") {
                return Some(self.expect_comment());
            }

            if self.peek_matches_ignore_case("<!DOCTYPE") {
                return Some(self.expect_doctype());
            }

            if self.peek_matches("</") {
                return Some(self.expect_close_tag());
            }

            if self.peek_matches("<") {
                return Some(self.expect_open_or_self_close_tag());
            }

            match self.peek_char().unwrap() {
                c if c.is_whitespace() => {
                    // Munch whitespace
                    while self.peek_char().is_some_and(|c| c.is_whitespace()) {
                        self.chop_char();
                    }
                }
                _ => return Some(self.expect_text()),
            }
        }

        return None;
    }

    fn expect_doctype(&mut self) -> Token {
        let start = self.get_position();
        let mut text = String::new();
        let mut doctype = String::new();

        // "<!DOCTYPE"
        assert!(self.peek_matches_ignore_case("<!DOCTYPE"));
        for _ in 0..9 {
            text.push(self.next_char().unwrap());
        }

        // " "
        assert_eq!(self.peek_char().unwrap(), ' ');
        text.push(self.next_char().unwrap());

        // "html"
        while self.has_next() {
            if self.peek_matches(">") {
                break;
            }

            let c = self.next_char().unwrap();

            text.push(c);
            doctype.push(c);
        }

        // ">"
        assert_eq!(self.peek_char().unwrap(), '>');
        text.push(self.next_char().unwrap());

        // Transforms
        let doctype = doctype.trim().to_ascii_lowercase();
        assert_eq!(doctype, "html");

        Token::new(
            TokenKind::Doctype(doctype),
            text,
            Span::new(start, self.get_position()),
        )
    }

    fn expect_comment(&mut self) -> Token {
        let start = self.get_position();
        let mut text = String::new();
        let mut comment = String::new();

        // "<!--"
        assert!(self.peek_matches("<!--"));
        for _ in 0..4 {
            text.push(self.next_char().unwrap());
        }

        // "this is a comment"
        while self.has_next() {
            if self.peek_matches("-->") {
                break;
            }

            let c = self.next_char().unwrap();

            text.push(c);
            comment.push(c);
        }

        // "-->"
        assert!(self.peek_matches("-->"));
        for _ in 0..3 {
            text.push(self.next_char().unwrap());
        }

        Token::new(
            TokenKind::Comment(comment),
            text,
            Span::new(start, self.get_position()),
        )
    }

    fn expect_close_tag(&mut self) -> Token {
        let start = self.get_position();
        let mut text = String::new();
        let mut tag_name = String::new();

        // "</"
        assert!(self.peek_matches("</"));
        for _ in 0..2 {
            text.push(self.next_char().unwrap());
        }

        // "body"
        while self.has_next() {
            if self.peek_matches(">") {
                break;
            }

            let c = self.next_char().unwrap();

            assert!(
                c.is_ascii_alphabetic() || c.is_ascii_whitespace(),
                "illegal character in tag name"
            );

            text.push(c);
            tag_name.push(c);
        }

        assert!(tag_name.len() > 0);
        assert!(tag_name.trim().len() > 0);
        assert!(
            tag_name.trim().chars().all(|c| !c.is_ascii_whitespace()),
            "spaces within tag close"
        );

        // ">"
        assert_eq!(self.peek_char().unwrap(), '>');
        text.push(self.next_char().unwrap());

        Token::new(
            TokenKind::TagClose(tag_name.trim().to_string()),
            text,
            Span::new(start, self.get_position()),
        )
    }

    fn expect_open_or_self_close_tag(&mut self) -> Token {
        todo!("parse open or self close tag")
    }

    fn expect_text(&mut self) -> Token {
        let start = self.get_position();
        let mut text = String::new();

        while self.has_next() {
            if self.peek_matches("<") {
                break;
            }

            text.push(self.next_char().unwrap());
        }

        assert_eq!(self.peek_char().unwrap(), '>', "Unexpected end of file");

        Token::new(
            TokenKind::Text(text.trim().to_string()),
            text,
            Span::new(start, self.get_position()),
        )
    }
}

pub struct StringLexer {
    input: String,
    position: usize,
}

impl StringLexer {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }
}

impl Lexer for StringLexer {
    fn next_char(&mut self) -> Option<char> {
        let c = self.input.chars().nth(self.position)?;
        self.position += 1;

        Some(c)
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn get_position(&self) -> usize {
        self.position
    }

    fn peek_char_nth(&self, n: usize) -> Option<char> {
        self.input.chars().nth(self.position + n)
    }
}

pub struct StreamLexer<T: Read> {
    input: RefCell<BufReader<T>>,
    position: usize,
    peeked: RefCell<VecDeque<char>>,
}

impl<T: Read> StreamLexer<T> {
    pub fn new(input: T) -> Self {
        Self {
            input: RefCell::new(BufReader::new(input)),
            position: 0,
            peeked: RefCell::new(VecDeque::new()),
        }
    }
}

impl<T: Read> Lexer for StreamLexer<T> {
    fn next_char(&mut self) -> Option<char> {
        if !self.peeked.borrow().is_empty() {
            let c = self.peeked.borrow_mut().pop_front();
            self.position += 1;
            return c;
        }

        let mut buf = [0; 1];
        let num_bytes = self
            .input
            .borrow_mut()
            .read(&mut buf)
            .expect("Could not read from stream");

        if num_bytes == 0 {
            return None;
        }

        self.position += 1;

        Some(buf[0] as char)
    }

    fn peek_char(&self) -> Option<char> {
        if !self.peeked.borrow().is_empty() {
            return self.peeked.borrow().front().cloned();
        }

        let mut buf = [0; 1];
        let num_bytes = self
            .input
            .borrow_mut()
            .read(&mut buf)
            .expect("Could not read from stream");

        if num_bytes == 0 {
            return None;
        }

        self.peeked.borrow_mut().push_front(buf[0] as char);
        self.peeked.borrow().front().cloned()
    }

    fn get_position(&self) -> usize {
        self.position
    }

    fn peek_char_nth(&self, n: usize) -> Option<char> {
        if self.peeked.borrow().len() > n {
            return self.peeked.borrow().get(n).cloned();
        }

        let chars_to_peek = n + 1 - self.peeked.borrow().len();

        let mut buf = vec![0; chars_to_peek];
        let num_bytes = self
            .input
            .borrow_mut()
            .read(&mut buf)
            .expect("Could not read from stream");

        if num_bytes < chars_to_peek {
            return None;
        }

        for c in buf {
            self.peeked.borrow_mut().push_back(c as char);
        }

        self.peeked.borrow().get(n).cloned()
    }
}
