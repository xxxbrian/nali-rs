use crate::geo::geodb::GeoLocation;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Plain(String),
    IPv4(String, Option<GeoLocation>),
    IPv6(String, Option<GeoLocation>),
    Domain(String),
}

impl Token {
    // Get the priority of the token
    pub fn priority(&self) -> u8 {
        match self {
            Token::IPv4(_, _) => 3, // The highest priority
            Token::IPv6(_, _) => 2, // The second highest priority
            Token::Domain(_) => 1,  // The lowest priority
            Token::Plain(_) => 0,   // No priority
        }
    }
}

impl Token {
    pub fn is_plain(&self) -> bool {
        matches!(self, Token::Plain(_))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Plain(text) => write!(f, "{}", text),
            Token::IPv4(addr, _) => write!(f, "{} [IPv4]", addr),
            Token::IPv6(addr, _) => write!(f, "{} [IPv6]", addr),
            Token::Domain(domain) => write!(f, "{} [Domain]", domain),
        }
    }
}
