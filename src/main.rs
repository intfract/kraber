extern crate strum;
#[macro_use] extern crate strum_macros;

use std::{any::Any, collections::HashMap, fmt, str::FromStr};

mod functions;
use functions::add;

struct Main {}

#[derive(Debug, PartialEq, Clone, EnumString, Display)]
enum Meta {
    REF,
    TYP,
    WHL,
    INT,
    FLT,
    KEY,
    BLN,
    COM,
    BRK,
}

#[derive(Debug, PartialEq, Clone, EnumString, Display)]
enum Data {
    Main,
    Declare,
    Assign,
    Identifier(String),
    Type(String),
    Whole(usize),
    Integer(isize),
    Float(f64),
}

#[derive(Clone)]
struct Token {
    value: String,
    category: Meta,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.category, self.value)
    }
}

struct Lexer {
    code: String,
    index: usize,
    character: char,
    end: bool,
    letters: String,
    digits: String,
    commutators: String,
    comfns: HashMap<char, fn(Vec<f64>) -> f64>,
    brackets: String,
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

    fn get_tokens(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while !self.end {
            if self.letters.contains(self.character) {
                let word = self.get_word();
                if self.keywords.contains(&word) {
                    tokens.push(Token { value: word, category: Meta::KEY });
                } else if word == "false" || word == "true" {
                    tokens.push(Token { value: word, category: Meta::BLN });
                } else if ["whole".to_string(), "integer".to_string(), "float".to_string()].contains(&word) {
                    tokens.push(Token { value: word, category: Meta::TYP });
                } else {
                    tokens.push(Token { value: word, category: Meta::REF });
                }
            } else if self.commutators.contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: Meta::COM })
            } else if self.digits.contains(self.character) || self.character == '+' || self.character == '-' {
                let mut number = self.character.to_string();
                self.step();
                number.push_str(&self.get_number());
                if number.contains('.') {
                    tokens.push(Token { value: number, category: Meta::FLT })
                } else if number.contains('+') || number.contains('-') {
                    tokens.push(Token { value: number, category: Meta::INT })
                } else {
                    tokens.push(Token { value: number, category: Meta::WHL })
                }
            } else if self.brackets.contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: Meta::BRK })
            }
            self.step();
        }
        tokens
    }
}

struct Variable<'a> {
    value: &'a dyn Any,
    category: String,
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
                        node.insert(&Data::Identifier(self.token.value.clone()));
                        self.step();
                        if self.token.value == "as" {
                            self.step();
                            if self.token.category != Meta::TYP {
                                panic!("expected TYP");
                            }
                            node.insert(&Data::Type(self.token.value.clone()));
                        }
                    }
                    if self.token.value == "set" {
                        self.step();
                        if self.token.category != Meta::REF {
                            panic!("expected REF");
                        }
                        let node = ast.get_scope(scope.clone()).insert(&Data::Assign);
                        self.step();
                        if self.token.value == "to" {
                            self.step();
                            match self.token.category {
                                Meta::WHL => {
                                    node.insert(&Data::Whole(self.token.value.clone().parse().unwrap()));
                                },
                                Meta::INT => {
                                    node.insert(&Data::Integer(self.token.value.clone().parse().unwrap()));
                                },
                                Meta::FLT => {
                                    node.insert(&Data::Float(self.token.value.clone().parse().unwrap()));
                                },
                                Meta::REF => {
                                    node.insert(&Data::Identifier(self.token.value.clone()));
                                }
                                _ => {
                                    panic!("expected expression");
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
            self.step();
        }
        ast
    }
}

fn create_lexer(code: String) -> Lexer {
    let character = code.chars().nth(0).expect("blank code string detected");
    let mut comfns: HashMap<char, fn(Vec<f64>) -> f64> = HashMap::new();
    comfns.insert('+', add);
    Lexer {
        code,
        index: 0,
        character,
        end: false,
        letters: "abcdefghijklmnopqrstuvwxyz_".to_string(),
        digits: "0123456789".to_string(),
        commutators: "+*=".to_string(),
        comfns,
        brackets: "()[]{}".to_string(),
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
    let code = "declare x as integer\ndeclare y as integer\nset x to 1\nset y to x".to_string();
    let mut lexer = create_lexer(code);
    let tokens = lexer.get_tokens();
    for token in &tokens {
        println!("{token}");
    }
    let mut parser = create_parser(tokens);
    let ast = parser.parse();
    println!("{ast:?}");
}