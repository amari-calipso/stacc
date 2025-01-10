use std::{collections::HashMap, io::Error, ops::Rem, rc::Rc};

use scanner::Scanner;
use tokens::{Token, TokenType};

const DEBUG: bool = false;

mod tokens;
mod scanner;
mod utils;

#[derive(Debug, Clone)]
struct Code {
    pub tokens: Vec<Token>,
    pub labels: HashMap<Rc<str>, usize>
}

impl Code {
    pub fn new(tokens: Vec<Token>, labels: HashMap<Rc<str>, usize>) -> Self {
        Code { tokens, labels }
    }
}

#[derive(Debug, Clone)]
enum Object {
    Int(i64),
    Float(f64),
    String(Rc<str>),
    Code(Code)
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Int(x)   => *x != 0,
            Object::Float(x) => *x != 0.0,
            Object::String(_) | Object::Code(_) => true,
        }
    }

    pub fn print(&self) {
        match self {
            Object::Int(x)        => println!("{}", x),
            Object::Float(x)      => println!("{}", x),
            Object::String(x) => println!("{}", x),
            Object::Code(_)             => println!("<Code object>"),
        }
    }
}

struct Interpreter {
    st_stack: Vec<Object>,
    nd_stack: Vec<Object>,
    functions: HashMap<Rc<str>, Code>,
}

macro_rules! simple_binary {
    ($slf: ident, $tok: ident, $op: tt, $on_int_op: ident) => {
        {
            let b = $slf.checked_pop($tok)?;
            let a = $slf.checked_pop($tok)?;
                        
            match a {
                Object::Int(x) => {
                    match b {
                        Object::Int(y)   => $slf.st_stack.push(Object::Int(x.$on_int_op(y))),
                        Object::Float(y) => $slf.st_stack.push(Object::Float(x as f64 $op y)),
                        _ => {
                            token_runtime_error!(
                                $tok, 
                                format!("Cannot perform this operation on type {:?}", b).as_ref()
                            );
                            return Err(());
                        }
                    }
                }
                Object::Float(x) => {
                    match b {
                        Object::Int(y)   => $slf.st_stack.push(Object::Float(x $op y as f64)),
                        Object::Float(y) => $slf.st_stack.push(Object::Float(x $op y)),
                        _ => {
                            token_runtime_error!(
                                $tok, 
                                format!("Cannot perform this operation on type {:?}", b).as_ref()
                            );
                            return Err(());
                        }
                    }
                }
                _ => {
                    token_runtime_error!(
                        $tok, 
                        format!("Cannot perform this operation on type {:?}", a).as_ref()
                    );
                    return Err(());
                }
            }
        }
    };
}

macro_rules! bitwise_binary {
    ($slf: ident, $tok: ident, $op: tt) => {
        {
            let b = $slf.checked_pop($tok)?;
            let a = $slf.checked_pop($tok)?;

            if let Object::Int(x) = a {
                if let Object::Int(y) = b {
                    $slf.st_stack.push(Object::Int(x $op y));
                } else {
                    token_runtime_error!(
                        $tok, 
                        format!("Cannot perform this operation on type {:?}", b).as_ref()
                    );
                    return Err(());
                }
            } else {
                token_runtime_error!(
                    $tok, 
                    format!("Cannot perform this operation on type {:?}", a).as_ref()
                );
                return Err(());
            }
        }
    };
}

