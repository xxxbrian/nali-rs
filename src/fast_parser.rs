use std::cmp::Ordering;

use crate::geo::geodb::GeoDB;
use crate::parser::Parser;
use crate::token::Token;
use crate::NaliText;

#[derive(PartialEq, Clone, Copy, Debug)]
enum IPv4State {
    Any,
    Digit,
    Dot,
}

#[derive(Debug)]
struct IPv4Token {
    last_state: IPv4State,
    dot_count: u8,
    current_number: u32,
    start: usize,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum IPv6State {
    Any,
    Hex,
    Colon,
}

#[derive(Debug)]
struct IPv6Token {
    last_state: IPv6State,
    colon_count: u8,
    has_double_colon: bool,
    group_len: u8,
    start: usize,
}

#[derive(Default)]
pub struct FastParser {}

impl FastParser {
    // Check if two ranges are overlapping
    fn is_overlapping(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
        start1 < end2 && start2 < end1
    }

    // Decide which token to keep when two tokens are overlapping
    fn should_keep_first(token1: &Token, start1: usize, token2: &Token, start2: usize) -> bool {
        match token1.priority().cmp(&token2.priority()) {
            Ordering::Greater => true,           // First token has higher priority
            Ordering::Less => false,             // Second token has higher priority
            Ordering::Equal => start1 <= start2, // Same priority, keep the first one
        }
    }

    fn match_ipv6(&self, input: &str) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();
        let mut v6_token = IPv6Token {
            last_state: IPv6State::Any,
            colon_count: 0,
            has_double_colon: false,
            group_len: 0,
            start: 0,
        };

        fn is_hex_char(c: char) -> IPv6State {
            if c.is_ascii_hexdigit() {
                IPv6State::Hex
            } else if c == ':' {
                IPv6State::Colon
            } else {
                IPv6State::Any
            }
        }

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            match (v6_token.last_state, is_hex_char(c)) {
                (IPv6State::Any, IPv6State::Hex) => {
                    // Hex start
                    // A new IPv6 token might start here
                    v6_token.start = i;
                    v6_token.last_state = IPv6State::Hex;
                    v6_token.group_len = 1;
                }
                (IPv6State::Hex, IPv6State::Hex) => {
                    // Hex continuation
                    if v6_token.group_len >= 4 {
                        // Group is full
                        if v6_token.has_double_colon || v6_token.colon_count == 7 {
                            // Already have 7 colons, commit the token
                            matches.push((v6_token.start, i));
                            // Reset token
                            v6_token.last_state = IPv6State::Any;
                            v6_token.colon_count = 0;
                            v6_token.has_double_colon = false;
                            v6_token.group_len = 0;
                        } else {
                            // Move token
                            v6_token.group_len = 3;
                            v6_token.start = i - 3; // Don't keep first digits
                            v6_token.has_double_colon = false;
                            v6_token.colon_count = 0;
                        }
                    } else {
                        v6_token.group_len += 1;
                    }
                }
                (IPv6State::Hex, IPv6State::Colon) => {
                    // Hex end by colon
                    if (v6_token.has_double_colon && v6_token.colon_count == 6)
                        || v6_token.colon_count == 7
                    {
                        // Already have 7 colons, commit the token
                        matches.push((v6_token.start, i));
                        // Reset token
                        v6_token.last_state = IPv6State::Any;
                        v6_token.colon_count = 0;
                        v6_token.has_double_colon = false;
                        v6_token.group_len = 0;
                    } else {
                        v6_token.colon_count += 1;
                        v6_token.last_state = IPv6State::Colon;
                    }
                }
                (IPv6State::Hex, IPv6State::Any) => {
                    // Hex end by random char
                    if v6_token.has_double_colon || v6_token.colon_count == 7 {
                        // Already have 7 colons, commit the token
                        matches.push((v6_token.start, i));
                    }
                    v6_token.last_state = IPv6State::Any;
                    v6_token.colon_count = 0;
                    v6_token.has_double_colon = false;
                    v6_token.group_len = 0;
                }
                (IPv6State::Colon, IPv6State::Hex) => {
                    // Colon start after colon
                    v6_token.last_state = IPv6State::Hex;
                    v6_token.group_len = 1;
                }
                (IPv6State::Colon, IPv6State::Colon) => {
                    // Colon start after colon
                    if v6_token.has_double_colon {
                        // Already have double colon
                        matches.push((v6_token.start, i - 1));
                        v6_token.last_state = IPv6State::Any;
                        v6_token.colon_count = 0;
                        v6_token.has_double_colon = false;
                        v6_token.group_len = 0;
                    } else {
                        v6_token.has_double_colon = true;
                        v6_token.last_state = IPv6State::Colon;
                    }
                }
                (IPv6State::Colon, IPv6State::Any) => {
                    // Colon end by random char
                    if v6_token.has_double_colon {
                        // Already have double colon, commit the token
                        matches.push((v6_token.start, i));
                    }
                    v6_token.last_state = IPv6State::Any;
                    v6_token.colon_count = 0;
                    v6_token.has_double_colon = false;
                    v6_token.group_len = 0;
                }
                (_, _) => {
                    v6_token.last_state = IPv6State::Any;
                    v6_token.colon_count = 0;
                    v6_token.has_double_colon = false;
                    v6_token.group_len = 0;
                }
            }

            i += 1;
        }

