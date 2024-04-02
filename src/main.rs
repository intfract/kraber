use std::{collections::HashMap, fmt, env, fs, time::Instant, mem};

#[derive(Debug, PartialEq, Clone)]
enum Meta {
    REF,
    TYP,
    WHL,
    INT,
    FLT,
    KEY,
    BLN,
    TXT,
    FUN,
    PAR,
    BRK,
    BRC,
}

#[derive(Debug, PartialEq, Clone)]
enum Data {
    Main,
    Declare,
    Assign,
    While,
    Expression,
    KraberFunction {
        body: fn(Vec<Data>) -> Data,
    },
    FunctionContainer {
        params: Vec<String>,
        param_types: Vec<Data>,
        return_types: Vec<Data>,
    },
    Function {
        params: Vec<String>,
        param_types: Vec<Data>,
        return_types: Vec<Data>,
        body: Vec<Node>,
    },
    Return,
    Identifier { name: String },
    Type { name: String },
    Null,
    Whole { value: usize },
    Integer{ value: isize},
    Float { value: f64 },
    Boolean { value: bool },
    Text { value: String },
    List {
        data_types: Vec<Data>,
        value: Vec<Data>,
    },
}

fn stringify_enum(data: &Data) -> String {
    let enum_string = data.clone().to_string();
    let collection: Vec<&str> = enum_string.split(' ').collect();
    collection[0].to_string().to_lowercase()
}

fn expect_type(arg: Data) -> String {
    return match arg {
        Data::Type { name } => {
            name
        },
        _ => {
            panic!("expected Data::Type but got {:#?}", arg);
        }
    }
}

fn expect_boolean(arg: Data) -> bool {
    return match arg {
        Data::Boolean { value } => {
            value
        },
        _ => {
            panic!("expected Data::Boolean but got {:#?}", arg);
        }
    }
}

fn expect_numeric(arg: Data) -> f64 {
    return match arg {
        Data::Whole { value } => {
            value as f64
        },
        Data::Integer { value } => {
            value as f64
        },
        Data::Float { value } => {
            value
        },
        _ => {
            panic!("{:#?} is not numeric", arg);
        }
    }
}

fn expect_text(arg: Data) -> String {
    return match arg {
        Data::Text { value } => {
            value
        },
        _ => {
            panic!("expected Data::Text but got {:#?}", arg);
        }
    }
}

fn equal(args: Vec<Data>) -> Data {
    Data::Boolean {
        value: args.windows(2).all(
            |x|
            expect_numeric(x[0].clone()) == expect_numeric(x[1].clone())
        )
    }
}

fn lt(args: Vec<Data>) -> Data {
    if args.len() != 2 {
        panic!("expected 2 arguments but received {}", args.len());
    }
    Data::Boolean {
        value: expect_numeric(args[0].clone()) < expect_numeric(args[1].clone())
    }
}

fn nand(args: Vec<Data>) -> Data {
    if args.len() != 2 {
        panic!("expected 2 arguments but received {}", args.len());
    }
    Data::Boolean {
        value: !(
            expect_boolean(args[0].clone())
            &&
            expect_boolean(args[1].clone())
        )
    }
}

fn add(args: Vec<Data>) -> Data {
    let mut sum: f64 = 0.0;
    for arg in args {
        let num: f64 = expect_numeric(arg);
        sum += num;
    }
    Data::Float { value: sum }
}

fn multiply(args: Vec<Data>) -> Data {
    let mut text_string = "".to_string();
    let mut product: f64 = 1.0;
    for arg in args {
        if text_string.is_empty() {
            match &arg {
                Data::Text { value } => {
                    text_string = value.to_string();
                    continue;
                },
                _ => {}
            }
        }
        let num: f64 = expect_numeric(arg);
        product *= num;
    }
    if text_string.is_empty() {
        Data::Float { value: product }
    } else {
        Data::Text { value: text_string.repeat(product as usize) }
    }
}

