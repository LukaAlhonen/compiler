use compiler::compiler::ast::*;
use compiler::compiler::parser::Parser;
use compiler::compiler::tokenizer::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = tokenize("2 + 1 - 3", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        let bin_op = BinaryOp {
            left: Box::new(BinaryOp {
                left: Box::new(Literal { value: 2.into() }),
                op: "+".to_string(),
                right: Box::new(Literal { value: 1.into() }),
            }),
            op: "-".to_string(),
            right: Box::new(Literal { value: 3.into() }),
        };

        assert_eq!(
            bin_op,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<BinaryOp>()
                .unwrap()
        );
    }

    #[test]
    fn test_parse_mult() {
        let tokens = tokenize("2 + 4 * 3", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        let expected = BinaryOp {
            left: Box::new(Literal { value: 2.into() }),
            op: "+".to_string(),
            right: Box::new(BinaryOp {
                left: Box::new(Literal { value: 4.into() }),
                op: "*".to_string(),
                right: Box::new(Literal { value: 3.into() }),
            }),
        };

        assert_eq!(
            expected,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<BinaryOp>()
                .unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_empty() {
        let tokens = tokenize("", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        p.parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_panic() {
        let tokens = tokenize("a * 2 c", "file.txt");
        let mut p: Parser = Parser::new(tokens.clone());

        p.parse().unwrap();
    }
}
