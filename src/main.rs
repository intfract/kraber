use std::{collections::HashMap, fmt,};

mod functions;
use functions::add;

#[derive(Debug, PartialEq, Clone)]
enum Meta {
    REF,
    TYP,
    WHL,
    INT,
    FLT,
    KEY,
    BLN,
    FUN,
    PAR,
    TXT,
}

#[derive(Debug, PartialEq, Clone)]
enum Data {
    Main,
    Declare,
    Assign,
    Identifier { name: String },
    Type { name: String },
    Null,
    Whole { value: usize },
    Integer{ value: isize},
    Float { value: f64 },
    Text { value: String },
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
struct Token {
    value: String,
    category: Meta,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {}", self.category, self.value)
    }
}

struct Lexer {
    code: String,
    index: usize,
    character: char,
    end: bool,
    letters: String,
    digits: String,
    keywords: Vec<String>,
}

impl Lexer {
    fn step(&mut self) {
        self.index += 1;
        if self.index < self.code.len() {
            self.character = self.code.chars().nth(self.index).expect("index is out of range");
        } else {
            self.end = true;
        }
    }

    fn get_word(&mut self) -> String {
        let mut word: String = "".to_string();
        while !self.end && self.letters.contains(self.character) {
            word.push(self.character);
            self.step();
        }
        word
    }

    fn get_number(&mut self) -> String {
        let mut number: String = "".to_string();
        while !self.end && (self.digits.contains(self.character) || self.character == '.') {
            number.push(self.character);
            self.step();
        }
        number
    }

    fn get_string(&mut self) -> String {
        let mut text: String = "".to_string();
        while !self.end && self.character != '"' {
            text.push(self.character);
            self.step();
        }
        text
    }

    fn get_tokens(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while !self.end {
            if self.letters.contains(self.character) {
                let word = self.get_word();
                if self.keywords.contains(&word) {
                    tokens.push(Token { value: word, category: Meta::KEY });
                } else if word == "false" || word == "true" {
                    tokens.push(Token { value: word, category: Meta::BLN });
                } else if ["whole".to_string(), "integer".to_string(), "float".to_string(), "text".to_string()].contains(&word) {
                    tokens.push(Token { value: word, category: Meta::TYP });
                } else if word == "fun" {
                    tokens.push(Token { value: word, category: Meta::FUN });
                } else {
                    tokens.push(Token { value: word, category: Meta::REF });
                }
            } else if self.digits.contains(self.character) || self.character == '+' || self.character == '-' {
                let mut number = self.character.to_string();
                self.step();
                number.push_str(&self.get_number());
                if number.contains('.') {
                    tokens.push(Token { value: number, category: Meta::FLT });
                } else if number.contains('+') || number.contains('-') {
                    tokens.push(Token { value: number, category: Meta::INT });
                } else {
                    tokens.push(Token { value: number, category: Meta::WHL });
                }
            } else if "()".contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: Meta::PAR })
            } else if self.character == '"' {
                self.step();
                let text = self.get_string();
                tokens.push(Token { value: text, category: Meta::TXT });
            }
            self.step();
        }
        tokens
    }
}

#[derive(Debug)]
struct Variable {
    value: Data,
    data_type: Data,
}

#[derive(Debug, Clone)]
struct Tree {
    root: Node,
}

#[derive(Debug, Clone)]
struct Node {
    id: usize,
    data: Data,
    nodes: Vec<Node>,
}

impl Tree {
    fn new() -> Self {
        Tree {
            root: Node {
                id: 0,
                data: Data::Main,
                nodes: Vec::new(),
            },
        }
    }

    fn get_scope(&mut self, scope: Vec<usize>) -> &mut Node {
        self.root.get_scope(scope)
    }
}

impl Node {
    fn insert(&mut self, item: &Data) -> &mut Node {
        let mut node = self;

        let next_node_idx = Node::get_child_idx(&node.nodes, node.nodes.len());
        
        node = match next_node_idx {
            Some(x) => {
                node.nodes[x].insert(item)
            },
            None => {
                let new_node = Node {
                    id: node.nodes.len(),
                    data: item.to_owned(),
                    nodes: Vec::new(),
                };
                node.nodes.push(new_node);
                node.nodes.last_mut().unwrap()
            }
        };
        node
    }

    fn get_child_idx<'a>(v: &Vec<Node>, id: usize) -> Option<usize> {
        v.iter()
            .enumerate()
            .find(|(_, n)| n.id == id)
            .map(|(i, _)| i)
    }

    fn get_scope(&mut self, mut scope: Vec<usize>) -> &mut Node {
        let mut node = self;
        if scope.len() > 0 {
            node = &mut node.nodes[scope[0]];
            scope.pop();
            return node.get_scope(scope);
        }
        node
    }
}

struct Parser {
    index: usize,
    tokens: Vec<Token>,
    token: Token,
    end: bool,
}

impl Parser {
    fn step(&mut self) {
        self.index += 1;
        if self.index < self.tokens.len() {
            self.token = self.tokens[self.index].clone();
        } else {
            self.end = true;
        }
    }

