use anyhow::{anyhow, Result};
use std::{fmt, io::Write, ops::Mul, ptr::read_unaligned};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OperatorType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatorType::Add => write!(f, "+"),
            OperatorType::Subtract => write!(f, "-"),
            OperatorType::Multiply => write!(f, "*"),
            OperatorType::Divide => write!(f, "/"),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TokenType {
    Number(f32),
    Operator(OperatorType),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Number(n) => write!(f, "Number({})", n),
            TokenType::Operator(op) => write!(f, "Operator({})", op),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Token<'a> {
    type_: TokenType,
    start: usize,
    end: usize,
    literal: &'a str,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token {{ type: {}, literal: \"{}\" }}",
            self.type_, self.literal
        )
    }
}

pub struct Tokenizer<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip whitespace
        while self.index < self.input.len()
            && self.input.chars().nth(self.index).unwrap().is_whitespace()
        {
            self.index += 1;
        }

        if self.index >= self.input.len() {
            return None;
        }

        let start = self.index;
        let ch = self.input.chars().nth(self.index).unwrap();

        let token_type = if ch.is_ascii_digit() || ch == '.' {
            // Parse number
            while self.index < self.input.len() {
                let c = self.input.chars().nth(self.index).unwrap();
                if c.is_ascii_digit() || c == '.' {
                    self.index += 1;
                } else {
                    break;
                }
            }
            let literal = &self.input[start..self.index];
            let num = literal.parse::<f32>().ok()?;
            TokenType::Number(num)
        } else {
            // Parse operator
            self.index += 1;
            let token_type = match ch {
                '+' => TokenType::Operator(OperatorType::Add),
                '-' => TokenType::Operator(OperatorType::Subtract),
                '*' => TokenType::Operator(OperatorType::Multiply),
                '/' => TokenType::Operator(OperatorType::Divide),
                _ => return None,
            };
            token_type
        };

        let end = self.index;
        let literal = &self.input[start..end];

        Some(Token {
            type_: token_type,
            start,
            end,
            literal,
        })
    }
}

impl<'a> Tokenizer<'a> {
    pub fn tokenize(source: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            input: source,
            index: 0,
        }
    }
}

pub struct Parser<'a> {
    pub tokenizer: std::iter::Peekable<Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn accept(self: &mut Self, check: impl Fn(TokenType) -> bool) -> Option<Token<'a>> {
        if let Some(token) = self.tokenizer.peek() {
            if check(token.type_) {
                return self.tokenizer.next();
            }
        }
        None
    }

    pub fn except(self: &mut Self, check: impl Fn(TokenType) -> bool) -> Result<Token<'a>> {
        if let Some(token) = self.accept(check) {
            return Ok(token);
        }
        Err(anyhow!("unexpected token"))
    }

    /// a factor is either:
    /// a number
    pub fn factor(self: &mut Self) -> Result<f32> {
        if let Some(token) = self.tokenizer.next() {
            match token.type_ {
                TokenType::Number(n) => Ok(n),
                _ => Err(anyhow!("expected number, got {}", token.type_)),
            }
        } else {
            Err(anyhow!("unexpected end of input"))
        }
    }

    /// a term is:
    /// factor (* | /) factor (* | /) factor ...
    pub fn term(self: &mut Self) -> Result<f32> {
        use OperatorType::*;
        use TokenType::*;
        let mut left = self.factor()?;
        while let Some(op) = self.accept(|t| matches!(t, Operator(Multiply) | Operator(Divide))) {
            let right = self.factor()?;
            left = match op.type_ {
                Operator(Multiply) => left * right,
                Operator(Divide) => left / right,
                _ => unreachable!(),
            };
        }
        Ok(left)
    }

    /// an expression is:
    /// term (+ | -) term (+ | -) term ...
    pub fn expression(self: &mut Self) -> Result<f32> {
        use OperatorType::*;
        use TokenType::*;
        let mut left = self.term()?;
        while let Some(op) = self.accept(|t| matches!(t, Operator(Add) | Operator(Subtract))) {
            let right = self.term()?;
            left = match op.type_ {
                Operator(Add) => left + right,
                Operator(Subtract) => left - right,
                _ => unreachable!(),
            };
        }
        Ok(left)
    }
}

fn main() -> Result<()> {
    let mut buf = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        _ = std::io::stdin().read_line(&mut buf)?;

        // exit on q or quit
        if &buf == "q\n" || &buf == "quit\n" {
            return Ok(());
        }

        let tokenizer = Tokenizer::tokenize(&buf).peekable();
        let mut parser = Parser { tokenizer };

        let result = parser.expression()?;
        println!("{}", result);
        buf.clear();
    }
}