fn raise(args: Vec<Data>) -> Data {
    if args.len() != 2 {
        panic!("expected 2 arguments but received {}", args.len());
    }
    Data::Float {
        value: f64::powf(
            expect_numeric(args[0].clone()),
            expect_numeric(args[1].clone())
        )
    }
}

fn floor(args: Vec<Data>) -> Data {
    if args.len() != 1 {
        panic!("expected 1 argument but received {}", args.len());
    }
    Data::Integer { value: expect_numeric(args[0].clone()) as isize }
}

fn join(args: Vec<Data>) -> Data {
    let mut text_string = "".to_string();
    for arg in args {
        let string = expect_text(arg);
        text_string += &string;
    }
    Data::Text { value: text_string }
}

fn push(args: Vec<Data>) -> Data {
    match &args[0] {
        Data::List { data_types, value } => {
            let mut x = value.clone();
            let collection: Vec<String> = data_types.iter().map(|x| expect_type(x.clone())).collect();
            if !collection.contains(&stringify_enum(&args[1])) {
                panic!("expected one of types {:#?} but got {:#?}", &data_types, &args[1]);
            }
            x.push(args[1].clone());
            Data::List {
                data_types: data_types.to_vec(),
                value: x,
            }
        },
        _ => {
            panic!("sus");
        }
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
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
        let types = [
            "whole".to_string(),
            "integer".to_string(),
            "float".to_string(),
            "boolean".to_string(),
            "text".to_string(),
            "function".to_string(),
            "list".to_string(),
        ];
        while !self.end {
            if self.letters.contains(self.character) {
                let word = self.get_word();
                if self.keywords.contains(&word) {
                    tokens.push(Token { value: word, category: Meta::KEY });
                } else if word == "false" || word == "true" {
                    tokens.push(Token { value: word, category: Meta::BLN });
                } else if types.contains(&word) {
                    tokens.push(Token { value: word, category: Meta::TYP });
                } else if word == "fun" {
                    tokens.push(Token { value: word, category: Meta::FUN });
                } else {
                    tokens.push(Token { value: word, category: Meta::REF });
                }
                continue;
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
                continue;
            } else if "()".contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: Meta::PAR })
            } else if "{}".contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: Meta::BRC })
            } else if "[]".contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: Meta::BRK })
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

#[derive(Debug, Clone)]
struct Variable {
    value: Data,
    data_type: Data,
}

#[derive(Debug, PartialEq, Clone)]
struct Tree {
    root: Node,
}