macro_rules! cmp_binary {
    ($slf: ident, $tok: ident, $op: tt) => {
        {
            let b = $slf.checked_pop($tok)?;
            let a = $slf.checked_pop($tok)?;
                        
            let result = {
                match a {
                    Object::Int(x) => {
                        match b {
                            Object::Int(y)   => x $op y,
                            Object::Float(y) => (x as f64) $op y,
                            _ => {
                                token_runtime_error!(
                                    $tok, 
                                    format!("Cannot perform this operation on type {:?}", b).as_ref()
                                );
                                return Err(());
                            }
                        }
                    }
                    Object::Float(x) => {
                        match b {
                            Object::Int(y)   => x $op y as f64,
                            Object::Float(y) => x $op y,
                            _ => {
                                token_runtime_error!(
                                    $tok, 
                                    format!("Cannot perform this operation on type {:?}", b).as_ref()
                                );
                                return Err(());
                            }
                        }
                    }
                    Object::String(x) => {
                        match b {
                            Object::String(y) => x $op y,
                            _ => {
                                token_runtime_error!(
                                    $tok, 
                                    format!("Cannot perform this operation on type {:?}", b).as_ref()
                                );
                                return Err(());
                            }
                        }
                    }
                    _ => {
                        token_runtime_error!(
                            $tok, 
                            format!("Cannot perform this operation on type {:?}", a).as_ref()
                        );
                        return Err(());
                    }
                }
            };

            $slf.st_stack.push(Object::Int(result as i64));
        }
    };
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            st_stack: Vec::new(),
            nd_stack: Vec::new(),
            functions: HashMap::new(),
        }
    }

    pub fn checked_pop(&mut self, tok: &Token) -> Result<Object, ()> {
        if let Some(popped) = self.st_stack.pop() {
            Ok(popped)
        } else {
            token_runtime_error!(tok, "Popped empty primary stack");
            Err(())
        }
    }

    pub fn checked_pop_nd(&mut self, tok: &Token) -> Result<Object, ()> {
        if let Some(popped) = self.nd_stack.pop() {
            Ok(popped)
        } else {
            token_runtime_error!(tok, "Popped empty secondary stack");
            Err(())
        }
    }
 
    pub async fn execute(&mut self, code: &Code, ctx: &mut reblessive::Stk) -> Result<(), ()> {
        let mut i = 0usize;
        while i < code.tokens.len() {
            if DEBUG {
                println!(" FIRST STACK: {:?}", self.st_stack);
                println!("SECOND STACK: {:?}", self.nd_stack);
                println!("  NEXT TOKEN: {:?}", code.tokens[i].type_);
            }

            let curr = &code.tokens[i];

            match &curr.type_ {
                TokenType::String(value) => self.st_stack.push(Object::String(Rc::clone(value))),
                TokenType::Int(value)        => self.st_stack.push(Object::Int(*value)),
                TokenType::Float(value)      => self.st_stack.push(Object::Float(*value)),
                TokenType::Code(code)       => self.st_stack.push(Object::Code(code.clone())),
                TokenType::EOF => break,

                TokenType::At => {
                    self.checked_pop_nd(curr)?;
                }
                TokenType::Hash => {
                    let tmp = self.st_stack.clone();
                    self.st_stack = self.nd_stack.clone();
                    self.nd_stack = tmp;
                }
                TokenType::Comma => {
                    let popped = self.checked_pop(curr)?;
                    self.nd_stack.push(popped);
                }
                TokenType::Semicolon => {
                    let popped = self.checked_pop_nd(curr)?;
                    self.st_stack.push(popped);
                }
                TokenType::Dot => {
                    if let Some(peeked) = self.st_stack.last() {
                        self.st_stack.push(peeked.clone());
                    } else {
                        token_runtime_error!(curr, "Peeked empty primary stack");
                        return Err(());
                    }
                }
                TokenType::Print => {
                    self.checked_pop(curr)?.print();
                }
                TokenType::Bang => {
                    let popped = self.checked_pop(curr)?;
                    self.st_stack.push(Object::Int(popped.is_truthy() as i64));
                }
                TokenType::Tilde => {
                    let popped = self.checked_pop(curr)?;
                    match popped {
                        Object::Int(x)   => self.st_stack.push(Object::Int(!x)),
                        Object::Float(x) => self.st_stack.push(Object::Int(x as i64)),
                        Object::String(x) => {
                            let source = x.to_string();
                            let mut scanner = Scanner::new(&source);
                            scanner.scan_tokens();

                            if scanner.had_error {
                                return Err(());
                            }

                            self.st_stack.push(Object::Code(Code::new(scanner.tokens, scanner.labels)));
                        }
                        _ => {
                            token_runtime_error!(
                                curr, 
                                format!("Cannot perform this operation on type {:?}", popped).as_ref()
                            );
                            return Err(());
                        }
                    }
                }

                TokenType::Plus => {
                    let b = self.checked_pop(curr)?;
                    let a = self.checked_pop(curr)?;

                    match &a {
                        Object::Int(x) => {
                            match b {
                                Object::Int(y)        => self.st_stack.push(Object::Int(x.wrapping_add(y))),
                                Object::Float(y)      => self.st_stack.push(Object::Float(*x as f64 + y)),
                                Object::String(y) => self.st_stack.push(Object::String(format!("{}{}", x, y).into())),
                                _ => {
                                    token_runtime_error!(
                                        curr,
                                        format!("Cannot perform this operation on types {:?} and {:?}", a, b).as_ref()
                                    );
                                    return Err(());
                                }
                            }
                        }
                        Object::Float(x) => {
                            match b {
                                Object::Int(y)        => self.st_stack.push(Object::Float(x + y as f64)),
                                Object::Float(y)      => self.st_stack.push(Object::Float(x + y)),
                                Object::String(y) => self.st_stack.push(Object::String(format!("{}{}", x, y).into())),
                                _ => {
                                    token_runtime_error!(
                                        curr, 
                                        format!("Cannot perform this operation on types {:?} and {:?}", a, b).as_ref()
                                    );
                                    return Err(());
                                }
                            }
                        }
                        Object::String(x) => {
                            match b {
                                Object::Int(y)        => self.st_stack.push(Object::String(format!("{}{}", x, y).into())),
                                Object::Float(y)      => self.st_stack.push(Object::String(format!("{}{}", x, y).into())),
                                Object::String(y) => self.st_stack.push(Object::String(format!("{}{}", x, y).into())),
                                _ => {
                                    token_runtime_error!(
                                        curr, 
                                        format!("Cannot perform this operation on types {:?} and {:?}", a, b).as_ref()
                                    );
                                    return Err(());
                                }
                            }
                        }
                        Object::Code(x) => {
                            match b {
                                Object::Code(mut y) => {
                                    let mut result = x.clone();
                                    result.tokens.pop().expect("Malformed code"); // pops EOF
                                    result.tokens.append(&mut y.tokens);
                                    y.labels = y.labels.into_iter().map(|(k, v)| (k, v + x.tokens.len())).collect();
                                    
                                    for (label, index) in y.labels {
                                        if x.labels.contains_key(&label) {
                                            token_runtime_error!(
                                                curr, 
                                                format!("Label \"{}\" conflicts between concatenated code objects", label).as_ref()
                                            );
                                            return Err(());
                                        }

                                        result.labels.insert(label, index + x.tokens.len());
                                    }

                                    self.st_stack.push(Object::Code(result));
                                }
                                _ => {
                                    token_runtime_error!(
                                        curr, 
                                        format!("Cannot perform this operation on types {:?} and {:?}", a, b).as_ref()
                                    );
                                    return Err(());
                                }
                            }
                        }
                    }
                }

                TokenType::Minus   => simple_binary!(self, curr, -, wrapping_sub),
                TokenType::Slash   => simple_binary!(self, curr, /, wrapping_div),
                TokenType::Star    => simple_binary!(self, curr, *, wrapping_mul),
                TokenType::Mod     => simple_binary!(self, curr, %, rem),
                TokenType::And     => bitwise_binary!(self, curr, &),
                TokenType::Or      => bitwise_binary!(self, curr, |),
                TokenType::Equal   => cmp_binary!(self, curr, ==),
                TokenType::Greater => cmp_binary!(self, curr, >),
                TokenType::Less    => cmp_binary!(self, curr, <),
                
                TokenType::Jump => {
                    let jump_to = self.checked_pop(curr)?;
                    match jump_to {
                        Object::Int(amount)       => i = (i as i64 + amount       ) as usize % code.tokens.len(),
                        Object::Float(amount)     => i = (i as i64 + amount as i64) as usize % code.tokens.len(),
                        Object::String(label) => {
                            if let Some(index) = code.labels.get(&label) {
                                i = *index;
                            } else {
                                token_runtime_error!(
                                    curr, 
                                    format!("Unknown label \"{}\"", label).as_ref()
                                );
                                return Err(());
                            }
                        }
                        Object::Code(code) => ctx.run(|ctx| self.execute(&code, ctx)).await?
                    }
                    
                    continue;
                }

                TokenType::Question => {
                    let jump_to = self.checked_pop(curr)?;
                    let condition = self.checked_pop(curr)?;

                    match jump_to {
                        Object::Int(amount) => {
                            if condition.is_truthy() {
                                i = (i as i64 + amount) as usize % code.tokens.len();
                                continue;
                            }
                        }
                        Object::Float(amount) => {
                            if condition.is_truthy() {
                                i = (i as i64 + amount as i64) as usize % code.tokens.len();
                                continue;
                            }
                        }
                        Object::String(label) => {
                            if let Some(index) = code.labels.get(&label) {
                                if condition.is_truthy() {
                                    i = *index;
                                    continue;
                                }
                            } else {
                                token_runtime_error!(
                                    curr, 
                                    format!("Unknown label \"{}\"", label).as_ref()
                                );
                                return Err(());
                            }
                        }
                        Object::Code(code) => ctx.run(|ctx| self.execute(&code, ctx)).await?
                    }
                }

                TokenType::Colon => {
                    let name_obj = self.checked_pop(curr)?;
                    let code_obj = self.checked_pop(curr)?;

                    if let Object::String(name) = name_obj {
                        if let Object::Code(code) = code_obj {
                            self.functions.insert(name, code);
                        } else {
                            token_runtime_error!(
                                curr, 
                                format!("Expecting code object as function body (got {:?})", code_obj).as_ref()
                            );
                            return Err(());
                        }
                    } else {
                        token_runtime_error!(
                            curr, 
                            format!("Expecting string as function name (got {:?})", name_obj).as_ref()
                        );
                        return Err(());
                    }
                }

                TokenType::Identifier => {
                    if let Some(function) = self.functions.get(&curr.lexeme) {
                        let code = function.clone();
                        ctx.run(|ctx| self.execute(&code, ctx)).await?;
                    } else {
                        token_runtime_error!(curr, "Undefined function");
                        return Err(());
                    }
                }
            }
            
            i += 1;
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    if let Some(filename) = std::env::args().nth(1) {
        let source = std::fs::read_to_string(&filename)?;
        let mut scanner = Scanner::new(&source);
        scanner.scan_tokens();
        let mut interpreter = Interpreter::new();
        let code = Code::new(scanner.tokens, scanner.labels);
        let _ = reblessive::Stack::new().enter(|ctx| interpreter.execute(&code, ctx)).finish();
        Ok(())
    } else {
        Err(Error::other("No file provided"))
    }    
}
