use std::{
    borrow::Borrow,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};

fn main() {
    let interface = include_str!("messaging.interface");
    //let interface = include_str!("messaging.proto");
    //println!("{interface}");
    let classes = tokenize(interface);
    println!("{:?}", classes);
}

struct Interface {
    class_definitions: std::collections::HashMap<String, Option<Class>>,
}

struct Class {
    name: String,
    members: std::collections::HashMap<String, Function>,
}

struct Function {
    is_async: bool,
    input: Vec<String>,
    output: Vec<String>,
}

struct TokenPos {
    line_number: usize,
    position: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Whitespace,
    Class,
    Message,
    Word(String),
    CurlyOpen,
    CurlyClose,
    Comma,
    Dot,
    Slash,
    ParenOpen,
    GenericOpen,
    GenericClose,
    Arrow,
    ParenClose,
    Semicolon,
    Colon,
    EqSign,
}

enum Lex {
    ClassToken,
    ClassName(String),
    ClassStart,
    FunctionName(String),
    FunctionParam(Vec<String>),
    Arrow,
    FunctionResType(Vec<String>),
    EndFunction,
    ClassStop,
    Whitespace,
}
/*fn preprocess(input: &str) -> String {
    let preprocess_input = input
        .replace(";", " ; ")
        .replace("(", " ( ")
        .replace(")", " ) ")
        .replace("{", " {} ")
        .replace("}", " } ")
        .replace("->", " -> ");
    return preprocess_input;
}*/
#[derive(Debug)]
enum TokenizeError {}

fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    let mut ct_line = 0;
    let mut ct_pos = 0;
    let mut res = Vec::new();
    let mut pos: Vec<TokenPos> = Vec::new();
    let mut wordbuf = String::new();
    let mut is_comment = false;
    for char in input.chars() {
        ct_pos += 1;
        if char == '\n' {
            ct_line += 1;
            ct_pos = 0;
        }
        let current_pos = TokenPos {
            line_number: ct_line,
            position: ct_pos,
        };
        println!("{wordbuf}");
        if wordbuf == "class" && !char.is_alphanumeric() {
            res.push(Token::Class);
            wordbuf = String::new();
        }
        if wordbuf == "message" && !char.is_alphanumeric() {
            res.push(Token::Message);
            wordbuf = String::new();
        }
        match char {
            '(' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::ParenOpen);
            }
            '=' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::EqSign);
            }
            '.' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::Dot);
            }
            ':' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::Colon);
            }
            ')' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::ParenClose);
            }
            '{' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::CurlyOpen);
            }
            '}' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::CurlyClose);
            }
            ';' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::Semicolon);
            }
            ',' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::Comma);
            }
            '<' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::GenericOpen);
            }
            '>' => {
                let last = wordbuf.pop();
                if let Some(is_hyphen) = last {
                    if is_hyphen == '-' {
                        println!("Arrow");
                        if !wordbuf.is_empty() {
                            res.push(Token::Word(wordbuf));
                            wordbuf = String::new();
                        }
                        res.push(Token::Arrow);
                    } else {
                        wordbuf.push(is_hyphen);
                        res.push(Token::Word(wordbuf));
                        wordbuf = String::new();
                        res.push(Token::GenericClose);
                    }
                }
            }
            _ => {
                let last_token = res.last().cloned();
                if char.is_whitespace() && last_token != Some(Token::Whitespace) {
                    if !wordbuf.is_empty() {
                        res.push(Token::Word(wordbuf));
                        wordbuf = String::new();
                    }
                    res.push(Token::Whitespace);
                } else if char.is_whitespace() {
                    if !wordbuf.is_empty() {
                        res.push(Token::Word(wordbuf));
                        wordbuf = String::new();
                    }
                }

                if char.is_alphanumeric() || char == '-' || char == '_' {
                    wordbuf.push(char);
                }
            }
        }
    }

    Ok(res)
}