#[derive(Debug, PartialEq, Clone)]
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
            scope.remove(0);
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
            self.build_tree(&mut ast, &mut scope);
            self.step();
        }
        ast
    }

    fn build_tree(&mut self, ast: &mut Tree, scope: &mut Vec<usize>) {
        match self.token.category {
            Meta::KEY => {
                match self.token.value.as_str() {
                    "declare" => {
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
                            let sub_node = node.insert(&Data::Type { name: self.token.value.clone() });
                            if self.index < self.tokens.len() - 1 && self.tokens[self.index + 1].value == "[" {
                                self.step();
                                let mut counter: usize = 1;
                                self.step();
                                while !self.end && counter != 0 {
                                    if self.token.value == "[" {
                                        counter += 1;
                                    } else if self.token.value == "]" {
                                        counter -= 1;
                                    } else if self.token.category == Meta::TYP {
                                        sub_node.insert(&Data::Type { name: self.token.value.clone() });
                                    } else {
                                        // dynamic typing
                                    }
                                    self.step();
                                }
                                self.back();
                            }
                        }
                    },
                    "set" => {
                        self.step();
                        if self.token.category != Meta::REF {
                            panic!("expected REF");
                        }
                        let scoped_node = ast.get_scope(scope.clone());
                        scope.push(scoped_node.nodes.len());
                        let node = scoped_node.insert(&Data::Assign);
                        node.insert(&Data::Identifier { name: self.token.value.clone() });
                        self.step();
                        if self.token.value == "to" {
                            self.step();
                            // different rules for functions
                            match self.token.category {
                                Meta::FUN => {
                                    let mut params: Vec<String> = Vec::new();
                                    let mut param_types: Vec<Data> = Vec::new();
                                    if self.index < self.tokens.len() - 1 && self.tokens[self.index + 1].value == "(" {
                                        self.step();
                                        let mut counter: usize = 1;
                                        self.step();
                                        while !self.end && counter != 0 {
                                            if self.token.value == "(" {
                                                counter += 1;
                                            } else if self.token.value == ")" {
                                                counter -= 1;
                                            } else if self.token.category == Meta::REF {
                                                params.push(self.token.value.clone());
                                                self.step();
                                                if self.token.value == "as" {
                                                    self.step();
                                                    if self.token.category != Meta::TYP {
                                                        panic!("expected TYP");
                                                    }
                                                    param_types.push(Data::Type { name: self.token.value.clone() });
                                                }
                                            } else {
                                                // other feature
                                            }
                                            self.step();
                                        }
                                        let return_type = if self.token.value == "as" {
                                            self.step();
                                            if self.token.category != Meta::TYP {
                                                panic!("expected TYP");
                                            }
                                            Data::Type { name: self.token.value.clone() }
                                        } else {
                                            panic!("expected function return type");
                                        };
                                        self.step();
                                        scope.push(node.nodes.len());
                                        node.insert(&Data::FunctionContainer { params, param_types, return_types: [return_type].to_vec() });
                                        let mut counter: usize = 1;
                                        self.step();
                                        while !self.end && counter != 0 {
                                            if self.token.value == "{" {
                                                counter += 1;
                                            } else if self.token.value == "}" {
                                                counter -= 1;
                                            } else {
                                                self.build_tree(ast, scope);
                                            }
                                            self.step();
                                        }
                                        self.back();
                                        scope.pop(); // descope
                                        scope.pop(); // descope
                                    }
                                },
                                _ => {
                                    scope.pop(); // descope
                                    self.build_expression(node);
                                }
                            }
                        }
                    },
                    "while" => {
                        self.step();
                        let scoped_node = ast.get_scope(scope.clone());
                        let node_id = scoped_node.nodes.len();
                        scope.push(node_id);
                        let node = scoped_node.insert(&Data::While);
                        let sub_node = node.insert(&Data::Expression);
                        if self.index >= self.tokens.len() - 1 {
                            panic!("loop is missing a body");
                        }
                        while !self.end && !matches!(self.token.category, Meta::BRC) {
                            self.build_expression(sub_node);
                            self.step();
                        }
                        if self.token.value != "{" {
                            panic!("expected opening of loop body");
                        }
                        let mut counter: usize = 1;
                        self.step();
                        while !self.end && counter != 0 {
                            if self.token.value == "{" {
                                counter += 1;
                            } else if self.token.value == "}" {
                                counter -= 1;
                            } else {
                                self.build_tree(ast, scope);
                            }
                            self.step();
                        }
                        self.back();
                        scope.pop(); // descope
                    },
                    "return" => {
                        let node = ast.get_scope(scope.clone()).insert(&Data::Return);
                        self.step();
                        self.build_expression(node);
                    },
                    _ => {}
                }
            },
            Meta::TXT => {
                ast.get_scope(scope.clone()).insert(&Data::Text { value: self.token.value.clone() });
            },
            Meta::REF => {
                println!("{:#?}", self.token);
                self.build_expression(ast.get_scope(scope.clone()));
            },
            Meta::TYP => {
                ast.get_scope(scope.clone()).insert(&Data::Type { name: self.token.value.clone() });
            },
            _ => {}
        }
    }

    fn back(&mut self) {
        self.index -= 1;
        self.token = self.tokens[self.index].clone();
    }
    
    fn build_expression(&mut self, node: &mut Node) {
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
            Meta::BLN => {
                node.insert(&Data::Boolean { value: self.token.value.clone().parse().unwrap() });
            },
            Meta::TXT => {
                node.insert(&Data::Text { value: self.token.value.clone() });
            },
            Meta::REF => {
                let sub_node = node.insert(&Data::Identifier { name: self.token.value.clone() });
                if self.index < self.tokens.len() - 1 && self.tokens[self.index + 1].value == "(" {
                    self.step();
                    let mut counter: usize = 1;
                    self.step();
                    while !self.end && counter != 0 {
                        if self.token.value == "(" {
                            counter += 1;
                        } else if self.token.value == ")" {
                            counter -= 1;
                        } else {
                            self.build_expression(sub_node);
                        }
                        self.step();
                    }
                    self.back();
                }
            },
            _ => {
                println!("{node:?}");
                panic!("expected expression but got {:?}", self.token);
            }
        }
    }
}

