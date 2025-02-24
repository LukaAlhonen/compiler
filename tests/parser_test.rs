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
                left: Box::new(Literal {
                    value: Some(2.into()),
                }),
                op: "+".to_string(),
                right: Box::new(Literal {
                    value: Some(1.into()),
                }),
            }),
            op: "-".to_string(),
            right: Box::new(Literal {
                value: Some(3.into()),
            }),
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
            left: Box::new(Literal {
                value: Some(2.into()),
            }),
            op: "+".to_string(),
            right: Box::new(BinaryOp {
                left: Box::new(Literal {
                    value: Some(4.into()),
                }),
                op: "*".to_string(),
                right: Box::new(Literal {
                    value: Some(3.into()),
                }),
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

        let expected = If {
            cond: Box::new(Identifier {
                name: "a".to_string(),
            }),
            then: Box::new(BinaryOp {
                left: Box::new(Identifier {
                    name: "b".to_string(),
                }),
                op: "+".to_string(),
                right: Box::new(Identifier {
                    name: "c".to_string(),
                }),
            }),
            if_else: None,
        };

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<If>().unwrap()
        );
    }

    #[test]
    fn test_parse_if_else() {
        let tokens = tokenize("if a then b + c else b * c", "file.txt");
        let mut p: Parser = Parser::new(tokens);

        let expected = If {
            cond: Box::new(Identifier {
                name: "a".to_string(),
            }),
            then: Box::new(BinaryOp {
                left: Box::new(Identifier {
                    name: "b".to_string(),
                }),
                op: "+".to_string(),
                right: Box::new(Identifier {
                    name: "c".to_string(),
                }),
            }),
            if_else: Some(Box::new(BinaryOp {
                left: Box::new(Identifier {
                    name: "b".to_string(),
                }),
                op: "*".to_string(),
                right: Box::new(Identifier {
                    name: "c".to_string(),
                }),
            })),
        };

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<If>().unwrap()
        );
    }

    #[test]
    fn test_parse_embedded_if() {
        let tokens = tokenize("1 + if true then 2 else 3", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = BinaryOp {
            left: Box::new(Literal {
                value: Some(1.into()),
            }),
            op: "+".to_string(),
            right: Box::new(If {
                cond: Box::new(Literal {
                    value: Some(true.into()),
                }),
                then: Box::new(Literal {
                    value: Some(2.into()),
                }),
                if_else: Some(Box::new(Literal {
                    value: Some(3.into()),
                })),
            }),
        };
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

        let expected = If {
            cond: Box::new(Identifier {
                name: "a".to_string(),
            }),
            then: Box::new(If {
                cond: Box::new(Identifier {
                    name: "b".to_string(),
                }),
                then: Box::new(Identifier {
                    name: "c".to_string(),
                }),
                if_else: Some(Box::new(Identifier {
                    name: "d".to_string(),
                })),
            }),
            if_else: Some(Box::new(Identifier {
                name: "e".to_string(),
            })),
        };

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
        let expected1 = FunctionCall {
            name: Identifier {
                name: "a".to_string(),
            },
            args: vec![
                Box::new(Literal {
                    value: Some(1.into()),
                }),
                Box::new(Literal {
                    value: Some(2.into()),
                }),
                Box::new(Literal {
                    value: Some(3.into()),
                }),
            ],
        };
        assert_eq!(
            expected1,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<FunctionCall>()
                .unwrap()
        );

        p = Parser::new(tokens2);
        let expected2 = FunctionCall {
            name: Identifier {
                name: "b".to_string(),
            },
            args: vec![
                Box::new(Identifier {
                    name: "c".to_string(),
                }),
                Box::new(Identifier {
                    name: "d".to_string(),
                }),
                Box::new(Identifier {
                    name: "e".to_string(),
                }),
            ],
        };
        assert_eq!(
            expected2,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<FunctionCall>()
                .unwrap()
        );

        p = Parser::new(tokens3);
        let expected3 = FunctionCall {
            name: Identifier {
                name: "c".to_string(),
            },
            args: vec![],
        };
        assert_eq!(
            expected3,
            *p.parse()
                .unwrap()
                .as_any()
                .downcast_ref::<FunctionCall>()
                .unwrap()
        );

        p = Parser::new(tokens4);
        let expected4 = FunctionCall {
            name: Identifier {
                name: "d".to_string(),
            },
            args: vec![
                Box::new(BinaryOp {
                    left: Box::new(Literal {
                        value: Some(1.into()),
                    }),
                    op: "+".to_string(),
                    right: Box::new(Literal {
                        value: Some(2.into()),
                    }),
                }),
                Box::new(If {
                    cond: Box::new(Literal {
                        value: Some(true.into()),
                    }),
                    then: Box::new(Literal {
                        value: Some(3.into()),
                    }),
                    if_else: Some(Box::new(Literal {
                        value: Some(1.into()),
                    })),
                }),
            ],
        };
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

        let expected = BinaryOp {
            left: Box::new(Literal {
                value: Some(true.into()),
            }),
            op: "or".to_string(),
            right: Box::new(Literal {
                value: Some(false.into()),
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
    fn test_if_or() {
        let tokens = tokenize(
            "if a == 2 + 2 or b == 2 + 2 then a + 2 else b + 2",
            "file.txt",
        );

        let mut p: Parser = Parser::new(tokens);

        let expected = If {
            cond: Box::new(BinaryOp {
                left: Box::new(BinaryOp {
                    left: Box::new(Identifier {
                        name: "a".to_string(),
                    }),
                    op: "==".to_string(),
                    right: Box::new(BinaryOp {
                        left: Box::new(Literal {
                            value: Some(2.into()),
                        }),
                        op: "+".to_string(),
                        right: Box::new(Literal {
                            value: Some(2.into()),
                        }),
                    }),
                }),
                op: "or".to_string(),
                right: Box::new(BinaryOp {
                    left: Box::new(Identifier { name: "b".into() }),
                    op: "==".into(),
                    right: Box::new(BinaryOp {
                        left: Box::new(Literal {
                            value: Some(2.into()),
                        }),
                        op: "+".to_string(),
                        right: Box::new(Literal {
                            value: Some(2.into()),
                        }),
                    }),
                }),
            }),
            then: Box::new(BinaryOp {
                left: Box::new(Identifier {
                    name: "a".to_string(),
                }),
                op: "+".to_string(),
                right: Box::new(Literal {
                    value: Some(2.into()),
                }),
            }),
            if_else: Some(Box::new(BinaryOp {
                left: Box::new(Identifier {
                    name: "b".to_string(),
                }),
                op: "+".to_string(),
                right: Box::new(Literal {
                    value: Some(2.into()),
                }),
            })),
        };

        assert_eq!(
            expected,
            *p.parse().unwrap().as_any().downcast_ref::<If>().unwrap()
        );
    }

    #[test]
    fn test_binary_op_precedence() {
        let tokens = tokenize("a = 2 + 3 * 4", "file.txt");
        let mut p: Parser = Parser::new(tokens);
        let expected = BinaryOp {
            left: Box::new(Identifier {
                name: "a".to_string(),
            }),
            op: "=".to_string(),
            right: Box::new(BinaryOp {
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
    fn test_assignment() {
        let tokens = tokenize("a = b = c", "file.txt");
        let mut p = Parser::new(tokens);

        let expected = BinaryOp {
            left: Box::new(Identifier {
                name: "a".to_string(),
            }),
            op: "=".to_string(),
            right: Box::new(BinaryOp {
                left: Box::new(Identifier {
                    name: "b".to_string(),
                }),
                op: "=".to_string(),
                right: Box::new(Identifier {
                    name: "c".to_string(),
                }),
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
    fn test_parse_block() {
        let tokens1 = tokenize("{a + b; a}", "file.txt");
        let tokens2 = tokenize("{a + b; a;}", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut p2 = Parser::new(tokens2);

        let expected1 = Block {
            statements: vec![
                Box::new(BinaryOp {
                    left: Box::new(Identifier {
                        name: "a".to_string(),
                    }),
                    op: "+".to_string(),
                    right: Box::new(Identifier {
                        name: "b".to_string(),
                    }),
                }),
                Box::new(Identifier {
                    name: "a".to_string(),
                }),
            ],
        };
        let expected2 = Block {
            statements: vec![
                Box::new(BinaryOp {
                    left: Box::new(Identifier {
                        name: "a".to_string(),
                    }),
                    op: "+".to_string(),
                    right: Box::new(Identifier {
                        name: "b".to_string(),
                    }),
                }),
                Box::new(Identifier {
                    name: "a".to_string(),
                }),
                Box::new(Literal { value: None }),
            ],
        };

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
                    };
                    g(y);
                };
                123
            }",
            "file.txt",
        );
        let mut p = Parser::new(tokens);

        let expected = Block {
            statements: vec![
                Box::new(While {
                    cond: Box::new(FunctionCall {
                        name: Identifier {
                            name: "f".to_string(),
                        },
                        args: vec![],
                    }),
                    body: Box::new(Block {
                        statements: vec![
                            Box::new(BinaryOp {
                                left: Box::new(Identifier {
                                    name: "x".to_string(),
                                }),
                                op: "=".to_string(),
                                right: Box::new(Literal {
                                    value: Some(10.into()),
                                }),
                            }),
                            Box::new(BinaryOp {
                                left: Box::new(Identifier {
                                    name: "y".to_string(),
                                }),
                                op: "=".to_string(),
                                right: Box::new(If {
                                    cond: Box::new(FunctionCall {
                                        name: Identifier {
                                            name: "g".to_string(),
                                        },
                                        args: vec![Box::new(Identifier {
                                            name: "x".to_string(),
                                        })],
                                    }),
                                    then: Box::new(Block {
                                        statements: vec![
                                            Box::new(BinaryOp {
                                                left: Box::new(Identifier {
                                                    name: "x".to_string(),
                                                }),
                                                op: "=".to_string(),
                                                right: Box::new(BinaryOp {
                                                    left: Box::new(Identifier {
                                                        name: "x".to_string(),
                                                    }),
                                                    op: "+".to_string(),
                                                    right: Box::new(Literal {
                                                        value: Some(1.into()),
                                                    }),
                                                }),
                                            }),
                                            Box::new(Identifier {
                                                name: "x".to_string(),
                                            }),
                                        ],
                                    }),
                                    if_else: Some(Box::new(Block {
                                        statements: vec![Box::new(FunctionCall {
                                            name: Identifier {
                                                name: "g".to_string(),
                                            },
                                            args: vec![Box::new(Identifier {
                                                name: "x".to_string(),
                                            })],
                                        })],
                                    })),
                                }),
                            }),
                            Box::new(FunctionCall {
                                name: Identifier {
                                    name: "g".to_string(),
                                },
                                args: vec![Box::new(Identifier {
                                    name: "y".to_string(),
                                })],
                            }),
                            Box::new(Literal { value: None }),
                        ],
                    }),
                }),
                Box::new(Literal {
                    value: Some(123.into()),
                }),
            ],
        };

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

        let expected1 = VarDeclaration {
            var: Identifier {
                name: "x".to_string(),
            },
            initializer: Box::new(Literal {
                value: Some(1.into()),
            }),
        };
        let expected2 = Block {
            statements: vec![
                Box::new(VarDeclaration {
                    var: Identifier {
                        name: "x".to_string(),
                    },
                    initializer: Box::new(Literal {
                        value: Some(1.into()),
                    }),
                }),
                Box::new(Literal { value: None }),
            ],
        };

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

        let expected = While {
            cond: Box::new(Identifier {
                name: "x".to_string(),
            }),
            body: Box::new(Identifier {
                name: "y".to_string(),
            }),
        };

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
}
