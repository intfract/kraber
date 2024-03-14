use std::{any::Any, collections::HashMap, fmt};

mod functions;
use functions::add;

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
                } else {
                    tokens.push(Token { value: word, category: "REF".to_string() });
                }
            } else if self.commutators.contains(self.character) {
                tokens.push(Token { value: self.character.to_string(), category: "COM".to_string() })
            } else if self.digits.contains(self.character) || self.character == '+' || self.character == '-' {
                let mut number = self.character.to_string();
                number.push_str(&self.get_number());
                if number.contains('.') {
                    tokens.push(Token { value: self.character.to_string(), category: "FLT".to_string() })
                } else {
                    tokens.push(Token { value: self.character.to_string(), category: "INT".to_string() })
                }
            }
            self.step();
        }
        tokens
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
        keywords: vec!["declare".to_string(), "as".to_string(), "set".to_string(), "to".to_string()],
    }
}

fn main() {
    let code = "declare x\nset x to 1".to_string();
    let mut token_factory = create_token_factory(code);
    let tokens = token_factory.get_tokens();
    for token in &tokens {
        println!("token: {token}");
    }
}