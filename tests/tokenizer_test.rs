use compiler::compiler::tokenizer::{tokenize, Location as L, Token, TokenType};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(
            tokenize("aaa 123 bbb", "file.txt"),
            vec![
                Token {
                    text: "aaa".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "123".to_string(),
                    token_type: TokenType::Integer,
                    loc: L::special()
                },
                Token {
                    text: "bbb".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                }
            ]
        )
    }

    #[test]
    fn test_tokenize_comment() {
        assert_eq!(
            tokenize("// assign value to var\na = 1 # I like this variable\nb = 2\n# addition\nc = a + b //place result in c", "file.txt"),
            vec![
                Token {
                    text: "a".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "1".to_string(),
                    token_type: TokenType::Integer,
                    loc: L::special()
                },
                Token {
                    text: "b".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "2".to_string(),
                    token_type: TokenType::Integer,
                    loc: L::special()
                },
                Token {
                    text: "c".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "a".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "+".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "b".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
            ]
        )
    }

    #[test]
    fn test_tokenize_for_loop() {
        let source_code = "
        for (int i = 0; i < 10; ++i) {
            print(i);
        }
            ";

        assert_eq!(
            tokenize(source_code, "file.txt"),
            vec![
                Token {
                    text: "for".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "(".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: "int".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "i".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "0".to_string(),
                    token_type: TokenType::Integer,
                    loc: L::special()
                },
                Token {
                    text: ";".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: "i".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "<".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "10".to_string(),
                    token_type: TokenType::Integer,
                    loc: L::special()
                },
                Token {
                    text: ";".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: "+".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "+".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "i".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: ")".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: "{".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: "print".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: "(".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: "i".to_string(),
                    token_type: TokenType::Identifier,
                    loc: L::special()
                },
                Token {
                    text: ")".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: ";".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
                Token {
                    text: "}".to_string(),
                    token_type: TokenType::Punctuation,
                    loc: L::special()
                },
            ]
        )
    }

    #[test]
    fn test_tokenize_operators() {
        let source_code = "* + - / == != <= >= = > <";

        assert_eq!(
            tokenize(source_code, "file.txt"),
            vec![
                Token {
                    text: "*".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "+".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "-".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "/".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "==".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "!=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "<=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: ">=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "=".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: ">".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
                Token {
                    text: "<".to_string(),
                    token_type: TokenType::Operator,
                    loc: L::special()
                },
            ]
        );
    }

    #[test]
    fn test_tokenize_typed_var_declaration() {
        let source_code = "var x: Int = 1";
        let expected = vec![
            Token {
                text: "var".to_string(),
                token_type: TokenType::Identifier,
                loc: L::special(),
            },
            Token {
                text: "x".to_string(),
                token_type: TokenType::Identifier,
                loc: L::special(),
            },
            Token {
                text: ":".to_string(),
                token_type: TokenType::Punctuation,
                loc: L::special(),
            },
            Token {
                text: "Int".to_string(),
                token_type: TokenType::Identifier,
                loc: L::special(),
            },
            Token {
                text: "=".to_string(),
                token_type: TokenType::Operator,
                loc: L::special(),
            },
            Token {
                text: "1".to_string(),
                token_type: TokenType::Integer,
                loc: L::special(),
            },
        ];

        assert_eq!(tokenize(source_code, "file.txt"), expected);
    }
}
