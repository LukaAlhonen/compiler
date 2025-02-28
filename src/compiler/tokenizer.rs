use regex::Regex;

#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub line: String,
    pub column: String,
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.is_special()
            || other.is_special()
            || (self.file == other.file && self.line == other.line && self.column == other.column)
    }
}

impl Location {
    // Used for testing, "special" loc is equal to all other loc structs
    pub fn special() -> Self {
        Location {
            file: "SPECIAL".to_string(),
            line: "SPECIAL".to_string(),
            column: "SPECIAL".to_string(),
        }
    }

    fn is_special(&self) -> bool {
        self.file == "SPECIAL".to_string()
            && self.line == "SPECIAL".to_string()
            && self.column == "SPECIAL".to_string()
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.column)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Integer,
    Boolean,
    Operator,
    Punctuation,
    Identifier,
    End,
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::Integer => String::from("Integer"),
            TokenType::Boolean => String::from("Boolean"),
            TokenType::Operator => String::from("Operator"),
            TokenType::Identifier => String::from("Identifier"),
            TokenType::Punctuation => String::from("Punctuation"),
            TokenType::End => String::from("End"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
    pub loc: Location,
}

pub fn tokenize(source_code: &str, file_name: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

    let re = Regex::new(
        r"([a-zA-Z_]+[a-zA-Z0-9]*)|([0-9]+)|(\*|\+|\-|\/|==|!=|<=|>=|=|>|<)|([\(\)\[\]\{\}\,\;\:])",
    )
    .unwrap();

    for (line_number, line) in source_code.lines().enumerate() {
        // Remove comments from line before matching (# //)
        let line = line
            .split("//")
            .next()
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or("");
        // Match each capture group of regex and push token with coresponding type onto tokens vec
        for mat in re.captures_iter(line) {
            if let Some(identifier) = mat.get(1) {
                let text = identifier.as_str();
                let token_type = if text == "true" || text == "false" {
                    TokenType::Boolean
                } else if text == "and" || text == "or" {
                    TokenType::Operator
                } else {
                    TokenType::Identifier
                };
                tokens.push(Token {
                    text: text.to_string(),
                    token_type,
                    loc: Location {
                        file: file_name.to_string(),
                        line: (line_number + 1).to_string(),
                        column: (identifier.start() + 1).to_string(),
                    },
                })
            } else if let Some(integer) = mat.get(2) {
                tokens.push(Token {
                    text: integer.as_str().to_string(),
                    token_type: TokenType::Integer,
                    loc: Location {
                        file: file_name.to_string(),
                        line: (line_number + 1).to_string(),
                        column: (integer.start() + 1).to_string(),
                    },
                })
            } else if let Some(operator) = mat.get(3) {
                tokens.push(Token {
                    text: operator.as_str().to_string(),
                    token_type: TokenType::Operator,
                    loc: Location {
                        file: file_name.to_string(),
                        line: (line_number + 1).to_string(),
                        column: (operator.start() + 1).to_string(),
                    },
                })
            } else if let Some(punctuation) = mat.get(4) {
                tokens.push(Token {
                    text: punctuation.as_str().to_string(),
                    token_type: TokenType::Punctuation,
                    loc: Location {
                        file: file_name.to_string(),
                        line: (line_number + 1).to_string(),
                        column: (punctuation.start() + 1).to_string(),
                    },
                })
            }
        }
    }

    tokens
}
