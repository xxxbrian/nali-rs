use std::fmt::Display;

pub mod geo;
pub mod parser;
pub mod regex_parser;
pub mod token;

use colored::Colorize;
pub use parser::Parser;
pub use regex_parser::RegexParser;
pub use token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct NaliText {
    tokens: Vec<Token>,
}

impl NaliText {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn colorize(&self) -> String {
        self.tokens
            .iter()
            .map(|token| match token {
                Token::Plain(text) => text.to_string(),
                Token::IPv4(addr, geo) => format!(
                    "{} [{}]",
                    addr.green(),
                    match geo {
                        Some(geo) => geo.location.red(),
                        None => "Unknown".red(),
                    }
                ),
                Token::IPv6(addr, geo) => format!(
                    "{} [{}]",
                    addr.blue(),
                    match geo {
                        Some(geo) => geo.location.red(),
                        None => "Unknown".red(),
                    }
                ),
                Token::Domain(domain) => domain.yellow().to_string(),
            })
            .collect()
    }
}

impl Display for NaliText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.tokens
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join("")
        )
    }
}
