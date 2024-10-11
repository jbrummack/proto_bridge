use std::{collections::HashMap, hash::Hash};

fn main() {
    let interface = include_str!("messaging.interface");
    //let interface = include_str!("messaging.proto");
    //println!("{interface}");
    let classes = tokenize(interface).expect("tokenize err");
    let mut stm = InterfaceBuilder::new();
    let interface_desc = stm.parse_token_stream(classes);
    println!("{:#?}", interface_desc);

    for (interface_name, interface_impl) in interface_desc {
        for (func_name, func_impl) in interface_impl.clone() {
            println!(
                "{}",
                func_impl.codegen_rs_bridge(&interface_name, &func_name)
            );
        }
        println!("trait {interface_name} {{");
        for (func_name, func_impl) in interface_impl {
            println!("\t{}", func_impl.codegen_rs_trait(&func_name));
        }
        println!("}}\n");
    }
    //println!("{:?}", classes);
}

enum _BridgeError {
    Tokenize(TokenizeError),
    Codegen,
}

#[derive(Debug, Clone)]
struct Function {
    is_async: bool,
    input: Vec<FunctionParam>,
    output: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct FunctionParam {
    param_name: String,
    param_type: String,
}
impl std::fmt::Display for FunctionParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.param_name, self.param_type)
    }
}
impl Function {
    fn codegen_rs_bridge(self, class_name: &str, name: &str) -> String {
        let strfied_params_export: Vec<String> = self
            .input
            .clone()
            .into_iter()
            .map(|prm| format!("{}: MessageFatPointer", prm.param_name))
            .collect();
        format!(
            "pub extern \"C\" fn _internal_{class_name}_{name}({}) -> ({}) {{}}",
            strfied_params_export.join(", "),
            self.output.join(", ")
        )
    }
    fn codegen_rs_trait(&self, name: &str) -> String {
        let is_async = if self.is_async { "async " } else { "" };
        let strfied_params: Vec<String> = self
            .input
            .clone()
            .into_iter()
            .map(|prm| format!("{prm}"))
            .collect();
        format!(
            "{is_async}fn {name}({}) -> ({}) {{}}",
            strfied_params.join(", "),
            self.output.join(", ")
        )
        /**/
    }
}

/*struct TokenPos {
    line_number: usize,
    position: usize,
}*/

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

#[derive(Debug, PartialEq, Hash, Eq, Clone, Default)]
enum StateMachine {
    #[default]
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

type InterfaceClass = HashMap<String, Function>;
type Interface = HashMap<String, InterfaceClass>;

#[derive(Debug, Default)]
struct InterfaceBuilder {
    state: StateMachine,
    classes: Interface,
    current_i_class: InterfaceClass,
    current_function_name: String,
    current_param: FunctionParam,
    current_function_args: Vec<FunctionParam>,
    current_class: String,
    current_function_results: Vec<String>,
    protobuf_depth: i32,
    is_async: bool,
}
impl InterfaceBuilder {
    pub fn new() -> Self {
        InterfaceBuilder::default()
    }

