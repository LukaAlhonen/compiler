use super::ast::*;
use super::tokenizer::{Location, Token, TokenType};
use anyhow::{anyhow, Error, Result};

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
        found: String,
    },
    InvalidInteger(String),
    InvalidBoolean(String),
}

const PRECEDENCE_LEVELS: &[&[&str]] = &[
    &["="],
    &["or"],
    &["and"],
    &["==", "!="],
    &["<", "<=", ">", ">="],
    &["+", "-"],
    &["*", "/", "&"],
];

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedToken {
                location,
                expected,
                found,
            } => write!(
                f,
                "Unexpected token at {}\texpected {}, found {}",
                location.to_string(),
                expected,
                found
            ),
            Self::InvalidInteger(i) => write!(f, "{} is not a valid integer", i),
            Self::InvalidBoolean(b) => write!(f, "{} is not a valid boolean", b),
        }
    }
}

impl std::error::Error for ParserError {}

pub struct Parser {
    pos: i32,
    tokens: Vec<Token>,
}

// Implemented Parser as struct for easier tracking of pos and tokens
// TODO:
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
    fn consume_expect(&mut self, exptected: Expected) -> Result<Token, Error> {
        let token: Token = self.peek();

        match exptected {
            Expected::Single(s) => {
                if s != token.text {
                    return Err(anyhow!(ParserError::UnexpectedToken {
                        location: token.loc,
                        expected: s,
                        found: token.text,
                    }));
                }
            }
            Expected::Multiple(v) => {
                if !v.contains(&token.text) {
                    return Err(anyhow!(ParserError::UnexpectedToken {
                        location: token.loc,
                        expected: v.join(", "),
                        found: token.text
                    }));
                }
            }
            Expected::None => {}
        }
        self.pos += 1;
        Ok(token)
    }

    fn consume(&mut self) -> Result<Token, Error> {
        self.consume_expect(Expected::None)
    }

    // Parse the token at current pos into int literal, expects token to be of type Integer
    fn parse_int_literal(&mut self) -> Result<Literal, Error> {
        let token = self.peek();

        match token.token_type {
            TokenType::Integer => {
                // parse int, if it fails, map the error onto ParserError
                let int_val = self
                    .consume()?
                    .text
                    .parse::<i32>()
                    .map_err(|_| anyhow!(ParserError::InvalidInteger(token.text)))?;
                Ok(Literal {
                    value: Some(int_val.into()),
                })
            }
            _ => Err(anyhow!(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::Integer.to_string(),
                found: token.text
            })),
        }
    }

    fn parse_bool_literal(&mut self) -> Result<Literal, Error> {
        let token = self.peek();

        match token.token_type {
            TokenType::Boolean => {
                let bool_val = self
                    .consume()?
                    .text
                    .parse::<bool>()
                    .map_err(|_| anyhow!(ParserError::InvalidBoolean(token.text)))?;
                Ok(Literal {
                    value: Some(bool_val.into()),
                })
            }
            _ => Err(anyhow!(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::Boolean.to_string(),
                found: token.text
            })),
        }
    }

    // Get identifier at current pos, expects token to be identifier
    fn parse_identifier(&mut self) -> Result<Identifier, Error> {
        let token = self.peek();

        match token.token_type {
            TokenType::Identifier => {
                let identifier = self.consume()?;
                Ok(Identifier {
                    name: identifier.text,
                })
            }
            _ => Err(anyhow!(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::Identifier.to_string(),
                found: token.text
            })),
        }
    }

    fn parse_function_call(&mut self, name: Identifier) -> Result<FunctionCall, Error> {
        self.consume_expect(Expected::Single("(".to_string()))?;
        let mut args = vec![];
        while self.peek().text != ")".to_string() {
            args.push(self.parse_expression()?);
            if self.peek().text != ")".to_string() {
                self.consume_expect(Expected::Single(",".to_string()))?;
            }
        }
        self.consume_expect(Expected::Single(")".to_string()))?;
        Ok(FunctionCall { name, args })
    }

    // Parser if statemetn and returns If
    fn parse_if(&mut self) -> Result<Box<dyn Expression>, Error> {
        // Get rid of "if" token, return error if for some reason it has disappeared 😱
        self.consume_expect(Expected::Single("if".to_string()))?;
        // Parse expression that should produce boolean value
        let cond = self.parse_expression()?;
        // Get rid of "then" token
        self.consume_expect(Expected::Single("then".to_string()))?;
        //
        let then = self.parse_expression()?;
        let if_else = if self.peek().text == "else".to_string() {
            self.consume_expect(Expected::Single("else".to_string()))?;
            Some(self.parse_expression()?)
        } else {
            None
        };
        Ok(Box::new(If {
            cond,
            then,
            if_else,
        }))
    }

    fn parse_block(&mut self) -> Result<Box<dyn Expression>, Error> {
        self.consume_expect(Expected::Single("{".to_string()))?;
        let mut statements: Vec<Box<dyn Expression>> = vec![];
        while self.peek().text != "}".to_string() {
            let expr = self.parse_expression()?;
            if self.peek().text == ";".to_string() {
                self.consume_expect(Expected::Single(";".to_string()))?;
                statements.push(expr);
            } else {
                self.consume_expect(Expected::Single("}".to_string()))?;
                return Ok(Box::new(Block {
                    statements,
                    result: expr,
                }));
            }
        }
        self.consume_expect(Expected::Single("}".to_string()))?;
        Ok(Box::new(Block {
            statements,
            result: Box::new(Literal { value: None }),
        }))
    }

    // Get expression inside paranthesis, expects epxression to be wrapped in ()
    fn parse_parenthesized(&mut self) -> Result<Box<dyn Expression>, Error> {
        self.consume_expect(Expected::Single("(".to_string()))?;

        let expr = self.parse_expression()?;
        self.consume_expect(Expected::Single(")".to_string()))?;
        Ok(expr)
    }

    // Get int literal, identifier or epxression in parenthises at current pos
    fn parse_factor(&mut self) -> Result<Box<dyn Expression>, Error> {
        let token: Token = self.peek();

        if token.text == "(" {
            Ok(self.parse_parenthesized()?)
        } else if token.text == "if" {
            Ok(self.parse_if()?)
        } else if token.text == "{" {
            Ok(self.parse_block()?)
        } else {
            match token.token_type {
                TokenType::Integer => Ok(Box::new(self.parse_int_literal()?)),
                TokenType::Identifier => {
                    let identifier = self.parse_identifier()?;
                    // If next token after identifier is "(" then expression has to be function call
                    if self.peek().text == "(".to_string() {
                        Ok(Box::new(self.parse_function_call(identifier)?))
                    } else {
                        Ok(Box::new(identifier))
                    }
                }
                TokenType::Boolean => Ok(Box::new(self.parse_bool_literal()?)),
                _ => Err(anyhow!(ParserError::UnexpectedToken {
                    location: token.loc,
                    expected: "Integer or Identifier".to_string(),
                    found: token.text
                })),
            }
        }
    }

    fn parse_binary_op(&mut self, precedence_level: usize) -> Result<Box<dyn Expression>, Error> {
        if precedence_level >= PRECEDENCE_LEVELS.len() {
            return self.parse_factor();
        }

        let mut left = self.parse_binary_op(precedence_level + 1)?;

        while PRECEDENCE_LEVELS[precedence_level].contains(&self.peek().text.as_str()) {
            let op = self.consume()?.text;
            if op == "=".to_string() {
                let right = self.parse_binary_op(precedence_level)?;
                left = Box::new(BinaryOp { left, op, right });
            } else {
                let right = self.parse_binary_op(precedence_level + 1)?;
                left = Box::new(BinaryOp { left, op, right });
            }
        }

        Ok(left)
    }

    // get expression in for of Expression + Expression or Expression - Expression
    // left assiciative
    fn parse_expression(&mut self) -> Result<Box<dyn Expression>, Error> {
        self.parse_binary_op(0)
    }

    pub fn parse(&mut self) -> Result<Box<dyn Expression>, Error> {
        let expr = self.parse_expression()?;

        // Make sure the whole input has been parsed, if it has then peek() will return End token
        // If not, return error
        println!("{}, {}", self.pos, self.tokens.len());
        let token: Token = self.peek();
        if token.token_type != TokenType::End {
            return Err(anyhow!(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::End.to_string(),
                found: token.text,
            }));
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
            p.consume_expect(Expected::Single(exp_single)).unwrap(),
        );
        assert_eq!(
            Token {
                text: "-".to_string(),
                token_type: TokenType::Operator,
                loc: Location::special(),
            },
            p.consume_expect(Expected::Multiple(exp_mult)).unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_consume_single_panic() {
        let tokens = tokenize("2 - 3 + 1", "file.txt");
        let mut p: Parser = Parser::new(tokens);
        let exp: String = "-".to_string();

        p.consume_expect(Expected::Single(exp)).unwrap();
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

        p.consume_expect(Expected::Multiple(exp)).unwrap();
    }

    #[test]
    fn test_parse_int_literal() {
        let tokens = tokenize("2 + 2", "file.txt");
        let mut p = Parser::new(tokens);
        assert_eq!(
            Literal {
                value: Some(2.into())
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
    fn test_parse_parenthesized() {
        let tokens = tokenize("2 + (3 * 4)", "file.txt");
        let mut p = Parser::new(tokens);

        let bin_op = BinaryOp {
            left: Box::new(Literal {
                value: Some(2.into()),
            }),
            op: "+".to_string(),
            right: Box::new(BinaryOp {
                left: Box::new(Literal {
                    value: Some(3.into()),
                }),
                op: "*".to_string(),
                right: Box::new(Literal {
                    value: Some(4.into()),
                }),
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

    #[test]
    fn test_semicolon() {
        let tokens = tokenize("a + b;", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = BinaryOp {
            left: Box::new(Identifier {
                name: "a".to_string(),
            }),
            op: "+".to_string(),
            right: Box::new(Identifier {
                name: "b".to_string(),
            }),
        };

        assert_eq!(
            expected,
            *p.parse_expression()
                .unwrap()
                .as_any()
                .downcast_ref::<BinaryOp>()
                .unwrap()
        );

        assert_eq!(p.peek().text, ";".to_string());
    }
}
