use std::rc::Rc;

use crate::Code;

#[derive(Debug, Clone)]
pub enum TokenType {
    Comma, Dot, Minus, Plus, Semicolon,
    Slash, Star, Colon,  At, Mod, Tilde,
    Bang, Question, Print,
    Equal, Greater, Less, 
    Hash, And, Or, Jump,

    Identifier, String(Rc<str>), 
    Int(i64), Float(f64), Code(Code),
    
    EOF
}

#[derive(Clone)]
pub struct Token {
    pub source: Rc<str>,
    pub type_: TokenType,
    pub lexeme: Rc<str>,
    pub pos: usize,
    pub end: usize,
    pub line: usize
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token").field("type_", &self.type_).field("lexeme", &self.lexeme).field("pos", &self.pos).field("end", &self.end).field("line", &self.line).finish()
    }
}

impl Token {
    pub fn new(source: Rc<str>, type_: TokenType, lexeme: Rc<str>, pos: usize, end: usize, line: usize) -> Self {
        Token { source, type_, lexeme, pos, end, line }
    }
}