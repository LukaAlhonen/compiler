use super::ast::*;
use super::symtab::SymTab;
use super::tokenizer::*;
use super::types::*;
use anyhow::{anyhow, Error, Result};

#[derive(Debug, PartialEq)]
pub enum TypeError {
    MismatchedTypes { expected: Type, loc: Location },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MismatchedTypes { expected, loc } => {
                write!(
                    f,
                    "{}: Mismatched types, expected {}",
                    loc.to_string(),
                    expected.as_string()
                )
            }
        }
    }
}

impl std::error::Error for TypeError {}

fn fun_args_are_correct(args: Vec<Type>, expected: Vec<Box<Type>>) -> bool {
    if expected.len() != args.len() {
        return false;
    }

    if expected.iter().zip(args.iter()).all(|(a, b)| *b == **a) {
        return true;
    } else {
        return false;
    }
}

fn typecheck_assignment(
    left: &mut Box<dyn Expression>,
    right: &mut Box<dyn Expression>,
    symtab: &mut SymTab<Type>,
    is_leftmost: bool,
) -> Result<Type, Error> {
    let t1 = typecheck(left, symtab)?;
    let t2: Type = if let Some(bin_op) = right.as_any_mut().downcast_mut::<BinaryOp>() {
        if &bin_op.op == "=" {
            typecheck_assignment(&mut bin_op.left, &mut bin_op.right, symtab, false)?
        } else {
            typecheck(right, symtab)?
        }
    } else {
        typecheck(right, symtab)?
    };

    if t1 != t2 {
        return Err(anyhow!(TypeError::MismatchedTypes {
            expected: t1,
            loc: right.get_loc().clone()
        }));
    }

    if is_leftmost {
        return Ok(Type::Unit);
    } else {
        return Ok(t1);
    }
}

