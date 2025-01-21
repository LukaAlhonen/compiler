use super::location::Location as L;
use super::token::{Token, TokenType};
use regex::Regex;

pub fn tokenize(source_code: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let re = Regex::new(
        r"([a-zA-Z_]+[a-zA-Z0-9]*)|([0-9]+)|(\*|\+|\-|\/|==|!=|<=|>=|=|>|<)|([\(\)\[\]\{\}\,\;])",
    )
    .unwrap();

    for line in source_code.lines() {
        // Remove comments from line before matching (# //)
        let line = line
            .split("//")
            .next()
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or("");
        for mat in re.captures_iter(line) {
            if let Some(identifier) = mat.get(1) {
                tokens.push(Token::new(
                    identifier.as_str(),
                    TokenType::Identifier,
                    L::special(),
                ))
            } else if let Some(integer) = mat.get(2) {
                tokens.push(Token::new(
                    integer.as_str(),
                    TokenType::Integer,
                    L::special(),
                ))
            } else if let Some(operator) = mat.get(3) {
                tokens.push(Token::new(
                    operator.as_str(),
                    TokenType::Operator,
                    L::special(),
                ))
            } else if let Some(punctuation) = mat.get(4) {
                tokens.push(Token::new(
                    punctuation.as_str(),
                    TokenType::Punctuation,
                    L::special(),
                ))
            }
        }
    }

    tokens
}
