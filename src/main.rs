struct Token {
    value: String,
    category: String,
}

struct TokenFactory {
    code: String,
    index: usize,
    character: char,
    end: bool,
}

impl TokenFactory {
    fn step(&mut self) {
        self.index += 1;
        self.character = self.code.chars().nth(self.index).expect("index is out of range");
    }

    fn get_tokens(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        while !self.end {
            println!("{}", self.character);
            self.step();
        }
        tokens
    }
}

fn main() {
    let code = String::from("declare x");
    let character = code.chars().nth(0).expect("blank code string detected");
    let mut token_factory = TokenFactory {
        code,
        index: 0,
        character,
        end: false,
    };
    let tokens = token_factory.get_tokens();
}