// TODO:
// - Error handling
pub fn typecheck(node: &mut Box<dyn Expression>, symtab: &mut SymTab<Type>) -> Result<Type, Error> {
    if let Some(literal) = node.as_any_mut().downcast_mut::<Literal>() {
        match &literal.value {
            Some(lit_val) => match lit_val {
                LiteralValue::Bool(_) => {
                    literal.set_type(Type::Bool);
                    Ok(Type::Bool)
                }
                LiteralValue::Int(_) => {
                    literal.set_type(Type::Int);
                    Ok(Type::Int)
                }
            },
            None => {
                literal.set_type(Type::Unit);
                Ok(Type::Unit)
            }
        }
    } else if let Some(bin_op) = node.as_any_mut().downcast_mut::<BinaryOp>() {
        let t1 = typecheck(&mut bin_op.left, symtab)?;
        let t2 = typecheck(&mut bin_op.right, symtab)?;
        if let Some(op) = symtab.get(&bin_op.op) {
            let (expected, result) = match op {
                Type::FunType { args, result } => (args.clone(), result),
                _ => return Err(anyhow!("Not a function!")),
            };

            if fun_args_are_correct(vec![t1, t2], expected) {
                bin_op.set_type(*result.clone());
                return Ok(*result.clone());
            } else {
                return Err(anyhow!("Mismatched function arg types"));
            }
        } else if ["==", "!="].contains(&bin_op.op.as_str()) {
            if t1 == t2 {
                bin_op.set_type(Type::Bool);
                return Ok(Type::Bool);
            } else {
                return Err(anyhow!(TypeError::MismatchedTypes {
                    expected: t1,
                    loc: bin_op.right.get_loc().clone()
                }));
            }
        } else if &bin_op.op == "=" {
            if let Some(_) = &bin_op.left.as_any_mut().downcast_mut::<Identifier>() {
                bin_op.set_type(Type::Unit);
                return typecheck_assignment(&mut bin_op.left, &mut bin_op.right, symtab, true);
            } else {
                return Err(anyhow!(
                    "{}: Left side of assignment must be variable name",
                    bin_op.left.get_loc().to_string().clone()
                ));
            }
        } else {
            return Err(anyhow!(
                "{}: Invalid operator \"{}\"",
                &bin_op.loc.to_string(),
                &bin_op.op
            ));
        }
    } else if let Some(literal) = node.as_any_mut().downcast_mut::<Literal>() {
        match &literal.value {
            Some(val) => match val {
                LiteralValue::Int(_) => {
                    literal.set_type(Type::Int);
                    Ok(Type::Int)
                }
                LiteralValue::Bool(_) => {
                    literal.set_type(Type::Bool);
                    Ok(Type::Bool)
                }
            },
            None => {
                literal.set_type(Type::Unit);
                Ok(Type::Unit)
            }
        }
    } else if let Some(if_statement) = node.as_any_mut().downcast_mut::<If>() {
        let t1 = typecheck(&mut if_statement.cond, symtab)?;
        if t1 != Type::Bool {
            return Err(anyhow!(TypeError::MismatchedTypes {
                expected: Type::Bool,
                loc: if_statement.cond.get_loc().clone()
            }));
        }

        let t2 = typecheck(&mut if_statement.then_branch, symtab)?;
        if let Some(else_branch) = &mut if_statement.else_branch {
            let t3 = typecheck(else_branch, symtab)?;
            if t2 != t3 {
                return Err(anyhow!(TypeError::MismatchedTypes {
                    expected: Type::Bool,
                    loc: else_branch.get_loc().clone()
                }));
            }
            if_statement.set_type(t2.clone());
            return Ok(t2);
        }
        if_statement.set_type(Type::Unit);
        return Ok(Type::Unit);
    } else if let Some(block) = node.as_any_mut().downcast_mut::<Block>() {
        let mut local_symtab = SymTab::new_with_parent(symtab);
        for expression in &mut block.statements {
            typecheck(expression, &mut local_symtab)?;
        }
        match block.statements.last_mut() {
            Some(statement) => {
                let t = typecheck(statement, &mut local_symtab)?;
                block.set_type(t.clone());
                return Ok(t);
            }
            None => {
                block.set_type(Type::Unit);
                return Ok(Type::Unit);
            }
        }
    } else if let Some(while_statement) = node.as_any_mut().downcast_mut::<While>() {
        if typecheck(&mut while_statement.cond, symtab)? != Type::Bool {
            return Err(anyhow!(TypeError::MismatchedTypes {
                expected: Type::Bool,
                loc: while_statement.get_loc().clone()
            }));
        }
        return Ok(Type::Unit);
    } else if let Some(unary_op) = node.as_any_mut().downcast_mut::<UnaryOp>() {
        if let Some(op) = symtab.get(format!("unary_{}", unary_op.op).as_str()) {
            let (expected, result) = match op {
                Type::FunType { args, result } => (args.clone(), result.clone()),
                _ => return Err(anyhow!("Invalid operator")),
            };

            let t1 = typecheck(&mut unary_op.right, symtab)?;

            if fun_args_are_correct(vec![t1], expected) {
                unary_op.set_type(*result.clone());
                return Ok(*result);
            } else {
                return Err(anyhow!("Mismatched function arg types"));
            }
        } else {
            return Err(anyhow!("Invalid operator"));
        }
    } else if let Some(var_declaration) = node.as_any_mut().downcast_mut::<VarDeclaration>() {
        let initialized_value = typecheck(&mut var_declaration.initializer, symtab)?;
        if let Some(t) = &var_declaration.declared_type {
            if &initialized_value != t {
                return Err(anyhow!(
                    "Declared type {}, does not match type of initializer: {}",
                    t.as_string(),
                    initialized_value.as_string()
                ));
            }
        }

        let _ = symtab.insert(&var_declaration.var.name, initialized_value);
        var_declaration.set_type(Type::Unit);
        Ok(Type::Unit)
    } else if let Some(var) = node.as_any_mut().downcast_mut::<Identifier>() {
        match symtab.get(var.name.as_str()) {
            Some(t) => {
                var.set_type(t.clone());
                return Ok(t.clone());
            }
            None => {
                return Err(anyhow!(
                    "Could not find variable \"{}\" in this scope",
                    var.name
                ))
            }
        }
    } else if let Some(function_call) = node.as_any_mut().downcast_mut::<FunctionCall>() {
        let (expected_args, result) = match symtab.get(&function_call.name.name.as_str()) {
            Some(f) => match f {
                Type::FunType { args, result } => (args.clone(), result.clone()),
                _ => return Err(anyhow!("Not a function")),
            },
            None => {
                return Err(anyhow!(
                    "Could not find function \"{}\" in this scope",
                    &function_call.name.name
                ))
            }
        };

        let arg_types = function_call
            .args
            .iter_mut()
            .map(|arg| typecheck(arg, symtab))
            .collect::<Result<Vec<_>, _>>()?;

        if fun_args_are_correct(arg_types, expected_args) {
            function_call.set_type(*result.clone());
            return Ok(*result);
        } else {
            return Err(anyhow!("Mismatched function arg types"));
        }
    } else {
        return Ok(Type::Unit);
    }
}

