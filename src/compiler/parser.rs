use std::sync::OnceState;

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
                Ok(Literal::new(int_val, token.loc))
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
                Ok(Literal::new(bool_val, token.loc))
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
                Ok(Identifier::new(identifier.text, token.loc))
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
            args.push(self.parse_binary_op()?);
            if self.peek().text != ")".to_string() {
                self.consume_expect(Expected::Single(",".to_string()))?;
            }
        }
        self.consume_expect(Expected::Single(")".to_string()))?;
        // Ok(FunctionCall { name, args })
        Ok(FunctionCall::new(name, args))
    }

    // Parser if statemetn and returns If
    fn parse_if(&mut self) -> Result<If, Error> {
        let loc = self.peek().loc;
        // Get rid of "if" token, return error if for some reason it has disappeared 😱
        self.consume_expect(Expected::Single("if".to_string()))?;

        // Parse expression that should produce boolean value
        let cond = self.parse_binary_op()?;

        // Get rid of "then" token
        self.consume_expect(Expected::Single("then".to_string()))?;
        //
        let then = self.parse_binary_op()?;
        let if_else = if self.peek().text == "else".to_string() {
            self.consume_expect(Expected::Single("else".to_string()))?;
            Some(self.parse_binary_op()?)
        } else {
            None
        };
        Ok(If::new(cond, then, if_else, loc))
    }

    fn parse_while(&mut self) -> Result<While, Error> {
        let loc = self.peek().loc;
        self.consume_expect(Expected::Single("while".to_string()))?;

        // Get while condition
        let cond = self.parse_binary_op()?;

        // Get while body
        self.consume_expect(Expected::Single("do".to_string()))?;
        let body = self.parse_binary_op()?;

        Ok(While::new(cond, body, loc))
    }

    fn parse_var_declaration(&mut self) -> Result<VarDeclaration, Error> {
        self.consume_expect(Expected::Single("var".to_string()))?;
        let token = self.consume()?;
        if token.token_type != TokenType::Identifier {
            return Err(anyhow!(ParserError::UnexpectedToken {
                location: token.loc,
                expected: TokenType::Identifier.to_string(),
                found: token.token_type.to_string()
            }));
        } else {
            let var = Identifier::new(token.text, token.loc);
            let mut declared_type = None;
            if self.peek().text == ":" {
                self.consume()?;
                let t = self.consume_expect(Expected::Multiple(vec![
                    "Int".to_string(),
                    "Bool".to_string(),
                    "Unit".to_string(),
                ]))?;

                declared_type = if t.text == "Int" {
                    Some(VarType::Int)
                } else if t.text == "Bool" {
                    Some(VarType::Bool)
                } else {
                    // No need to check for Unit anymore, consume_expect does the job already
                    Some(VarType::Unit)
                }
            }

            self.consume_expect(Expected::Single("=".to_string()))?;
            let initializer = self.parse_binary_op()?;

            Ok(VarDeclaration::new(var, declared_type, initializer))
        }
    }

    fn parse_block(&mut self) -> Result<Box<dyn Expression>, Error> {
        let start_loc = self.peek().loc;
        self.consume_expect(Expected::Single("{".to_string()))?;
        let mut statements: Vec<Box<dyn Expression>> = vec![];
        while self.peek().text != "}".to_string() {
            let expression = self.parse_expression()?;
            let next_token = self.peek();

            if next_token.token_type == TokenType::Punctuation && next_token.text == ";" {
                self.consume()?;
                statements.push(expression);
            } else if next_token.token_type == TokenType::Punctuation && next_token.text == "}" {
                self.consume()?;
                statements.push(expression);
                return Ok(Box::new(Block::new(statements, start_loc)));
            } else if expression.is_block() {
                statements.push(expression);
            } else {
                return Err(anyhow!(ParserError::UnexpectedToken {
                    location: next_token.loc,
                    expected: "}".to_string(),
                    found: next_token.text
                }));
            }
        }
        let end_loc = self.peek().loc;
        self.consume_expect(Expected::Single("}".to_string()))?;
        statements.push(Box::new(Literal::none(end_loc)));
        Ok(Box::new(Block::new(statements, start_loc)))
    }

    // Get expression inside paranthesis, expects epxression to be wrapped in ()
    fn parse_parenthesized(&mut self) -> Result<Box<dyn Expression>, Error> {
        self.consume_expect(Expected::Single("(".to_string()))?;

        let expr = self.parse_binary_op()?;
        self.consume_expect(Expected::Single(")".to_string()))?;
        Ok(expr)
    }

    fn parse_unary_op(&mut self) -> Result<UnaryOp, Error> {
        let loc = self.peek().loc;
        let op = self
            .consume_expect(Expected::Multiple(vec!["not".to_string(), "-".to_string()]))?
            .text;
        let right = self.parse_factor()?;
        let result = UnaryOp::new(op, right, loc);
        Ok(result)
    }

    // Get int literal, identifier or epxression in parenthises at current pos
    fn parse_factor(&mut self) -> Result<Box<dyn Expression>, Error> {
        let token: Token = self.peek();
        if token.text == "var" {
            Err(anyhow!(
                "{}: \"var\" is only allowed directly inside blocks and in top-level expressions",
                token.loc.to_string()
            ))
        } else if token.text == "-" || token.text == "not" {
            Ok(Box::new(self.parse_unary_op()?))
        } else if token.text == "(" {
            Ok(self.parse_parenthesized()?)
        } else if token.text == "if" {
            Ok(Box::new(self.parse_if()?))
        } else if token.text == "while" {
            Ok(Box::new(self.parse_while()?))
        } else if token.text == "{" {
            Ok(self.parse_block()?)
        } else {
            match token.token_type {
                TokenType::Integer => Ok(Box::new(self.parse_int_literal()?)),
                TokenType::Identifier => {
                    let identifier = self.parse_identifier()?;
                    // If next token after identifier is "(" then expression has to be function call
                    if self.peek().text == "(" {
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

    fn parse_binary_op(&mut self) -> Result<Box<dyn Expression>, Error> {
        let mut expressions = vec![self.parse_factor()?];
        let mut operators: Vec<String> = vec![];

        while let Some(op) = self.peek_op() {
            let curr_precedence = self.get_op_precedence(&op);

            self.consume()?;

            let right = self.parse_factor()?;

            while !operators.is_empty() {
                let prev_precedence = self.get_op_precedence(&operators[operators.len() - 1]);

                // Left associativity for all operators except "="
                if curr_precedence < prev_precedence
                    || (curr_precedence == prev_precedence && op != "=")
                {
                    // Get right and left expressions from stack + operator from stack
                    // If missing, turn option into error result and pass it along to caller
                    let right = expressions
                        .pop()
                        .ok_or_else(|| anyhow!("Missing right expression of bin op"))?;
                    let left = expressions
                        .pop()
                        .ok_or_else(|| anyhow!("Missing left expression of bin op"))?;
                    let op = operators
                        .pop()
                        .ok_or_else(|| anyhow!("Missing operator of bin op"))?;

                    expressions.push(Box::new(BinaryOp::new(left, op, right)));
                } else {
                    break;
                }
            }

            operators.push(op);
            expressions.push(right);
        }

        while let Some(op) = operators.pop() {
            let right = expressions
                .pop()
                .ok_or_else(|| anyhow!("Missing right expression of bin op"))?;
            let left = expressions
                .pop()
                .ok_or_else(|| anyhow!("Missing left expression of bin op"))?;
            expressions.push(Box::new(BinaryOp::new(left, op, right)));
        }

        expressions
            .pop()
            .ok_or_else(|| anyhow!("Missing expression"))
    }

    fn peek_op(&mut self) -> Option<String> {
        let token = self.peek();
        match token.token_type {
            TokenType::Operator => Some(token.text),
            _ => None,
        }
    }

    fn get_op_precedence(&self, op: &str) -> usize {
        for (precedence_level, ops) in PRECEDENCE_LEVELS.iter().enumerate() {
            if ops.contains(&op) {
                return precedence_level;
            }
        }
        PRECEDENCE_LEVELS.len()
    }

    fn parse_expression(&mut self) -> Result<Box<dyn Expression>, Error> {
        let token = self.peek().text;
        if token == "var" {
            Ok(Box::new(self.parse_var_declaration()?))
        } else if token == "if" {
            Ok(Box::new(self.parse_if()?))
        } else if token == "while" {
            Ok(Box::new(self.parse_while()?))
        } else if token == "{" {
            Ok(self.parse_block()?)
        } else {
            self.parse_binary_op()
        }
    }

    fn parse_top_level_statements(
        &mut self,
        initializer: Box<dyn Expression>,
    ) -> Result<Vec<Box<dyn Expression>>, Error> {
        if !initializer.is_block() {
            self.consume_expect(Expected::Single(";".to_string()))?;
        }
        let mut statements: Vec<Box<dyn Expression>> = vec![initializer];
        while self.peek().token_type != TokenType::End {
            let expression = self.parse_expression()?;
            let next_token = self.peek();

            if next_token.text == ";" {
                self.consume()?;
                statements.push(expression);
            } else if next_token.token_type == TokenType::End {
                statements.push(expression);
                return Ok(statements);
            } else if expression.is_block() {
                statements.push(expression);
            } else {
                return Err(anyhow!(ParserError::UnexpectedToken {
                    location: next_token.loc,
                    expected: ";".to_string(),
                    found: next_token.text
                }));
            }
        }
        statements.push(Box::new(Literal::none(self.peek().loc)));
        Ok(statements)
    }

    pub fn parse(&mut self) -> Result<Box<dyn Expression>, Error> {
        let start_loc = self.peek().loc;
        let expr = self.parse_expression()?;
        let token = self.peek();

        if token.token_type != TokenType::End {
            let statements = self.parse_top_level_statements(expr)?;
            Ok(Box::new(Block::new(statements, start_loc)))
        } else {
            Ok(expr)
        }
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
                value: Some(2.into()),
                loc: Location::special()
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
                name: "a".to_string(),
                loc: Location::special()
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
                loc: Location::special(),
            }),
            op: "+".to_string(),
            right: Box::new(BinaryOp {
                left: Box::new(Literal {
                    value: Some(3.into()),
                    loc: Location::special(),
                }),
                op: "*".to_string(),
                right: Box::new(Literal {
                    value: Some(4.into()),
                    loc: Location::special(),
                }),
                loc: Location::special(),
            }),
            loc: Location::special(),
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
                loc: Location::special(),
            }),
            op: "+".to_string(),
            right: Box::new(Identifier {
                name: "b".to_string(),
                loc: Location::special(),
            }),
            loc: Location::special(),
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

    #[test]
    fn test_parse_unary() {
        let tokens = tokenize("not x", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = UnaryOp::new(
            "not",
            Box::new(Identifier::new("x", Location::special())),
            Location::special(),
        );

        assert_eq!(
            expected,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<UnaryOp>()
                .unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_identifier_panic() {
        let tokens = tokenize("1abc = 10", "file.txt");
        let mut p = Parser::new(tokens);

        p.parse().unwrap();
    }
}
