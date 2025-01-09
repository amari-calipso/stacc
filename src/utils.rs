use std::{cmp::max, rc::Rc};

pub fn substring(string: &String, a: usize, b: usize) -> String {
    string.chars().skip(a).take(b - a).collect()
}

pub fn report(source: &Rc<str>, msg: &str, type_: &str, pos: usize, len: usize, line: usize) {
    let lines = source.lines().collect::<Vec<&str>>();
    let iter_range = {
        if lines.len() < 5 {
            0..lines.len()
        } else {
            if line <= 2 {
                0..5
            } else if line >= lines.len() - 3 {
                (lines.len() - 5)..lines.len()
            } else {
                (line - 2)..(line + 3)
            }
        }
    };

    let linelen = max((iter_range.end as f64).log10().ceil() as usize, 1);

    println!("{} (line {}, pos {}): {}", type_, line + 1, pos, msg);

    for l in iter_range {
        println!("{:linelen$} | {}", l + 1, lines[l].trim_end());

        if l == line {
            println!("{} | {}{}", " ".repeat(linelen), " ".repeat(pos), "^".repeat(len));
        } 
    }
}

pub fn error(source: &Rc<str>, msg: &str, pos: usize, len: usize, line: usize) {
    report(source, msg, "error", pos, len, line);
}

pub fn runtime_error(source: &Rc<str>, msg: &str, pos: usize, len: usize, line: usize) {
    report(source, msg, "runtime error", pos, len, line);
}

#[macro_export]
macro_rules! token_error {
    ($token: expr, $msg: expr) => {
        crate::utils::error(&$token.source, $msg, $token.pos, $token.end - $token.pos, $token.line);
    };
}

#[macro_export]
macro_rules! token_runtime_error {
    ($token: expr, $msg: expr) => {
        crate::utils::runtime_error(&$token.source, $msg, $token.pos, $token.end - $token.pos, $token.line);
    };
}

pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') ||
    (c >= 'A' && c <= 'Z') ||
    c == '_'
}

pub fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}