// for testing
pub fn init_globals(st: &mut SymTab<Type>) {
    st.insert(
        "print_int".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int)],
            result: Box::new(Type::Unit),
        },
    );
    st.insert(
        "print_bool".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Bool)],
            result: Box::new(Type::Unit),
        },
    );
    st.insert(
        "+".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Int),
        },
    );
    st.insert(
        "-".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Int),
        },
    );
    st.insert(
        "/".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Int),
        },
    );
    st.insert(
        "%".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Int),
        },
    );
    st.insert(
        "*".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Int),
        },
    );
    st.insert(
        ">".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Bool),
        },
    );
    st.insert(
        "<".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Bool),
        },
    );
    st.insert(
        "<=".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Bool),
        },
    );
    st.insert(
        ">=".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int), Box::new(Type::Int)],
            result: Box::new(Type::Bool),
        },
    );
    st.insert(
        "and".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Bool), Box::new(Type::Bool)],
            result: Box::new(Type::Bool),
        },
    );
    st.insert(
        "or".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Bool), Box::new(Type::Bool)],
            result: Box::new(Type::Bool),
        },
    );
    st.insert(
        "unary_not".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Bool)],
            result: Box::new(Type::Bool),
        },
    );
    st.insert(
        "unary_-".to_string(),
        Type::FunType {
            args: vec![Box::new(Type::Int)],
            result: Box::new(Type::Int),
        },
    );
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::parser::Parser;

    #[test]
    fn test_typecheck_bin_op() {
        let tokens1 = tokenize("1 + 1", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut parsed1 = p1.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        assert_eq!(typecheck(&mut parsed1, &mut st).unwrap(), Type::Int);
        assert_eq!(parsed1.get_type(), &Type::Int);

        let tokens2 = tokenize("1 == 1", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let mut parsed2 = p2.parse().unwrap();
        assert_eq!(typecheck(&mut parsed2, &mut st).unwrap(), Type::Bool);
        assert_eq!(parsed2.get_type(), &Type::Bool);
    }

    #[test]
    #[should_panic]
    fn test_typecheck_bin_op_panic() {
        let tokens = tokenize("var x = 1;\nvar y = false;\nx + y", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }

    #[test]
    fn test_typecheck_if() {
        let tokens1 = tokenize("var a = if true or false then 1 else 2;\na", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut parsed1 = p1.parse().unwrap();
        let mut st1 = SymTab::new();
        init_globals(&mut st1);
        assert_eq!(typecheck(&mut parsed1, &mut st1).unwrap(), Type::Int);
        assert_eq!(parsed1.get_type(), &Type::Int);

        let mut st2 = SymTab::new();
        init_globals(&mut st2);
        let tokens2 = tokenize("if true then 1", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let mut parsed2 = p2.parse().unwrap();
        assert_eq!(typecheck(&mut parsed2, &mut st2).unwrap(), Type::Unit);
        assert_eq!(parsed2.get_type(), &Type::Unit);
    }

    #[test]
    #[should_panic]
    fn test_typecheck_if_panic() {
        let tokens = tokenize("if 1 + 1 then true else false", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }

    #[test]
    fn test_typetype_check_unary_op() {
        let tokens1 = tokenize("-1", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut parsed1 = p1.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        assert_eq!(typecheck(&mut parsed1, &mut st).unwrap(), Type::Int);
        assert_eq!(parsed1.get_type(), &Type::Int);

        let tokens2 = tokenize("{\nvar x = false;\n not x\n}", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let mut parsed2 = p2.parse().unwrap();
        assert_eq!(typecheck(&mut parsed2, &mut st).unwrap(), Type::Bool);
        assert_eq!(parsed2.get_type(), &Type::Bool);
    }
    #[test]
    #[should_panic]
    fn test_typecheck_unary_op_panic() {
        let tokens = tokenize("not 1", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }

    #[test]
    fn test_typecheck_var_declaration() {
        let tokens = tokenize("{\nvar a: Int = 1;\na\n}", "file.txt");
        let mut p = Parser::new(tokens);
        let mut parsed = p.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        assert_eq!(typecheck(&mut parsed, &mut st).unwrap(), Type::Int);
        assert_eq!(parsed.get_type(), &Type::Int);
    }

    #[test]
    #[should_panic]
    fn test_typecheck_var_declaration_panic() {
        let tokens = tokenize("var x: Int = true", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }

    #[test]
    fn test_typecheck_block() {
        let tokens1 = tokenize("{\nvar x: Int = {1 + 1}\n}", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut parsed1 = p1.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        assert_eq!(typecheck(&mut parsed1, &mut st).unwrap(), Type::Unit);
        assert_eq!(parsed1.get_type(), &Type::Unit);

        let tokens2 = tokenize("{\n1 > 1\n}", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let mut parsed2 = p2.parse().unwrap();
        assert_eq!(typecheck(&mut parsed2, &mut st).unwrap(), Type::Bool);
        assert_eq!(parsed2.get_type(), &Type::Bool);
    }

    #[test]
    fn test_typecheck_var_assignment() {
        let tokens = tokenize(
            "{\nvar a = 0;\nvar b = 0;\nvar c = a + b;\na = b = c = 1\n}",
            "file.txt",
        );
        let mut p = Parser::new(tokens);
        let mut parsed = p.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);

        assert_eq!(typecheck(&mut parsed, &mut st).unwrap(), Type::Unit);
        assert_eq!(parsed.get_type(), &Type::Unit);
    }

    #[test]
    #[should_panic]
    fn test_typecheck_var_assignment_panic() {
        let tokens = tokenize(
            "var x = 1;\nvar y = 1;\nvar z = true;\nx = y = z = 1",
            "file.txt",
        );
        let mut p = Parser::new(tokens);

        let mut st = SymTab::new();
        init_globals(&mut st);

        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }

    #[test]
    fn test_typecheck_special_operators() {
        let tokens1 = tokenize("true == false", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut parsed1 = p1.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        assert_eq!(typecheck(&mut parsed1, &mut st).unwrap(), Type::Bool);
        assert_eq!(parsed1.get_type(), &Type::Bool);

        let tokens2 = tokenize("1 != 2", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let mut parsed2 = p2.parse().unwrap();
        assert_eq!(typecheck(&mut parsed2, &mut st).unwrap(), Type::Bool);
        assert_eq!(parsed2.get_type(), &Type::Bool);
    }

    #[test]
    #[should_panic]
    fn test_typecheck_special_operators_panic() {
        let tokens = tokenize("1 == true", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }

    #[test]
    fn test_typecheck_function_call() {
        let tokens1 = tokenize("{\nvar x = true;\nprint_bool(x)\n}", "file.txt");
        let mut p1 = Parser::new(tokens1);
        let mut parsed1 = p1.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        assert_eq!(typecheck(&mut parsed1, &mut st).unwrap(), Type::Unit);
        assert_eq!(parsed1.get_type(), &Type::Unit);

        let tokens2 = tokenize("{\nvar y = 1; print_int(y)\n}", "file.txt");
        let mut p2 = Parser::new(tokens2);
        let mut parsed2 = p2.parse().unwrap();
        assert_eq!(typecheck(&mut parsed2, &mut st).unwrap(), Type::Unit);
        assert_eq!(parsed2.get_type(), &Type::Unit);
    }

    #[test]
    #[should_panic]
    fn test_typecheck_function_call_panic() {
        let tokens = tokenize("print_int(false)", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }

    #[test]
    fn test_typecheck_while() {
        let tokens = tokenize("var x = 0; while x < 10 do x + 1", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);

        let mut parsed = p.parse().unwrap();
        assert_eq!(typecheck(&mut parsed, &mut st).unwrap(), Type::Unit);
        assert_eq!(parsed.get_type(), &Type::Unit);
    }

    #[test]
    #[should_panic]
    fn test_typecheck_while_panic() {
        let tokens = tokenize("while 1 + 1 do false", "file.txt");
        let mut p = Parser::new(tokens);
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut p.parse().unwrap(), &mut st).unwrap();
    }
}
