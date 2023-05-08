#![feature(is_some_and)]

use std::{collections::HashMap, io::Read};

use lexer::{StreamLexer, StringLexer, Lexer};

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
    while lexer.has_next() {
        println!("{:?}", lexer.next_token().unwrap());
    }
    
    todo!("Parse HTML")
}