    fn parse_token_stream(&mut self, ts: Vec<Token>) -> Interface {
        type Sm = StateMachine;
        type Tk = Token;

        for token in ts {
            self.state = match (&self.state, &token) {
                (ignore, Tk::Whitespace) => ignore.clone(),
                (Sm::State0, Tk::Class) => {
                    if !self.current_i_class.is_empty() {
                        let move_out = self.current_i_class.clone();
                        println!("{:?}", move_out);
                        self.classes.insert(self.current_class.to_owned(), move_out);
                        println!("reset i class");
                        self.current_i_class = InterfaceClass::new();
                        self.current_class = String::new();
                    }
                    Sm::ParsingClass
                }

                (Sm::State0, Tk::Message) => {
                    print!("message");
                    Sm::ProtoBuf
                }
                (Sm::State0, _) => Sm::State0,

                (Sm::ProtoBuf, Tk::CurlyOpen) => {
                    self.protobuf_depth += 1;
                    print!("{{\n");
                    Sm::ProtoBuf
                }
                (Sm::ProtoBuf, Tk::CurlyClose) => {
                    self.protobuf_depth += -1;
                    print!("}}\n");
                    if self.protobuf_depth == 0 {
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
                    self.current_class = dbg!(class_name.to_owned());
                    Sm::ParsingClassFunctions
                }
                (Sm::ParsingClassFunctions, Tk::CurlyOpen) => Sm::ParsingFunctionName,
                (Sm::ParsingFunctionName, Tk::Word(name)) => {
                    self.current_function_name = dbg!(name.to_owned());

                    Sm::ParsingFunctionArgs
                }
                (Sm::ParsingFunctionName, Tk::CurlyClose) => {
                    let move_out = self.current_i_class.clone();
                    println!("MOVE OUT I CLASS = {:?}", move_out);
                    self.classes.insert(self.current_class.to_owned(), move_out);
                    Sm::State0
                }
                (Sm::ParsingFunctionArgs, Tk::ParenOpen) => Sm::ParsingFunctionArgsWord,

                (Sm::ParsingFunctionArgsWord, Tk::ParenClose) => Sm::ParsingAsync,
                (Sm::ParsingFunctionArgsWord, Tk::Word(arg)) => {
                    //current_function_args.push(arg.to_owned());
                    self.current_param.param_name = arg.to_owned();
                    Sm::ParsingFunctionArgsName
                }
                (Sm::ParsingFunctionArgsName, Tk::Colon) => {
                    //current_function_args.push(arg.to_owned());
                    //current_param.param_name = arg.to_owned();
                    Sm::ParsingFunctionArgsType
                }
                (Sm::ParsingFunctionArgsType, Tk::Word(arg)) => {
                    //current_function_args.push(arg.to_owned());
                    self.current_param.param_type = arg.to_owned();
                    self.current_function_args.push(self.current_param.clone());
                    Sm::ParsingFunctionArgsWord
                }
                (Sm::ParsingFunctionArgsWord, Tk::Comma) => Sm::ParsingFunctionArgsWord,

                (Sm::ParsingAsync, Tk::Async) => {
                    self.is_async = true;
                    Sm::ParsingAsync
                }
                (Sm::ParsingAsync, Tk::Arrow) => Sm::ParsingFunctionResult,
                (Sm::ParsingFunctionResult, Tk::ParenOpen) => Sm::ParsingFunctionResultWord,

                (Sm::ParsingFunctionResultWord, Tk::ParenClose) => {
                    Sm::ParsingFunctionResultSemicolon
                }
                (Sm::ParsingFunctionResultWord, Tk::Word(res)) => {
                    self.current_function_results.push(res.to_owned());
                    Sm::ParsingFunctionResultWord
                }
                (Sm::ParsingFunctionResultWord, Tk::Comma) => Sm::ParsingFunctionResultWord,

                (Sm::ParsingFunctionResultSemicolon, Tk::Semicolon) => {
                    let func = Function {
                        is_async: self.is_async,
                        input: self.current_function_args.clone(),
                        output: self.current_function_results.clone(),
                    };
                    self.current_i_class
                        .insert(self.current_function_name.clone(), func.clone());
                    /*self.add_fn(
                        self.current_class.clone(),
                        self.current_function_name.clone(),
                        func,
                    );*/
                    //func.codegen_rs(&self.current_function_name);

                    self.is_async = false;
                    self.current_function_args = Vec::new();
                    self.current_function_results = Vec::new();
                    Sm::ParsingFunctionName
                }
                (Sm::ParsingClass, Tk::CurlyClose) => {
                    let move_out = self.current_i_class.clone();
                    println!("MOVE OUT I CLASS = {:?}", move_out);
                    self.classes.insert(self.current_class.to_owned(), move_out);
                    Sm::State0
                }

                //(Sm::ParsingClassFunctions, Tk::CurlyClose) => Sm::State0,
                _ => {
                    eprintln!("Unexpected Token: {:?} at 0:0", token);
                    break;
                }
            };
            //println!("==============\n {:?} \n==============", self);
            //println!("{:?}:{:?}", token, self.state);
        }
        self.classes.to_owned()
    }
}

#[derive(Debug)]
enum TokenizeError {}

fn tokenize(input: &str) -> Result<Vec<Token>, TokenizeError> {
    //let mut ct_line = 0;
    //let mut ct_pos = 0;
    let mut res = Vec::new();
    //let mut pos: Vec<TokenPos> = Vec::new();
    let mut wordbuf = String::new();
    //let mut is_comment = false;
    for char in input.chars() {
        /*ct_pos += 1;
        if char == '\n' {
            ct_line += 1;
            ct_pos = 0;
        }
        let _current_pos = TokenPos {
            line_number: ct_line,
            position: ct_pos,
        };*/
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
            '/' => {
                if !wordbuf.is_empty() {
                    res.push(Token::Word(wordbuf));
                    wordbuf = String::new();
                }
                res.push(Token::Slash);
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
