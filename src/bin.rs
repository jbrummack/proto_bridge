use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
};

fn main() {
    let interface = include_str!("messaging.interface");
    //let interface = include_str!("messaging.proto");
    //println!("{interface}");
    let classes = tokenize(interface).expect("tokenize err");
    let mut stm = StateMachine::new();
    stm.parse_token_stream(classes);
    //println!("{:?}", classes);
}

enum BridgeError {
    Tokenize(TokenizeError),
    Codegen,
}

struct Interface {
    class_definitions: std::collections::HashMap<String, Option<Class>>,
}

struct Class {
    name: String,
    members: std::collections::HashMap<String, Function>,
}

#[derive(Debug)]
struct Function {
    is_async: bool,
    input: Vec<FunctionParam>,
    output: Vec<String>,
}

#[derive(Debug, Clone)]
struct FunctionParam {
    param_name: String,
    param_type: String,
}
impl std::fmt::Display for FunctionParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.param_name, self.param_type)
    }
}
use convert_case::{Case, Casing};
impl Function {
    fn codegen_rs(&self, name: &str) -> () {
        let is_async = if self.is_async { "async " } else { "" };
        let strfied_params: Vec<String> = self
            .input
            .clone()
            .into_iter()
            .map(|prm| format!("{prm}"))
            .collect();
        let strfied_params_export: Vec<String> = self
            .input
            .clone()
            .into_iter()
            .map(|prm| format!("{}: MessageFatPointer", prm.param_name))
            .collect();
        println!(
            "{is_async}fn {name}({}) -> ({}) {{}}",
            strfied_params.join(", "),
            self.output.join(", ")
        );
        println!(
            "pub extern \"C\" fn _internal_{name}({}) -> ({}) {{}}",
            strfied_params_export.join(", "),
            self.output.join(", ")
        );
    }
}

