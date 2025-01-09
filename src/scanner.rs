use std::collections::HashMap;
use std::rc::Rc;

use crate::tokens::{Token, TokenType};
use crate::utils::{error, is_alpha, is_alphanumeric, is_digit, substring};
use crate::Code;

pub struct Scanner<'a> {
    source: &'a String,
    pub tokens: Vec<Token>,
    pub labels: HashMap<Rc<str>, usize>,
    start_positions: Vec<usize>,
    
    start: usize,
    curr:  usize,
    line:  usize,

    pub had_error: bool
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Scanner {
            source, tokens: Vec::new(), start_positions: Vec::new(),
            labels: HashMap::new(), start: 0, curr: 0, line: 0, had_error: false
        }
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.curr).unwrap();
        self.curr += 1;
        c
    }

    fn get_substring(&mut self) -> Rc<str> {
        substring(&self.source, self.start, self.curr).into()
    }

    fn add_token(&mut self, type_: TokenType) {
        let lexeme = self.get_substring();
        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()), 
            type_, lexeme, 
            self.start - self.start_positions[self.line], 
            self.curr - self.start_positions[self.line], self.line
        ));
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.curr).unwrap()
        }
    }

    fn error(&mut self, msg: &str) {
        error(
            &Rc::from(self.source.as_ref()), msg, 
            self.start - self.start_positions[self.line], 
            self.curr - self.start, self.line
        );
        
        self.had_error = true;
    }

    fn string(&mut self, ch: char) {
        let mut back_slash = false;
        loop {
            let c = self.peek();
            if self.is_at_end() || (c == ch && !back_slash) {
                break;
            }

            let old_backslash = back_slash;
            back_slash = false;

            match c {
                '\n' => self.line += 1,
                '\\' => {
                    if !old_backslash {
                        back_slash = true;
                    }
                }
                _ => (),
            }

            self.advance();
        }

        if self.is_at_end() {
            self.error("Unterminated string");
            return;
        }

        self.advance();
    }
    
    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        let c = self.peek();
        if c == '.' {
            self.advance();

            if is_digit(self.peek()) {
                loop {
                    self.advance();

                    if !is_digit(self.peek()) {
                        break;
                    }
                }
            } else {
                self.error("Expecting digits after decimal point");
            }
            
            let parsed = self.get_substring().parse().unwrap();
            self.add_token(TokenType::Float(parsed));
        } else {
            let parsed = self.get_substring().parse().unwrap();
            self.add_token(TokenType::Int(parsed));
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ';' => self.add_token(TokenType::Semicolon),
            '@' => self.add_token(TokenType::At),
            '?' => self.add_token(TokenType::Question),
            '~' => self.add_token(TokenType::Tilde),
            '=' => self.add_token(TokenType::Equal),
            '*' => self.add_token(TokenType::Star),
            '%' => self.add_token(TokenType::Mod),
            '^' => self.add_token(TokenType::Jump),
            ':' => self.add_token(TokenType::Colon),
            '+' => self.add_token(TokenType::Plus),
            '|' => self.add_token(TokenType::Or),
            '&' => self.add_token(TokenType::And),
            '-' => self.add_token(TokenType::Minus),
            '!' => self.add_token(TokenType::Bang),
            '<' => self.add_token(TokenType::Less),
            '>' => self.add_token(TokenType::Greater),
            '/' => self.add_token(TokenType::Slash),
            '#' => self.add_token(TokenType::Hash),
            '$' => self.add_token(TokenType::Print),
            '"'  => {
                self.string('"');
                self.start += 1;
                self.curr -= 1;
                
                let parsed = self.get_substring();
                self.add_token(TokenType::String(parsed));

                self.start -= 1;
                self.curr += 1;
            }
            '['  => {
                self.string(']');
                self.start += 1;
                self.curr -= 1;
                
                let name = self.get_substring();
                self.labels.insert(name, self.tokens.len() - 1);

                self.start -= 1;
                self.curr += 1;
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            '{' => {
                loop {
                    let c = self.peek();
                    if self.is_at_end() || (c == '}') {
                        break;
                    }

                    if c == '\n' {
                        self.line += 1;
                    }
        
                    self.advance();
                }

                if self.is_at_end() {
                    self.error("Unterminated code block");
                    return;
                }

                self.advance();
                
                self.start += 1;
                self.curr -= 1;
                let source = self.get_substring().to_string();
                self.start -= 1;
                self.curr += 1;

                let mut scanner = Scanner::new(&source);
                scanner.scan_tokens();
                self.start = self.curr;
                self.add_token(TokenType::Code(Code::new(scanner.tokens, scanner.labels)));
            }

            _ => {
                if is_digit(c) {
                    self.number();
                } else if is_alpha(c) {
                    while is_alphanumeric(self.peek()) {
                        self.advance();
                    }
            
                    self.add_token(TokenType::Identifier);
                } else {
                    self.error("Unexpected character");
                }
            }
        }
    }

    fn get_start_positions(&mut self) {
        self.start_positions.push(0);
        for (i, c) in self.source.chars().enumerate() {
            if c == '\n' {
                self.start_positions.push(i + 1);
            }
        }
    }

    pub fn scan_tokens(&mut self) {
        self.get_start_positions();
        
        while !self.is_at_end() {
            self.start = self.curr;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()), 
            TokenType::EOF, Rc::from(""), 
            0, 1, self.line
        ));
    }
}