    fn parse(&mut self) -> Tree {
        let mut ast = Tree::new();
        let mut scope: Vec<usize> = [].to_vec();
        while !self.end {
            match self.token.category {
                Meta::KEY => {
                    if self.token.value == "declare" {
                        self.step();
                        if self.token.category != Meta::REF {
                            panic!("expected REF");
                        }
                        let node = ast.get_scope(scope.clone()).insert(&Data::Declare);
                        node.insert(&Data::Identifier { name: self.token.value.clone() });
                        self.step();
                        if self.token.value == "as" {
                            self.step();
                            if self.token.category != Meta::TYP {
                                panic!("expected TYP");
                            }
                            node.insert(&Data::Type { name: self.token.value.clone() });
                        }
                    }
                    if self.token.value == "set" {
                        self.step();
                        if self.token.category != Meta::REF {
                            panic!("expected REF");
                        }
                        let node = ast.get_scope(scope.clone()).insert(&Data::Assign);
                        node.insert(&Data::Identifier { name: self.token.value.clone() });
                        self.step();
                        if self.token.value == "to" {
                            self.step();
                            match self.token.category {
                                Meta::WHL => {
                                    node.insert(&Data::Whole { value: self.token.value.clone().parse().unwrap() });
                                },
                                Meta::INT => {
                                    node.insert(&Data::Integer { value: self.token.value.clone().parse().unwrap() });
                                },
                                Meta::FLT => {
                                    node.insert(&Data::Float { value: self.token.value.clone().parse().unwrap() });
                                },
                                Meta::TXT => {
                                    node.insert(&Data::Text { value: self.token.value.clone() });
                                },
                                Meta::REF => {
                                    node.insert(&Data::Identifier { name: self.token.value.clone() });
                                }
                                _ => {
                                    panic!("expected expression");
                                }
                            }
                        }
                    }
                },
                Meta::TXT => {
                    ast.get_scope(scope.clone()).insert(&Data::Text { value: self.token.value.clone() });
                },
                Meta::REF => {
                    ast.get_scope(scope.clone()).insert(&Data::Identifier { name: self.token.value.clone() });
                },
                _ => {}
            }
            self.step();
        }
        ast
    }
}

struct Interpreter {
    tree: Tree,
    memory: HashMap<String, Variable>,
}

impl Interpreter {
    fn interpret(&mut self) -> &mut HashMap<String, Variable> {
        let current = &self.tree.root;
        for node in &current.nodes {
            match &node.data {
                Data::Declare => {
                    match &node.nodes[0].data {
                        Data::Identifier { name } => {
                            self.memory.insert(name.clone(), Variable {
                                value: Data::Null,
                                data_type: node.nodes[1].data.clone(),
                            });
                        },
                        _ => {}
                    };
                },
                Data::Assign => {
                    match &node.nodes[0].data {
                        Data::Identifier { name } => {
                            let data_type = self.memory.get(&name.clone()).unwrap().data_type.clone();
                            let value = match &node.nodes[1].data {
                                Data::Identifier { name: var_name } => {
                                    let var_data_type = self.memory.get(&var_name.clone()).unwrap().data_type.clone();
                                    if data_type != var_data_type {
                                        panic!("expected {data_type:?} but got {var_data_type:?}");
                                    }
                                    self.memory.get(var_name).unwrap().value.clone()
                                },
                                _ => {
                                    let x = node.nodes[1].data.clone().to_string().to_lowercase();
                                    let index = x.find(' ').unwrap();
                                    let var_type_name: String = x.chars().into_iter().take(index).collect();
                                    match &data_type {
                                        Data::Type { name: type_name } => {
                                            if type_name.to_string() != var_type_name {
                                                panic!("expected {type_name:?} but got {var_type_name:?}");
                                            }
                                        },
                                        _ => {}
                                    }
                                    node.nodes[1].data.clone()
                                }
                            };
                            self.memory.insert(name.clone(), Variable {
                                value,
                                data_type,
                            });
                        },
                        _ => {}
                    };
                },
                Data::Text { value } => {
                    println!("{}", value); // implicit print
                },
                Data::Identifier { name } => {
                    let x = &self.memory.get(&name.clone()).unwrap().value;
                    match x {
                        Data::Type { name } => println!("{name}"),
                        Data::Null => println!("null"),
                        Data::Whole { value } => println!("{value}"),
                        Data::Integer { value } => println!("{value}"),
                        Data::Float { value } => println!("{value}"),
                        Data::Text { value } => println!("{value}"),
                        _ => {}
                    };
                },
                _ => {}
            };
        }
        &mut self.memory
    }
}

fn create_lexer(code: String) -> Lexer {
    let character = code.chars().nth(0).expect("blank code string detected");
    Lexer {
        code,
        index: 0,
        character,
        end: false,
        letters: "abcdefghijklmnopqrstuvwxyz_".to_string(),
        digits: "0123456789".to_string(),
        keywords: vec!["declare".to_string(), "as".to_string(), "set".to_string(), "to".to_string()],
    }
}

fn create_parser(tokens: Vec<Token>) -> Parser {
    let token = tokens[0].clone();
    Parser {
        index: 0,
        tokens,
        token,
        end: false,
    }
}

fn main() {
    let code = "declare x as whole\ndeclare y as whole\ndeclare z as text\nset x to 1\nset y to x\nset z to \"hello\"\nz".to_string();
    let mut lexer = create_lexer(code);
    let tokens = lexer.get_tokens();
    for token in &tokens {
        println!("{token}");
    }
    let mut parser = create_parser(tokens);
    let ast = parser.parse();
    println!("{ast:?}");
    let mut interpreter = Interpreter {
        tree: ast,
        memory: HashMap::new(),
    };
    let memory = interpreter.interpret();
    println!("{memory:?}");
}