struct TokenPos {
    line_number: usize,
    position: usize,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
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
    Async,
    ParenClose,
    Semicolon,
    Colon,
    EqSign,
    Enum,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Whitespace => {
                write!(f, " ")
            }
            Token::Class => {
                write!(f, "class")
            }
            Token::Message => {
                write!(f, "message")
            }
            Token::Word(word) => {
                write!(f, "{word}")
            }
            Token::CurlyOpen => {
                write!(f, "{{")
            }
            Token::CurlyClose => {
                write!(f, "}}")
            }
            Token::Comma => {
                write!(f, ",")
            }
            Token::Dot => {
                write!(f, ".")
            }
            Token::Slash => {
                write!(f, "/")
            }
            Token::ParenOpen => {
                write!(f, "(")
            }
            Token::GenericOpen => {
                write!(f, "<")
            }
            Token::GenericClose => {
                write!(f, ">")
            }
            Token::Arrow => {
                write!(f, "->")
            }
            Token::Async => {
                write!(f, "async")
            }
            Token::ParenClose => {
                write!(f, ")")
            }
            Token::Semicolon => {
                write!(f, ";\n")
            }
            Token::Colon => {
                write!(f, ":")
            }
            Token::EqSign => {
                write!(f, "=")
            }
            Token::Enum => {
                write!(f, "enum")
            }
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
enum StateMachine {
    State0,
    ProtoBuf,
    ParsingClass,
    ParsingClassFunctions,
    ParsingFunctionName,
    ParsingFunctionArgs,
    ParsingFunctionArgsWord,
    ParsingFunctionArgsName,
    ParsingFunctionArgsType,
    ParsingAsync,
    ParsingFunctionResult,
    ParsingFunctionResultWord,
    ParsingFunctionResultSemicolon,
}

impl StateMachine {
    fn new() -> Self {
        StateMachine::State0
    }
    fn parse_token_stream(&mut self, ts: Vec<Token>) {
        type Sm = StateMachine;
        type Tk = Token;
        //let mut token = Token::Class;
        let mut state = StateMachine::new();
        let mut current_class: String = String::new();
        let mut current_function_name: String = String::new();
        let mut current_param: FunctionParam = FunctionParam {
            param_name: String::new(),
            param_type: String::new(),
        };
        let mut current_function_args: Vec<FunctionParam> = Vec::new();
        let mut current_function_results: Vec<String> = Vec::new();
        let mut protobuf_depth = 0;
        let mut is_async = false;
        for token in ts {
            state = match (&state, &token) {
                (ignore, Tk::Whitespace) => ignore.clone(),
                (Sm::State0, Tk::Class) => Sm::ParsingClass,

                (Sm::State0, Tk::Message) => {
                    print!("message");
                    Sm::ProtoBuf
                }
                (Sm::State0, _) => Sm::State0,

                (Sm::ProtoBuf, Tk::CurlyOpen) => {
                    protobuf_depth += 1;
                    print!("{{\n");
                    Sm::ProtoBuf
                }
                (Sm::ProtoBuf, Tk::CurlyClose) => {
                    protobuf_depth += -1;
                    print!("}}\n");
                    if protobuf_depth == 0 {
                        Sm::State0
                    } else {
                        Sm::ProtoBuf
                    }
                }
                (Sm::ProtoBuf, any) => {
                    print!(" {any}");
                    Sm::ProtoBuf
                }
                (Sm::ParsingClass, Tk::Word(class_name)) => {
                    current_class = class_name.to_owned();
                    Sm::ParsingClassFunctions
                }
                (Sm::ParsingClassFunctions, Tk::CurlyOpen) => Sm::ParsingFunctionName,
                (Sm::ParsingFunctionName, Tk::Word(name)) => {
                    current_function_name = name.to_owned();
                    Sm::ParsingFunctionArgs
                }
                (Sm::ParsingFunctionName, Tk::CurlyClose) => Sm::State0,
                (Sm::ParsingFunctionArgs, Tk::ParenOpen) => Sm::ParsingFunctionArgsWord,

                (Sm::ParsingFunctionArgsWord, Tk::ParenClose) => Sm::ParsingAsync,
                (Sm::ParsingFunctionArgsWord, Tk::Word(arg)) => {
                    //current_function_args.push(arg.to_owned());
                    current_param.param_name = arg.to_owned();
                    Sm::ParsingFunctionArgsName
                }
                (Sm::ParsingFunctionArgsName, Tk::Colon) => {
                    //current_function_args.push(arg.to_owned());
                    //current_param.param_name = arg.to_owned();
                    Sm::ParsingFunctionArgsType
                }
                (Sm::ParsingFunctionArgsType, Tk::Word(arg)) => {
                    //current_function_args.push(arg.to_owned());
                    current_param.param_type = arg.to_owned();
                    current_function_args.push(current_param.clone());
                    Sm::ParsingFunctionArgsWord
                }
                (Sm::ParsingFunctionArgsWord, Tk::Comma) => Sm::ParsingFunctionArgsWord,

                (Sm::ParsingAsync, Tk::Async) => {
                    is_async = true;
                    Sm::ParsingAsync
                }
                (Sm::ParsingAsync, Tk::Arrow) => Sm::ParsingFunctionResult,
                (Sm::ParsingFunctionResult, Tk::ParenOpen) => Sm::ParsingFunctionResultWord,

                (Sm::ParsingFunctionResultWord, Tk::ParenClose) => {
                    Sm::ParsingFunctionResultSemicolon
                }
                (Sm::ParsingFunctionResultWord, Tk::Word(res)) => {
                    current_function_results.push(res.to_owned());
                    Sm::ParsingFunctionResultWord
                }
                (Sm::ParsingFunctionResultWord, Tk::Comma) => Sm::ParsingFunctionResultWord,

                (Sm::ParsingFunctionResultSemicolon, Tk::Semicolon) => {
                    let func = Function {
                        is_async,
                        input: current_function_args.clone(),
                        output: current_function_results.clone(),
                    };

                    func.codegen_rs(&current_function_name);

                    is_async = false;
                    current_function_args = Vec::new();
                    current_function_results = Vec::new();
                    Sm::ParsingFunctionName
                }
                (Sm::ParsingClass, Tk::CurlyClose) => Sm::State0,

                //(Sm::ParsingClassFunctions, Tk::CurlyClose) => Sm::State0,
                _ => {
                    eprintln!("Unexpected Token: {:?} at 0:0", token);
                    break;
                }
            };
            //println!("{:?}:{:?}", token, state);
        }
    }
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
        //println!("{wordbuf}");
        if wordbuf == "class" && !char.is_alphanumeric() {
            res.push(Token::Class);
            wordbuf = String::new();
        }
        if wordbuf == "message" && !char.is_alphanumeric() {
            res.push(Token::Message);
            wordbuf = String::new();
        }
        if wordbuf == "async" && !char.is_alphanumeric() {
            res.push(Token::Async);
            wordbuf = String::new();
        }
        if wordbuf == "enum" && !char.is_alphanumeric() {
            res.push(Token::Enum);
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
                        //println!("Arrow");
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
