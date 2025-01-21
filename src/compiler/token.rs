use super::location::Location;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Integer,
    Operator,
    Punctuation,
    Identifier,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    text: String,
    token_type: TokenType,
    loc: Location,
}

impl Token {
    pub fn new<S: Into<String>>(text: S, token_type: TokenType, loc: Location) -> Self {
        Token {
            text: text.into(),
            token_type,
            loc,
        }
    }

    // Return the content of token
    pub fn get_text(&self) -> &str {
        &self.text
    }

    // Return type of token
    pub fn get_type(&self) -> &TokenType {
        &self.token_type
    }

    // Return location of token
    pub fn get_loc(&self) -> &Location {
        &self.loc
    }
}
