use super::ast::*;
use super::tokenizer::{Location, Token, TokenType};

enum Expected {
    Single(String),
    Multiple(Vec<String>),
    None,
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnexpectedToken {
        location: Location,
        expected: String,
    },
    InvalidInteger(String),
}

pub struct Parser {
    pos: i32,
    tokens: Vec<Token>,
}

// Implemented Parser as struct for easier tracking of pos and tokens
// TODO:
// Support for all operators
// If and while statements
// Function calls
// Blocks
// variable declaration
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // Check what token is at postion pos
    fn peek(&self) -> Token {
        if self.pos >= 0 && (self.pos as usize) < self.tokens.len() {
            // Don't like clone() here but functions using peek() need ownership of the returned
            // token, as of now atleast
            self.tokens[self.pos as usize].clone()
        } else {
            // If we have run out of tokens, then return End token with loc of last token in tokens
            // If tokens is empty, return end token with special loc
            if let Some(last_token) = self.tokens.last() {
                Token {
                    text: "".to_string(),
                    token_type: TokenType::End,
                    loc: last_token.loc.clone(),
                }
            } else {
                Token {
                    text: "".to_string(),
                    token_type: TokenType::End,
                    loc: Location::special(),
                }
            }
        }
    }

    // Get token at current pos and move pos one step forward
    // expected can hold None, String, or Vec<String>
    fn consume(&mut self, exptected: Expected) -> Result<Token, ParserError> {
        let token: Token = self.peek();

        match exptected {
            Expected::Single(s) => {
                if s != token.text {
                    return Err(ParserError::UnexpectedToken {
                        location: token.loc,
                        expected: s,
                    });
                }
            }
            Expected::Multiple(v) => {
                if !v.contains(&token.text) {
                    return Err(ParserError::UnexpectedToken {
                        location: token.loc,
                        expected: v.join(", "),
                    });
                }
            }
            Expected::None => {}
        }
        self.pos += 1;
        Ok(token)
    }

    // Parse the token at current pos into int literal, expects token to be of type Integer
    fn parse_int_literal(&mut self) -> Result<Literal, ParserError> {
        let token = self.peek();

        match token.token_type {
            TokenType::Integer => {
                // parse int, if it fails, map the error onto ParserError
                let int_val = self
                    .consume(Expected::None)?
                    .text
                    .parse::<i32>()
                    .map_err(|_| ParserError::InvalidInteger(token.text))?;
                Ok(Literal {
                    value: int_val.into(),
                })
            }
            _ => Err(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::Integer.to_string(),
            }),
        }
    }

    // Get identifier at current pos, expects token to be identifier
    fn parse_identifier(&mut self) -> Result<Identifier, ParserError> {
        let token = self.peek();

        match token.token_type {
            TokenType::Identifier => {
                let identifier = self.consume(Expected::None)?;

                Ok(Identifier {
                    name: identifier.text,
                })
            }
            _ => Err(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::Identifier.to_string(),
            }),
        }
    }

    // Get expression inside paranthesis, expects epxression to be wrapped in ()
    fn parse_parenthesized(&mut self) -> Result<Box<dyn Expression>, ParserError> {
        self.consume(Expected::Single("(".to_string()))?;

        let expr = self.parse_expression()?;
        self.consume(Expected::Single(")".to_string()))?;
        Ok(expr)
    }

    // Get int literal, identifier or epxression in parenthises at current pos
    fn parse_factor(&mut self) -> Result<Box<dyn Expression>, ParserError> {
        let token: Token = self.peek();

        if token.text == "(" {
            Ok(self.parse_parenthesized()?)
        } else {
            match token.token_type {
                TokenType::Integer => Ok(Box::new(self.parse_int_literal()?)),
                TokenType::Identifier => Ok(Box::new(self.parse_identifier()?)),
                _ => Err(ParserError::UnexpectedToken {
                    location: token.loc.clone(),
                    expected: "Integer or Identifier".to_string(),
                }),
            }
        }
    }

    // get expression in for of Expression * Expression or Expression / Expression
    // left assiciative
    fn parse_term(&mut self) -> Result<Box<dyn Expression>, ParserError> {
        let mut left = self.parse_factor()?;

        while ["*".to_string(), "/".to_string()].contains(&self.peek().text) {
            let op = self.consume(Expected::None)?.text;
            let right = self.parse_factor()?;
            left = Box::new(BinaryOp { left, op, right });
        }

        Ok(left)
    }

    // get expression in for of Expression + Expression or Expression - Expression
    // left assiciative
    fn parse_expression(&mut self) -> Result<Box<dyn Expression>, ParserError> {
        let mut left = self.parse_term()?;

        while ["+".to_string(), "-".to_string()].contains(&self.peek().text) {
            let op = self.consume(Expected::None)?.text;
            let right = self.parse_term()?;
            left = Box::new(BinaryOp { left, op, right });
        }

        Ok(left)
    }

    pub fn parse(&mut self) -> Result<Box<dyn Expression>, ParserError> {
        let expr = self.parse_expression()?;

        // Make sure the whole input has been parsed, if it has then peek() will return End token
        // If not, return error
        let token: Token = self.peek();
        if token.token_type != TokenType::End {
            return Err(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::End.to_string(),
            });
        }

        Ok(expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::tokenizer::tokenize;

    #[test]
    fn test_peek() {
        let tokens = tokenize("2 - 3 + 1", "file.txt");
        let p: Parser = Parser::new(tokens);
        assert_eq!(
            Token {
                text: "2".to_string(),
                token_type: TokenType::Integer,
                loc: Location {
                    file: "file.txt".to_string(),
                    line: "1".to_string(),
                    column: "1".to_string()
                }
            },
            p.peek()
        );
    }

    #[test]
    fn test_consume_single() {
        let tokens = tokenize("2 - 3 + 1", "file.txt");
        let mut p: Parser = Parser::new(tokens);
        let exp_single: String = "2".to_string();
        let exp_mult: Vec<String> = vec!["-".to_string(), "+".to_string()];
        assert_eq!(
            Token {
                text: "2".to_string(),
                token_type: TokenType::Integer,
                loc: Location::special(),
            },
            p.consume(Expected::Single(exp_single)).unwrap(),
        );
        assert_eq!(
            Token {
                text: "-".to_string(),
                token_type: TokenType::Operator,
                loc: Location::special(),
            },
            p.consume(Expected::Multiple(exp_mult)).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_consume_single_panic() {
        let tokens = tokenize("2 - 3 + 1", "file.txt");
        let mut p: Parser = Parser::new(tokens);
        let exp: String = "-".to_string();

        p.consume(Expected::Single(exp)).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_consume_mul_panic() {
        let tokens = tokenize("2 - 3 + 1", "file.txt");
        let mut p: Parser = Parser::new(tokens);
        let exp: Vec<String> = vec![
            "-".to_string(),
            "+".to_string(),
            "a".to_string(),
            "c".to_string(),
            "d".to_string(),
            "b".to_string(),
        ];

        p.consume(Expected::Multiple(exp)).unwrap();
    }

    #[test]
    fn test_parse_int_literal() {
        let tokens = tokenize("2 + 2", "file.txt");
        let mut p = Parser::new(tokens);
        assert_eq!(
            Literal {
                value: LiteralValue::Int(2)
            },
            p.parse_int_literal().unwrap()
        );
    }

    #[test]
    fn test_parse_identifier() {
        let tokens = tokenize("a + 1", "file.txt");
        let mut p = Parser::new(tokens);
        assert_eq!(
            Identifier {
                name: "a".to_string()
            },
            p.parse_identifier().unwrap()
        );
    }

    #[test]
    fn test_parse_term() {
        let tokens = tokenize("2 * 3", "file.txt");
        let mut p = Parser::new(tokens);

        let bin_op = BinaryOp {
            left: Box::new(Literal { value: 2.into() }),
            op: "*".to_string(),
            right: Box::new(Literal { value: 3.into() }),
        };

        assert_eq!(
            bin_op,
            *p.parse_term()
                .unwrap()
                .as_any()
                .downcast_ref::<BinaryOp>()
                .unwrap()
        )
    }

    #[test]
    fn test_parse_parenthesized() {
        let tokens = tokenize("2 + (3 * 4)", "file.txt");
        let mut p = Parser::new(tokens);

        let bin_op = BinaryOp {
            left: Box::new(Literal { value: 2.into() }),
            op: "+".to_string(),
            right: Box::new(BinaryOp {
                left: Box::new(Literal { value: 3.into() }),
                op: "*".to_string(),
                right: Box::new(Literal { value: 4.into() }),
            }),
        };

        assert_eq!(
            bin_op,
            *p.parse_expression()
                .unwrap()
                .as_any()
                .downcast_ref::<BinaryOp>()
                .unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_parenthesized_panic() {
        let tokens = tokenize("2 (3 * 4", "file.txt");
        let mut p = Parser::new(tokens);

        p.parse_parenthesized().unwrap();
    }
}