struct Interpreter {
    tree: Tree,
    memory: HashMap<String, Variable>,
    locals: Vec<String>,
}

impl Interpreter {
    fn init_memory(&mut self) {
        self.memory.insert("equal".to_string(), Variable { value: Data::KraberFunction { body: equal }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("lt".to_string(), Variable { value: Data::KraberFunction { body: lt }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("nand".to_string(), Variable { value: Data::KraberFunction { body: nand }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("add".to_string(), Variable { value: Data::KraberFunction { body: add }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("multiply".to_string(), Variable { value: Data::KraberFunction { body: multiply }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("raise".to_string(), Variable { value: Data::KraberFunction { body: raise }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("floor".to_string(), Variable { value: Data::KraberFunction { body: floor }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("join".to_string(), Variable { value: Data::KraberFunction { body: join }, data_type: Data::Type { name: "kraberfunction".to_string() } });
        self.memory.insert("push".to_string(), Variable { value: Data::KraberFunction { body: push }, data_type: Data::Type { name: "kraberfunction".to_string() } });
    }

    fn eval_expression(&mut self, expression: Node) -> Data {
        return match &expression.nodes[0].data {
            Data::Identifier { name } => {
                let var = self.memory.get(&name.clone()).unwrap().clone();
                let data: Data = match &var.value {
                    Data::KraberFunction { body } => {
                        let args: Vec<Data> = expression.nodes[0].nodes.iter().map(
                            |x|
                            if matches!(&x.data, Data::Identifier { name }) {
                                let mut nodes: Vec<Node> = Vec::new();
                                nodes.push(x.clone());
                                let expression = Node {
                                    id: 0,
                                    data: Data::Expression,
                                    nodes,
                                };
                                self.eval_expression(expression)
                            } else {
                                x.data.clone()
                            }
                        ).collect();
                        body(args)
                    },
                    Data::Function { body, params, param_types, return_types } => {
                        let args: Vec<Data> = expression.nodes[0].nodes.iter().map(
                            |x|
                            if matches!(&x.data, Data::Identifier { name }) {
                                let mut nodes: Vec<Node> = Vec::new();
                                nodes.push(x.clone());
                                let expression = Node {
                                    id: 0,
                                    data: Data::Expression,
                                    nodes,
                                };
                                self.eval_expression(expression)
                            } else {
                                x.data.clone()
                            }
                        ).collect();
                        let tree = Tree {
                            root: Node {
                                id: 0,
                                data: Data::Main,
                                nodes: body.to_vec(),
                            }
                        };
                        let mut memory = self.memory.clone();
                        let mut locals: Vec<String> = Vec::new();
                        for i in 0..params.len() {
                            let param = &params[i];
                            let param_type = &param_types[i];
                            locals.push(param.to_string());
                            memory.insert(param.to_string(), Variable {
                                value: args[i].clone(),
                                data_type: param_type.clone(),
                            });
                        }
                        memory.insert("return".to_string(), Variable {
                            value: Data::Null,
                            data_type: return_types[0].clone(),
                        });
                        let mut sub = Interpreter {
                            tree,
                            memory,
                            locals,
                        };
                        sub.interpret();
                        sub.filter_memory();
                        self.memory = sub.memory.clone();
                        sub.memory.get("return").unwrap().value.clone()
                    },
                    _ => {
                        var.value.clone()
                    }
                };
                data
            },
            Data::FunctionContainer { params, param_types, return_types } => {
                Data::Function { params: params.to_vec(), param_types: param_types.to_vec(), body: expression.nodes[0].nodes.clone(), return_types: return_types.to_vec() }
            },
            _ => {
                expression.nodes[0].data.clone()
            }
        }
    }

    fn loop_while(&mut self, expression: Node, body: Vec<Node>) -> bool {
        let tree = Tree {
            root: Node {
                id: 0,
                data: Data::Main,
                nodes: body,
            },
        };
        let mut sub = Interpreter {
            tree,
            memory: self.memory.clone(),
            locals: Vec::new(),
        };
        let mut condition: bool = match self.eval_expression(expression.clone()) {
            Data::Boolean { value } => {
                value
            },
            _ => {
                panic!("expected boolean");
            }
        };
        while condition {
            sub.interpret();
            condition = match sub.eval_expression(expression.clone()) {
                Data::Boolean { value } => {
                    value
                },
                _ => {
                    panic!("expected boolean");
                }
            };
            match sub.memory.get("return") {
                Some(variable) => {
                    if variable.value != Data::Null {
                        self.memory = sub.memory.clone();
                        return true;
                    }
                },
                None => {},
            }
        }
        self.memory = sub.filter_memory().to_owned();
        false
    }

    fn interpret(&mut self) {
        for node in self.tree.root.nodes.clone() {
            match &node.data {
                Data::While => {
                    if self.loop_while(node.nodes[0].clone(), node.nodes[1..].to_vec().clone()) {
                        return;
                    }
                },
                Data::Return => {
                    let expression = Node {
                        id: 0,
                        data: Data::Expression,
                        nodes: node.nodes.clone(),
                    };
                    let value = self.eval_expression(expression);
                    self.memory.insert("return".to_string(), Variable {
                        value,
                        data_type: self.memory.get("return").unwrap().data_type.clone(),
                    });
                    return;
                },
                Data::Declare => {
                    match &node.nodes[0].data {
                        Data::Identifier { name } => {
                            let data_type = node.nodes[1].data.clone();
                            self.memory.insert(name.clone(), Variable {
                                value: match &data_type {
                                    Data::Type { name } => {
                                        match name.as_str() {
                                            "list" => {
                                                Data::List {
                                                    data_types: node.nodes[1].nodes.iter().map(
                                                        |x|
                                                        x.data.clone()
                                                    ).collect(),
                                                    value: [].to_vec(),
                                                }
                                            },
                                            _ => {
                                                Data::Null
                                            }
                                        }
                                    },
                                    _ => {
                                        panic!("expected Data::Type");
                                    }
                                },
                                data_type,
                            });
                            self.locals.push(name.to_string());
                        },
                        _ => {}
                    };
                },
                Data::Assign => {
                    match &node.nodes[0].data {
                        Data::Identifier { name } => {
                            let variable = self.memory.get(&name.clone()).unwrap().clone();
                            let data_type = variable.data_type;
                            let mut expression = Node {
                                id: 0,
                                data: Data::Expression,
                                nodes: Vec::new(),
                            };
                            expression.nodes.push(node.nodes[1].clone());
                            let mut expression_value = self.eval_expression(expression);
                            let type_name = stringify_enum(&expression_value);
                            println!("{:#?}", mem::discriminant(&expression_value));
                            match &data_type {
                                Data::Type { name } => {
                                    if name.to_string() != type_name {
                                        match expression_value {
                                            Data::Float { value: float }  => {
                                                match name.as_str() {
                                                    "whole" => {
                                                        if float < 0.0 {
                                                            panic!("could not cast negative float to whole");
                                                        }
                                                        expression_value = Data::Whole {
                                                            value: float as usize,
                                                        };
                                                    },
                                                    "integer" => {
                                                        expression_value = Data::Integer {
                                                            value: float as isize,
                                                        };
                                                    },
                                                    _ => {
                                                        panic!("could not cast {type_name:#?} to {name:#?}");
                                                    }
                                                }
                                            },
                                            Data::Integer { value: integer }  => {
                                                match name.as_str() {
                                                    "whole" => {
                                                        if integer < 0 {
                                                            panic!("could not cast negative integer to whole");
                                                        }
                                                        expression_value = Data::Whole {
                                                            value: integer as usize,
                                                        };
                                                    },
                                                    "float" => {
                                                        expression_value = Data::Float {
                                                            value: integer as f64,
                                                        };
                                                    },
                                                    _ => {
                                                        panic!("could not cast {type_name:#?} to {name:#?}");
                                                    }
                                                }
                                            },
                                            Data::Whole { value: whole }  => {
                                                match name.as_str() {
                                                    "integer" => {
                                                        expression_value = Data::Integer {
                                                            value: whole as isize,
                                                        };
                                                    },
                                                    "float" => {
                                                        expression_value = Data::Float {
                                                            value: whole as f64,
                                                        };
                                                    },
                                                    _ => {
                                                        panic!("could not cast {type_name:#?} to {name:#?}");
                                                    }
                                                }
                                            },
                                            _ => {
                                                panic!("could not cast {type_name:#?} to {name:#?}");
                                            }
                                        }
                                    } else if name == "list" {
                                        match &variable.value {
                                            Data::List { data_types: type_vector, value: old_vector } => {
                                                match expression_value {
                                                    Data::List { data_types, value } => {
                                                        if type_vector.to_vec() != data_types {
                                                            panic!("mismatched subtypes");
                                                        }
                                                        expression_value = Data::List { data_types, value };
                                                    },
                                                    _ => {
                                                        panic!("could not cast {type_name:#?} to {name:#?}");
                                                    }
                                                }
                                            },
                                            _ => {
                                                panic!("expected Data::List");
                                            }
                                        }
                                    }
                                },
                                _ => {
                                    panic!("expected data type to be Data::Type");
                                }
                            }
                            self.memory.insert(name.clone(), Variable {
                                value: expression_value,
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
                        Data::Boolean { value } => println!("{value}"),
                        Data::Text { value } => println!("{value}"),
                        Data::KraberFunction { body } => {
                            let mut expression = Node {
                                id: 0,
                                data: Data::Expression,
                                nodes: Vec::new(),
                            };
                            expression.nodes.push(node.clone());
                            println!("{}", self.eval_expression(expression))
                        }
                        _ => {}
                    };
                },
                _ => {}
            };
        }
    }

    fn filter_memory(&mut self) -> &mut HashMap<String, Variable> {
        for local in &self.locals {
            if self.memory.contains_key(local) {
                self.memory.remove(local);
            }
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
        keywords: vec!["declare".to_string(), "as".to_string(), "set".to_string(), "to".to_string(), "while".to_string(), "return".to_string()],
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
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("missing path to kraber file (cargo run -- <path>)");
    }
    let code = fs::read_to_string(&args[1]).expect("file not found");
    // println!("{}", code);
    let start_time = Instant::now();
    let mut lexer = create_lexer(code);
    let tokens = lexer.get_tokens();
    for token in &tokens {
        println!("{token}");
    }
    let mut parser = create_parser(tokens);
    let ast = parser.parse();
    println!("{ast:#?}");
    let mut interpreter = Interpreter {
        tree: ast,
        memory: HashMap::new(),
        locals: Vec::new(),
    };
    interpreter.init_memory();
    interpreter.interpret();
    let elapsed = start_time.elapsed();
    // println!("{:#?}", elapsed);
    let memory = interpreter.memory;
    println!("{memory:#?}");
}
