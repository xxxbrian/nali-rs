use std::cmp::Ordering;

use lazy_static::lazy_static;
use regex::Regex;

use crate::geo::geodb::GeoDB;
use crate::parser::Parser;
use crate::token::Token;
use crate::NaliText;

// from https://github.com/zu1k/nali/blob/master/pkg/re/re.go
lazy_static! {
    static ref IPV4_REGEX: Regex = Regex::new(
        r"(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}"
    )
    .unwrap();

    static ref IPV6_REGEX: Regex = Regex::new(
        r"fe80:(:[0-9a-fA-F]{1,4}){0,4}(%\w+)?|([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|::[fF]{4}:(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}|(([0-9a-fA-F]{1,4}:){0,6}[0-9a-fA-F]{1,4})?::(([0-9a-fA-F]{1,4}:){0,6}[0-9a-fA-F]{1,4})?"
    )
    .unwrap();

    static ref DOMAIN_REGEX: Regex = Regex::new(
        r"([a-zA-Z0-9][-a-zA-Z0-9]{0,62}\.)+([a-zA-Z][-a-zA-Z]{0,62})"
    )
    .unwrap();
}

pub struct RegexParser<G: GeoDB> {
    db: G,
}

impl<G: GeoDB> RegexParser<G> {
    pub fn new(db: G) -> Self {
        Self { db }
    }

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
}

impl<G: GeoDB> Parser for RegexParser<G> {
    fn name(&self) -> &str {
        "regex"
    }

    fn parse(&self, input: &str) -> NaliText {
        let mut tokens = Vec::new();
        let mut last_end = 0;

        // Save all matches
        let mut matches = Vec::new();

        // Collect by priority
        // 1. IPv4 (highest priority)
        for ip_match in IPV4_REGEX.find_iter(input) {
            matches.push((
                ip_match.start(),
                ip_match.end(),
                Token::IPv4(
                    ip_match.as_str().to_string(),
                    self.db.lookup(ip_match.as_str()),
                ),
            ));
        }

        // 2. IPv6
        for ip_match in IPV6_REGEX.find_iter(input) {
            matches.push((
                ip_match.start(),
                ip_match.end(),
                Token::IPv6(
                    ip_match.as_str().to_string(),
                    self.db.lookup(ip_match.as_str()),
                ),
            ));
        }

        // 3. Domain (lowest priority)
        for domain_match in DOMAIN_REGEX.find_iter(input) {
            matches.push((
                domain_match.start(),
                domain_match.end(),
                Token::Domain(domain_match.as_str().to_string()),
            ));
        }

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
