use compiler::compiler::ast::*;
use compiler::compiler::parser::Parser;
use compiler::compiler::tokenizer::*;
use compiler::compiler::types::Type;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let tokens = tokenize("2 + 1 - 3", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        let expected = BinaryOp::new(
            Box::new(BinaryOp::new(
                Box::new(Literal::new(2, Location::special())),
                "+",
                Box::new(Literal::new(1, Location::special())),
            )),
            "-",
            Box::new(Literal::new(3, Location::special())),
        );

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
    fn test_parse_mult() {
        let tokens = tokenize("2 + 4 * 3", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        let expected = BinaryOp::new(
            Box::new(Literal::new(2, Location::special())),
            "+",
            Box::new(BinaryOp::new(
                Box::new(Literal::new(4, Location::special())),
                "*",
                Box::new(Literal::new(3, Location::special())),
            )),
        );

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
        let mut p: Parser = Parser::new(tokens);

        p.parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_parse_comma_panic() {
        let tokens = tokenize("a, b, c", "file.txt");
        let mut p = Parser::new(tokens);

        p.parse().unwrap();
    }

    #[test]
    fn test_parse_if() {
        let tokens = tokenize("if a then b + c", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        let expected = If::new(
            // cond
            Box::new(Identifier::new("a", Location::special())),
            // then
            Box::new(BinaryOp::new(
                Box::new(Identifier::new("b", Location::special())), // left
                "+",                                                 // op
                Box::new(Identifier::new("c", Location::special())), // right
            )),
            None,                // else_branch
            Location::special(), // loc
        );

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<If>().unwrap()
        );
    }

    #[test]
    fn test_parse_if_else() {
        let tokens = tokenize("if a then b + c else b * c", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        let expected = If::new(
            // cond
            Box::new(Identifier::new("a", Location::special())),
            // then
            Box::new(BinaryOp::new(
                Box::new(Identifier::new("b", Location::special())), // left
                "+",                                                 // op
                Box::new(Identifier::new("c", Location::special())), // right
            )),
            // if_else
            Some(Box::new(BinaryOp::new(
                Box::new(Identifier::new("b", Location::special())), // left
                "*",                                                 // op
                Box::new(Identifier::new("c", Location::special())), // right
            ))),
            Location::special(), // loc
        );

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<If>().unwrap()
        );
    }

    #[test]
    fn test_parse_embedded_if() {
        let tokens = tokenize("1 + if true then 2 else 3", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = BinaryOp::new(
            Box::new(Literal::new(1, Location::special())), // left
            "+",                                            // op
            // right
            Box::new(If::new(
                Box::new(Literal::new(true, Location::special())), // cond
                Box::new(Literal::new(2, Location::special())),    // then
                Some(Box::new(Literal::new(3, Location::special()))), // else_branch
                Location::special(),                               // loc
            )),
        );
        let parsed = p.parse().unwrap();

        assert_eq!(
            expected,
            *parsed.as_any().downcast_ref::<BinaryOp>().unwrap()
        )
    }

    #[test]
    #[should_panic]
    fn test_parse_if_panic() {
        let tokens = tokenize("1 + if true then", "file.txt");
        let mut p = Parser::new(tokens);

        p.parse().unwrap();
    }

    #[test]
    fn test_parse_nested_if() {
        let tokens = tokenize("if a then if b then c else d else e", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = If::new(
            Box::new(Identifier::new("a", Location::special())), // cond
            // then
            Box::new(If::new(
                Box::new(Identifier::new("b", Location::special())), // cond
                Box::new(Identifier::new("c", Location::special())), // then
                Some(Box::new(Identifier::new("d", Location::special()))), // else_branch
                Location::special(),
            )),
            Some(Box::new(Identifier::new("e", Location::special()))), // else_branch
            Location::special(),                                       // loc
        );

        let parsed = p.parse().unwrap();

        assert_eq!(expected, *parsed.as_any().downcast_ref::<If>().unwrap());
    }

    #[test]
    fn test_parse_function() {
        let tokens1 = tokenize("a(1, 2, 3)", "file.txt");
        let tokens2 = tokenize("b(c, d, e)", "file.txt");
        let tokens3 = tokenize("c()", "file.txt");
        let tokens4 = tokenize("d(1 + 2, if true then 3 else 1)", "file.txt");

        let mut p = Parser::new(tokens1);
        let expected1 = FunctionCall::new(
            Identifier::new("a", Location::special()), // name
            // args
            vec![
                Box::new(Literal::new(1, Location::special())),
                Box::new(Literal::new(2, Location::special())),
                Box::new(Literal::new(3, Location::special())),
            ],
        );
        assert_eq!(
            expected1,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<FunctionCall>()
                .unwrap()
        );

        p = Parser::new(tokens2);
        let expected2 = FunctionCall::new(
            Identifier::new("b", Location::special()), // name
            // args
            vec![
                Box::new(Identifier::new("c", Location::special())),
                Box::new(Identifier::new("d", Location::special())),
                Box::new(Identifier::new("e", Location::special())),
            ],
        );
        assert_eq!(
            expected2,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<FunctionCall>()
                .unwrap()
        );

        p = Parser::new(tokens3);
        let expected3 = FunctionCall::new(
            Identifier::new("c", Location::special()), // name
            vec![],                                    // args
        );
        assert_eq!(
            expected3,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<FunctionCall>()
                .unwrap()
        );

        p = Parser::new(tokens4);
        let expected4 = FunctionCall::new(
            Identifier::new("d", Location::special()), // name
            // args
            vec![
                Box::new(BinaryOp::new(
                    Box::new(Literal::new(1, Location::special())), // left
                    "+",                                            // right
                    Box::new(Literal::new(2, Location::special())), // right
                )),
                Box::new(If::new(
                    Box::new(Literal::new(true, Location::special())), // cond
                    Box::new(Literal::new(3, Location::special())),    // then
                    Some(Box::new(Literal::new(1, Location::special()))), // else_branch
                    Location::special(),                               // loc
                )),
            ],
        );
        assert_eq!(
            expected4,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<FunctionCall>()
                .unwrap()
        );
    }

    #[test]
    fn test_parse_or() {
        let tokens = tokenize("true or false", "file.txt");

        let mut p: Parser = Parser::new(tokens);

        let expected = BinaryOp::new(
            Box::new(Literal::new(true, Location::special())), // left
            "or",                                              // op
            Box::new(Literal::new(false, Location::special())), // right
        );

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
    fn test_if_or() {
        let tokens = tokenize(
            "if a == 2 + 2 or b == 2 + 2 then a + 2 else b + 2",
            "file.txt",
        );

        let mut p: Parser = Parser::new(tokens);

        let expected = If::new(
            // cond
            Box::new(BinaryOp::new(
                // left
                Box::new(BinaryOp::new(
                    Box::new(Identifier::new("a", Location::special())), // left
                    "==",                                                // op
                    // right
                    Box::new(BinaryOp::new(
                        Box::new(Literal::new(2, Location::special())), // left
                        "+",                                            // op
                        Box::new(Literal::new(2, Location::special())), // right
                    )),
                )),
                "or", // op
                // right
                Box::new(BinaryOp::new(
                    Box::new(Identifier::new("b", Location::special())), // left
                    "==",                                                // op
                    // right
                    Box::new(BinaryOp::new(
                        Box::new(Literal::new(2, Location::special())), // left
                        "+",                                            // op
                        Box::new(Literal::new(2, Location::special())), // right
                    )),
                )),
            )),
            // then
            Box::new(BinaryOp::new(
                Box::new(Identifier::new("a", Location::special())), // left
                "+",                                                 // op
                Box::new(Literal::new(2, Location::special())),      // right
            )),
            // else_branch
            Some(Box::new(BinaryOp::new(
                Box::new(Identifier::new("b", Location::special())), // left
                "+",                                                 // op
                Box::new(Literal::new(2, Location::special())),      // right
            ))),
            Location::special(), // loc
        );

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<If>().unwrap()
        );
    }

    #[test]
    fn test_binary_op_precedence() {
        let tokens = tokenize("a = 2 + 3 * 4", "file.txt");
        let mut p: Parser = Parser::new(tokens);
        let expected = BinaryOp::new(
            Box::new(Identifier::new("a", Location::special())), // left
            "=",                                                 // op
            // right
            Box::new(BinaryOp::new(
                Box::new(Literal::new(2, Location::special())), // left
                "+",                                            // op
                // right
                Box::new(BinaryOp::new(
                    Box::new(Literal::new(3, Location::special())), // left
                    "*",                                            // op
                    Box::new(Literal::new(4, Location::special())), // right
                )),
            )),
        );

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
    fn test_assignment() {
        let tokens = tokenize("a = b = c", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = BinaryOp::new(
            Box::new(Identifier::new("a", Location::special())), // left
            "=",                                                 // op
            // right
            Box::new(BinaryOp::new(
                Box::new(Identifier::new("b", Location::special())), // left
                "=",                                                 // op
                Box::new(Identifier::new("c", Location::special())), // right
            )),
        );

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
    fn test_parse_block() {
        let tokens1 = tokenize("{a + b; a}", "file.txt");
        let tokens2 = tokenize("{a + b; a;}", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut p2 = Parser::new(tokens2);

        let expected1 = Block::new(
            // statements
            vec![
                Box::new(BinaryOp::new(
                    Box::new(Identifier::new("a", Location::special())), // left
                    "+",                                                 // op
                    Box::new(Identifier::new("b", Location::special())), // right
                )),
                Box::new(Identifier::new("a", Location::special())), // result
            ],
            Location::special(), // loc
        );
        let expected2 = Block::new(
            // statements
            vec![
                Box::new(BinaryOp::new(
                    Box::new(Identifier::new("a", Location::special())), // left
                    "+",                                                 // op
                    Box::new(Identifier::new("b", Location::special())), // right
                )),
                Box::new(Identifier::new("a", Location::special())),
                Box::new(Literal::none(Location::special())), // result
            ],
            Location::special(), // loc
        );

        assert_eq!(
            expected1,
            *p1.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<Block>()
                .unwrap()
        );

        assert_eq!(
            expected2,
            *p2.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<Block>()
                .unwrap()
        );
    }

    #[test]
    fn test_parse_large_block() {
        let tokens = tokenize(
            "{
                while f() do {
                    x = 10;
                    y = if g(x) then {
                        x = x + 1;
                        x
                    } else {
                        g(x)
                    }
                    g(y);
                }
                123
            }",
            "file.txt",
        );
        let mut p = Parser::new(tokens);

        let expected = Block::new(
            vec![
                Box::new(While::new(
                    Box::new(FunctionCall::new(
                        Identifier::new("f", Location::special()),
                        vec![],
                    )),
                    Box::new(Block::new(
                        vec![
                            Box::new(BinaryOp::new(
                                Box::new(Identifier::new("x", Location::special())),
                                "=",
                                Box::new(Literal::new(10, Location::special())),
                            )),
                            Box::new(BinaryOp::new(
                                Box::new(Identifier::new("y", Location::special())),
                                "=",
                                Box::new(If::new(
                                    Box::new(FunctionCall::new(
                                        Identifier::new("g", Location::special()),
                                        vec![Box::new(Identifier::new("x", Location::special()))],
                                    )),
                                    Box::new(Block::new(
                                        vec![
                                            Box::new(BinaryOp::new(
                                                Box::new(Identifier::new("x", Location::special())),
                                                "=",
                                                Box::new(BinaryOp::new(
                                                    Box::new(Identifier::new(
                                                        "x",
                                                        Location::special(),
                                                    )),
                                                    "+",
                                                    Box::new(Literal::new(1, Location::special())),
                                                )),
                                            )),
                                            Box::new(Identifier::new("x", Location::special())),
                                        ],
                                        Location::special(),
                                    )),
                                    Some(Box::new(Block::new(
                                        vec![Box::new(FunctionCall::new(
                                            Identifier::new("g", Location::special()),
                                            vec![Box::new(Identifier::new(
                                                "x",
                                                Location::special(),
                                            ))],
                                        ))],
                                        Location::special(),
                                    ))),
                                    Location::special(),
                                )),
                            )),
                            Box::new(FunctionCall::new(
                                Identifier::new("g", Location::special()),
                                vec![Box::new(Identifier::new("y", Location::special()))],
                            )),
                            Box::new(Literal::none(Location::special())),
                        ],
                        Location::special(),
                    )),
                    Location::special(),
                )),
                Box::new(Literal::new(123, Location::special())),
            ],
            Location::special(),
        );

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<Block>().unwrap()
        );
    }

    #[test]
    fn test_var_declaration() {
        let tokens1 = tokenize("var x = 1", "file.txt");
        let mut p1 = Parser::new(tokens1);

        let tokens2 = tokenize("{ var x = 1; }", "file.txt");
        let mut p2 = Parser::new(tokens2);

        let expected1 = VarDeclaration::new(
            Identifier::new("x", Location::special()),
            None,
            Box::new(Literal::new(1, Location::special())),
        );
        let expected2 = Block::new(
            vec![
                Box::new(VarDeclaration::new(
                    Identifier::new("x", Location::special()),
                    None,
                    Box::new(Literal::new(1, Location::special())),
                )),
                Box::new(Literal::none(Location::special())),
            ],
            Location::special(),
        );

        assert_eq!(
            expected1,
            *p1.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<VarDeclaration>()
                .unwrap()
        );
        assert_eq!(
            expected2,
            *p2.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<Block>()
                .unwrap()
        );
    }

    #[test]
    fn test_var_declaration_panic() {
        let tokens1 = tokenize("if var x = 1 then 2", "file.txt");
        let tokens2 = tokenize("a + var x = 1", "file.txt");
        let tokens3 = tokenize("while var x = 1 do y", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut p2 = Parser::new(tokens2);
        let mut p3 = Parser::new(tokens3);

        let result1 = p1.parse();
        let result2 = p2.parse();
        let result3 = p3.parse();
        assert!(result1.is_err(), "expected error but got: {:?}", result1);
        assert!(result2.is_err(), "expected error but got: {:?}", result2);
        assert!(result3.is_err(), "expected error but got: {:?}", result3);
    }

    #[test]
    fn test_parse_while() {
        let tokens = tokenize("while x do y", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = While::new(
            Box::new(Identifier::new("x", Location::special())),
            Box::new(Identifier::new("y", Location::special())),
            Location::special(),
        );

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<While>().unwrap()
        );
    }

    #[test]
    fn test_block_cases() {
        let tokens1 = tokenize("{ { a } { b } }", "file.txt"); // yes
        let tokens2 = tokenize("{ a b }", "file.txt"); // no
        let tokens3 = tokenize("{ if true then { a } b }", "file.txt"); // yes
        let tokens4 = tokenize("{ if true then { a }; b }", "file.txt"); // yes
        let tokens5 = tokenize("{ if true then { a } b c }", "file.txt"); // no
        let tokens6 = tokenize("{ if true then { a } b; c }", "file.txt"); // yes
        let tokens7 = tokenize("{ if true then { a } else { b } c }", "file.txt"); // yes
        let tokens8 = tokenize("x = { { f(a) } { b } }", "file.txt"); // yes
        let tokens9 = tokenize("{ while true do { f(g) } g }", "file.txt");

        let mut p1 = Parser::new(tokens1);
        let mut p2 = Parser::new(tokens2);
        let mut p3 = Parser::new(tokens3);
        let mut p4 = Parser::new(tokens4);
        let mut p5 = Parser::new(tokens5);
        let mut p6 = Parser::new(tokens6);
        let mut p7 = Parser::new(tokens7);
        let mut p8 = Parser::new(tokens8);
        let mut p9 = Parser::new(tokens9);

        let result1 = p1.parse();
        let result2 = p2.parse();
        let result3 = p3.parse();
        let result4 = p4.parse();
        let result5 = p5.parse();
        let result6 = p6.parse();
        let result7 = p7.parse();
        let result8 = p8.parse();
        let result9 = p9.parse();

        assert!(
            result1.is_ok(),
            "1: expected expression but got {:?}",
            result1
        );
        assert!(result2.is_err(), "2: expected error but got {:?}", result2);
        assert!(
            result3.is_ok(),
            "3: expected expression but got {:?}",
            result3
        );
        assert!(
            result4.is_ok(),
            "4: expected expression but got {:?}",
            result4
        );
        assert!(result5.is_err(), "5: expected error but got {:?}", result5);
        assert!(
            result6.is_ok(),
            "6: expected expression but got {:?}",
            result6
        );
        assert!(
            result7.is_ok(),
            "7: expected expression but got {:?}",
            result7
        );
        assert!(
            result8.is_ok(),
            "8: expected expression but got {:?}",
            result8
        );
        assert!(
            result9.is_ok(),
            "9: expected expression but got {:?}",
            result9
        );
    }

    #[test]
    fn test_parse_unary() {
        let tokens1 = tokenize("-1 and not x", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let expected1 = BinaryOp::new(
            Box::new(UnaryOp::new(
                "-",
                Box::new(Literal::new(1, Location::special())),
                Location::special(),
            )),
            "and",
            Box::new(UnaryOp::new(
                "not",
                Box::new(Identifier::new("x", Location::special())),
                Location::special(),
            )),
        );

        let tokens2 = tokenize("while not true do false", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let expected2 = While::new(
            Box::new(UnaryOp::new(
                "not",
                Box::new(Literal::new(true, Location::special())),
                Location::special(),
            )),
            Box::new(Literal::new(false, Location::special())),
            Location::special(),
        );

        let tokens3 = tokenize("if not true then x", "filte.txt");
        let mut p3 = Parser::new(tokens3);
        let expected3 = If::new(
            Box::new(UnaryOp::new(
                "not",
                Box::new(Literal::new(true, Location::special())),
                Location::special(),
            )),
            Box::new(Identifier::new("x", Location::special())),
            None,
            Location::special(),
        );

        let tokens4 = tokenize("not { true or false }", "file.txt");
        let mut p4 = Parser::new(tokens4);
        let expected4 = UnaryOp::new(
            "not",
            Box::new(Block::new(
                vec![Box::new(BinaryOp::new(
                    Box::new(Literal::new(true, Location::special())),
                    "or",
                    Box::new(Literal::new(false, Location::special())),
                ))],
                Location::special(),
            )),
            Location::special(),
        );

        let tokens5 = tokenize("-1 - -1", "file.txt");
        let mut p5 = Parser::new(tokens5);
        let expected5 = BinaryOp::new(
            Box::new(UnaryOp::new(
                "-",
                Box::new(Literal::new(1, Location::special())),
                Location::special(),
            )),
            "-",
            Box::new(UnaryOp::new(
                "-",
                Box::new(Literal::new(1, Location::special())),
                Location::special(),
            )),
        );

        // test1
        assert_eq!(
            expected1,
            *p1.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<BinaryOp>()
                .unwrap()
        );

        // test2
        assert_eq!(
            expected2,
            *p2.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<While>()
                .unwrap()
        );

        // test3
        assert_eq!(
            expected3,
            *p3.parse().unwrap().as_any().downcast_ref::<If>().unwrap()
        );

        // test4
        assert_eq!(
            expected4,
            *p4.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<UnaryOp>()
                .unwrap()
        );

        // test5
        assert_eq!(
            expected5,
            *p5.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<BinaryOp>()
                .unwrap()
        )
    }

    #[test]
    fn test_parse_typed_var_declaration() {
        let tokens1 = tokenize("var a: Int = 1", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let expected1 = VarDeclaration::new(
            Identifier::new("a", Location::special()),
            Some(Type::Int),
            Box::new(Literal::new(1, Location::special())),
        );

        let tokens2 = tokenize("var b: Bool = true", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let expected2 = VarDeclaration::new(
            Identifier::new("b", Location::special()),
            Some(Type::Bool),
            Box::new(Literal::new(true, Location::special())),
        );

        let tokens3 = tokenize("var c: Unit = {1 + 1}", "file.txt");
        let mut p3 = Parser::new(tokens3);
        let expected3 = VarDeclaration::new(
            Identifier::new("c", Location::special()),
            Some(Type::Unit),
            Box::new(Block::new(
                vec![Box::new(BinaryOp::new(
                    Box::new(Literal::new(1, Location::special())),
                    "+",
                    Box::new(Literal::new(1, Location::special())),
                ))],
                Location::special(),
            )),
        );

        // test1
        assert_eq!(
            expected1,
            *p1.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<VarDeclaration>()
                .unwrap()
        );

        // test2
        assert_eq!(
            expected2,
            *p2.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<VarDeclaration>()
                .unwrap()
        );

        // test3
        assert_eq!(
            expected3,
            *p3.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<VarDeclaration>()
                .unwrap()
        )
    }

    #[test]
    fn parse_top_level_block() {
        let tokens1 = tokenize(
            "
            1 + 1;
            2+2;
            {3+3}",
            "file.txt",
        );
        let mut p1 = Parser::new(tokens1);
        let expected1 = Block::new(
            vec![
                Box::new(BinaryOp::new(
                    Box::new(Literal::new(1, Location::special())),
                    "+",
                    Box::new(Literal::new(1, Location::special())),
                )),
                Box::new(BinaryOp::new(
                    Box::new(Literal::new(2, Location::special())),
                    "+",
                    Box::new(Literal::new(2, Location::special())),
                )),
                Box::new(Block::new(
                    vec![Box::new(BinaryOp::new(
                        Box::new(Literal::new(3, Location::special())),
                        "+",
                        Box::new(Literal::new(3, Location::special())),
                    ))],
                    Location::special(),
                )),
            ],
            Location::special(),
        );

        let tokens2 = tokenize(
            "
            {1 + 1}
            {2 + 2}
            3 + 3;
            ",
            "file.txt",
        );
        let mut p2 = Parser::new(tokens2);
        let expected2 = Block::new(
            vec![
                Box::new(Block::new(
                    vec![Box::new(BinaryOp::new(
                        Box::new(Literal::new(1, Location::special())),
                        "+",
                        Box::new(Literal::new(1, Location::special())),
                    ))],
                    Location::special(),
                )),
                Box::new(Block::new(
                    vec![Box::new(BinaryOp::new(
                        Box::new(Literal::new(2, Location::special())),
                        "+",
                        Box::new(Literal::new(2, Location::special())),
                    ))],
                    Location::special(),
                )),
                Box::new(BinaryOp::new(
                    Box::new(Literal::new(3, Location::special())),
                    "+",
                    Box::new(Literal::new(3, Location::special())),
                )),
                Box::new(Literal::none(Location::special())),
            ],
            Location::special(),
        );

        // test1
        assert_eq!(
            expected1,
            *p1.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<Block>()
                .unwrap()
        );

        // test2
        assert_eq!(
            expected2,
            *p2.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<Block>()
                .unwrap()
        );
    }
}
