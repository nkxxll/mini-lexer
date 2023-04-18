use anyhow::Result;
use regex::Regex;
use std::{io, fmt::Formatter};

const NUMBER_TOKEN: &str = r"[0-9]";
const OPERATOR_TOKEN: &str = r"[+-]";

fn main() -> Result<()> {
    let mut input = String::new();
    let number_regex = Regex::new(NUMBER_TOKEN).unwrap();
    let operator_regex = Regex::new(OPERATOR_TOKEN).unwrap();
    io::stdin().read_line(&mut input).unwrap();
    let mut lexer = Lexer::new(
        input,
        Rex {
            number_regex,
            operator_regex,
        },
    );
    loop {
        lexer.next_token();
        if lexer.position == lexer.input.len() {
            break;
        }
    }
    println!("These are the tokens: {:?}", lexer.tokens);
    Ok(())
}

struct Rex {
    number_regex: Regex,
    operator_regex: Regex,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
    current_token: Token,
    tokens: Vec<Token>,
    regex: Rex,
}

impl Lexer {
    fn new(input: String, rex: Rex) -> Self {
        let token = Token {
            token_type: TokenType::NonToken,
            literal: Vec::new(),
        };
        let lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            current_token: token,
            regex: rex,
            tokens: Vec::new(),
        };
        lexer
    }
    fn next_token(&mut self) {
        let c: char = self.input[self.position];
        // book keeping
        self.position += 1;
        if self.regex.number_regex.is_match(&c.to_string()) {
            if self.current_token.token_type == TokenType::Number {
                self.current_token.literal.push(c);
            } else if self.current_token.token_type == TokenType::Operator {
                self.tokens.push(self.current_token.clone());
                self.current_token.token_type = TokenType::Number;
                // clear the literal
                self.current_token.literal = Vec::new();
                self.current_token.literal.push(c);
            } else {
                self.current_token.token_type = TokenType::Number;
                self.current_token.literal.push(c);
            }
        } else if self.regex.operator_regex.is_match(&c.to_string()) {
            self.current_token.token_type = TokenType::Operator;
            self.current_token.literal = Vec::new();
            self.current_token.literal.push(c);
        } else if self.current_token.token_type != TokenType::NonToken {
            self.tokens.push(self.current_token.clone());
        } else {
            // TODO: decide whether it is a whitespace char or an illegal char
            // if it is a whitespace char, do nothing
            // if it is an illegal char, throw an error
            //
            // do nothing reset the current literal
            self.current_token.token_type = TokenType::NonToken;
            self.current_token.literal = Vec::new();
        }
    }
}

#[derive(PartialEq, Clone)]
struct Token {
    token_type: TokenType,
    literal: Vec<char>,
}

// impl Debug for Token
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("token_type", &self.token_type)
            .field("literal", &self.literal.iter().collect::<String>())
            .finish()
    }
}

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    Number,
    Operator,
    NonToken,
}
