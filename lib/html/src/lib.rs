#![feature(is_some_and)]

use std::{
    collections::{HashMap, VecDeque},
    io::Read,
};

use lexer::{Lexer, StreamLexer, StringLexer};

use crate::lexer::Token;

mod lexer;

pub struct Document {
    pub root: Node,
}

pub enum Node {
    Text(String),
    Tag(Tag),
}

pub struct Tag {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
}

pub fn parse_string(html: String) -> Document {
    let lexer = StringLexer::new(html);
    parse_html(lexer)
}

pub fn parse_stream(html_stream: impl Read) -> Document {
    let lexer = StreamLexer::new(html_stream);
    parse_html(lexer)
}

fn parse_html(mut lexer: impl Lexer) -> Document {
    let mut tokens: VecDeque<Token> = VecDeque::new();

    let mut token = lexer.next_token();
    while let Some(t) = token {
        tokens.push_back(t);

        token = lexer.next_token();
    }

    println!("{:#?}", tokens);

    todo!("Parse HTML")
}