        // Last token
        if v6_token.has_double_colon || v6_token.colon_count == 7 {
            // Find last non colon character
            let mut last_non_colon = chars.len();
            for i in (0..chars.len()).rev() {
                if chars[i] != ':' {
                    last_non_colon = i + 1;
                    break;
                }
            }
            matches.push((v6_token.start, last_non_colon));
        };

        matches
    }

    fn match_ipv4(&self, input: &str) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();

        let mut v4_token = IPv4Token {
            last_state: IPv4State::Any,
            dot_count: 0,
            current_number: 0,
            start: 0,
        };

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // IPv4 parsing
            match (v4_token.last_state, c) {
                (IPv4State::Any, '0'..='9') => {
                    // Digit start
                    // A new IPv4 token might start here
                    v4_token.start = i;
                    v4_token.last_state = IPv4State::Digit;
                    v4_token.current_number = c.to_digit(10).unwrap();
                }
                (IPv4State::Digit, '0'..='9') => {
                    // Digit continuation
                    let current_number = v4_token.current_number * 10 + c.to_digit(10).unwrap();
                    if current_number > 255 {
                        // Overflow
                        if v4_token.dot_count == 3 {
                            // Check if previous token already valid, commit it
                            matches.push((v4_token.start, i));
                            // A new IPv4 token might start here
                            v4_token.start = i;
                            v4_token.dot_count = 0;
                            v4_token.current_number = c.to_digit(10).unwrap();
                            v4_token.last_state = IPv4State::Digit;
                        } else {
                            // Move token
                            v4_token.last_state = IPv4State::Digit;
                            v4_token.dot_count = 0;
                            v4_token.start = i - 1;
                            // dont keep first digit
                            v4_token.current_number = current_number % 100;
                        }
                    } else {
                        // Normal digit
                        v4_token.current_number = current_number;
                        v4_token.last_state = IPv4State::Digit;
                    }
                }
                (IPv4State::Digit, '.') => {
                    // Digit end by dot
                    if v4_token.dot_count == 3 {
                        // Already have 3 dots, commit the token
                        matches.push((v4_token.start, i));
                        // Reset token
                        v4_token.last_state = IPv4State::Any;
                        v4_token.dot_count = 0;
                        v4_token.current_number = 0;
                    } else {
                        v4_token.dot_count += 1;
                        v4_token.last_state = IPv4State::Dot;
                    }
                }
                (IPv4State::Digit, _) => {
                    // Digit end by random char
                    if v4_token.dot_count == 3 {
                        // Already have 3 dots, commit the token
                        matches.push((v4_token.start, i));
                    }
                    v4_token.last_state = IPv4State::Any;
                    v4_token.dot_count = 0;
                    v4_token.current_number = 0;
                }
                (IPv4State::Dot, '0'..='9') => {
                    // Digit start after dot
                    v4_token.last_state = IPv4State::Digit;
                    v4_token.current_number = c.to_digit(10).unwrap();
                }
                (_, _) => {
                    v4_token.last_state = IPv4State::Any;
                    v4_token.dot_count = 0;
                    v4_token.current_number = 0;
                }
            }

            i += 1;
        }

        // Last token
        if v4_token.dot_count == 3 && v4_token.last_state == IPv4State::Digit {
            matches.push((v4_token.start, chars.len()));
        };

        matches
    }
}

impl<G: GeoDB> Parser<G> for FastParser {
    fn name(&self) -> &str {
        "fast"
    }

    fn parse(&self, input: &str, db: &G) -> NaliText {
        let mut tokens: Vec<Token> = Vec::new();
        let mut matches: Vec<(usize, usize, Token)> = Vec::new();
        let ipv4_matches = self.match_ipv4(input);
        let ipv6_matches = self.match_ipv6(input);

        ipv4_matches.iter().for_each(|(start, end)| {
            let ip = &input[*start..*end];
            matches.push((*start, *end, Token::IPv4(ip.to_string(), db.lookup(ip))));
        });

        ipv6_matches.iter().for_each(|(start, end)| {
            let ip = &input[*start..*end];
            matches.push((*start, *end, Token::IPv6(ip.to_string(), db.lookup(ip))));
        });

        // Sort matches by start position
        matches.sort_by_key(|(start, _, _)| *start);

        // Handle overlapping matches
        let mut filtered_matches = Vec::new();
        let mut i = 0;
        while i < matches.len() {
            let current = &matches[i];
            let mut should_add = true;
            let mut j = i + 1;

            while j < matches.len()
                && Self::is_overlapping(current.0, current.1, matches[j].0, matches[j].1)
            {
                if !Self::should_keep_first(&current.2, current.0, &matches[j].2, matches[j].0) {
                    should_add = false;
                    break;
                }
                j += 1;
            }

            if should_add {
                filtered_matches.push(current.clone());
                while j < matches.len()
                    && Self::is_overlapping(current.0, current.1, matches[j].0, matches[j].1)
                {
                    j += 1;
                }
                i = j;
            } else {
                i += 1;
            }
        }

        let mut last_end = 0;

        // Construct the final token sequence
        for (start, end, token) in filtered_matches {
            if start > last_end {
                tokens.push(Token::Plain(input[last_end..start].to_string()));
            }
            tokens.push(token);
            last_end = end;
        }

        // Add the remaining plain text
        if last_end < input.len() {
            tokens.push(Token::Plain(input[last_end..].to_string()));
        }

        NaliText::new(tokens)
    }
}
