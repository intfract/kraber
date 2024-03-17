use std::{any::Any, collections::HashMap, fmt};

mod functions;
use functions::add;

#[derive(Clone)]
struct Token {
    value: String,
    category: String,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.category, self.value)
    }
}

struct TokenFactory {
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

impl TokenFactory {
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
                    tokens.push(Token { value: word, category: "KEY".to_string() });
                } else if word == "false" || word == "true" {
                    tokens.push(Token { value: word, category: "BLN".to_string() });
                } else if ["int".to_string()].contains(&word) {
                    tokens.push(Token { value: word, category: "TYP".to_string() });
                } else {
                    tokens.push(Token { value: word, category: "REF".to_string() });
                }
            } else if self.commutators.contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: "COM".to_string() })
            } else if self.digits.contains(self.character) || self.character == '+' || self.character == '-' {
                let mut number = self.character.to_string();
                self.step();
                number.push_str(&self.get_number());
                if number.contains('.') {
                    tokens.push(Token { value: number, category: "FLT".to_string() })
                } else if number.contains('+') || number.contains('-') {
                    tokens.push(Token { value: number, category: "INT".to_string() })
                } else {
                    tokens.push(Token { value: number, category: "WHL".to_string() })
                }
            } else if self.brackets.contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: "BRK".to_string() })
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

#[derive(Debug, PartialEq, Clone)]
struct Integer {
    value: i32,
}

#[derive(Debug, PartialEq, Clone)]
struct Whole {
    value: u32,
}

#[derive(Debug, PartialEq, Clone)]
struct Float {
    value: f64,
}

struct Declare {}

#[derive(Debug, PartialEq, Clone)]
enum Types {
    Whole(Whole),
    Integer(Integer),
    Float(Float),
    Declare,
    String(String),
}

#[derive(Debug)]
struct Tree {
    root: Node,
}

#[derive(Debug, Clone)]
struct Node {
    id: usize,
    data: Types,
    nodes: Vec<Node>,
}

impl Tree {
    fn new() -> Self {
        Tree {
            root: Node {
                id: 0,
                data: Types::Whole(Whole { value: 0 }),
                nodes: Vec::new(),
            },
        }
    }

    fn get_scope(&mut self, scope: Vec<usize>) -> &mut Node {
        self.root.get_scope(scope)
    }
}

impl Node {
    fn insert(&mut self, item: &Types) -> &mut Node {
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
            match self.token.category.as_str() {
                "KEY" => {
                    if self.token.value == "declare" {
                        self.step();
                        if self.token.category != "REF" {
                            panic!("expected REF");
                        }
                        let node = ast.get_scope(scope.clone()).insert(&Types::Declare);
                        self.step();
                        if self.token.value == "as" {
                            self.step();
                            if self.token.category != "TYP" {
                                panic!("expected TYP");
                            }
                            node.insert(&Types::String(self.token.value.clone()));
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

fn create_token_factory(code: String) -> TokenFactory {
    let character = code.chars().nth(0).expect("blank code string detected");
    let mut comfns: HashMap<char, fn(Vec<f64>) -> f64> = HashMap::new();
    comfns.insert('+', add);
    TokenFactory {
        code,
        index: 0,
        character,
        end: false,
        letters: "abcdefghijklmnopqrstuvwxyz_".to_string(),
        digits: "0123456789".to_string(),
        commutators: "+*=".to_string(),
        comfns,
        brackets: "()[]{}".to_string(),
        keywords: vec!["declare".to_string(), "as".to_string(), "set".to_string().to_string(), "to".to_string()],
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
    let code = "declare x as int\nset x to f(1)".to_string();
    let mut token_factory = create_token_factory(code);
    let tokens = token_factory.get_tokens();
    for token in &tokens {
        println!("{token}");
    }
    let mut parser = create_parser(tokens);
    let ast = parser.parse();
    println!("{ast:?}");
}