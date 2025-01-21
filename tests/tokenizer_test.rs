use compiler::compiler::tokenizer::tokenize;
use compiler::compiler::token::{Token, TokenType};
use compiler::compiler::location::Location as L;

#[cfg(test)]
mod test {
    use super::*; 

    #[test]
    fn test_tokenize() {
        assert_eq!(
            tokenize(String::from("aaa 123 bbb")),
            vec![
                Token::new("aaa", TokenType::Identifier, L::special()),
                Token::new("123", TokenType::Integer, L::special()),
                Token::new("bbb", TokenType::Identifier, L::special())
            ]
        )
    }

    #[test]
    fn test_tokenize_comment() {
        assert_eq!(
            tokenize(String::from("// assign value to var\na = 1 # I like this variable\nb = 2\n# addition\nc = a + b //place result in c")),
            vec![
                Token::new("a", TokenType::Identifier, L::special()),
                Token::new("=", TokenType::Operator, L::special()),
                Token::new("1", TokenType::Integer, L::special()),
                Token::new("b", TokenType::Identifier, L::special()),
                Token::new("=", TokenType::Operator, L::special()),
                Token::new("2", TokenType::Integer, L::special()),
                Token::new("c", TokenType::Identifier, L::special()),
                Token::new("=", TokenType::Operator, L::special()),
                Token::new("a", TokenType::Identifier, L::special()),
                Token::new("+", TokenType::Operator, L::special()),
                Token::new("b", TokenType::Identifier, L::special()),
            ]
        )
    }

    #[test]
    fn test_tokenize_for_loop() {
        let source_code = String::from(
            "
        for (int i = 0; i < 10; ++i) {
            print(i);
        }
            ",
        );

        assert_eq!(
            tokenize(source_code),
            vec![
                Token::new("for", TokenType::Identifier, L::special()),
                Token::new("(", TokenType::Punctuation, L::special()),
                Token::new("int", TokenType::Identifier, L::special()),
                Token::new("i", TokenType::Identifier, L::special()),
                Token::new("=", TokenType::Operator, L::special()),
                Token::new("0", TokenType::Integer, L::special()),
                Token::new(";", TokenType::Punctuation, L::special()),
                Token::new("i", TokenType::Identifier, L::special()),
                Token::new("<", TokenType::Operator, L::special()),
                Token::new("10", TokenType::Integer, L::special()),
                Token::new(";", TokenType::Punctuation, L::special()),
                Token::new("+", TokenType::Operator, L::special()),
                Token::new("+", TokenType::Operator, L::special()),
                Token::new("i", TokenType::Identifier, L::special()),
                Token::new(")", TokenType::Punctuation, L::special()),
                Token::new("{", TokenType::Punctuation, L::special()),
                Token::new("print", TokenType::Identifier, L::special()),
                Token::new("(", TokenType::Punctuation, L::special()),
                Token::new("i", TokenType::Identifier, L::special()),
                Token::new(")", TokenType::Punctuation, L::special()),
                Token::new(";", TokenType::Punctuation, L::special()),
                Token::new("}", TokenType::Punctuation, L::special()),
            ]
        )
    }

    #[test]
    fn test_tokenize_operators() {
        let source_code = String::from("* + - / == != <= >= = > <");

        assert_eq!(
            tokenize(source_code),
            vec![
                Token::new("*", TokenType::Operator, L::special()),
                Token::new("+", TokenType::Operator, L::special()),
                Token::new("-", TokenType::Operator, L::special()),
                Token::new("/", TokenType::Operator, L::special()),
                Token::new("==", TokenType::Operator, L::special()),
                Token::new("!=", TokenType::Operator, L::special()),
                Token::new("<=", TokenType::Operator, L::special()),
                Token::new(">=", TokenType::Operator, L::special()),
                Token::new("=", TokenType::Operator, L::special()),
                Token::new(">", TokenType::Operator, L::special()),
                Token::new("<", TokenType::Operator, L::special()),
            ]
        );
    }
}
