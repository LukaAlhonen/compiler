use anyhow::anyhow;
use anyhow::Error;

use super::ast::*;
use super::ir::*;
use super::symtab::SymTab;
use super::tokenizer::Location;
use super::types::Type;
use std::collections::HashMap;

fn new_var(t: Type, var_types: &mut HashMap<IRVar, Type>) -> IRVar {
    let mut i = 0;
    while var_types.contains_key(&IRVar {
        name: format!("x{}", i),
    }) {
        i += 1;
    }

    let v = IRVar::new(format!("x{}", i));

    var_types.insert(v.clone(), t);
    v
}

fn new_label(ins: &mut Vec<Box<dyn Instruction>>, base: &str, loc: Location) -> Label {
    let mut i = 0;
    for instruction in ins {
        if let Some(label) = instruction.as_any().downcast_ref::<Label>() {
            let l = Label::new(format!("{}{}", base, i), loc.clone());
            if label == &l {
                i += 1;
            } else {
                return l;
            }
        }
    }
    Label {
        name: base.to_string(),
        loc,
    }
}

// TODO:
// - While
// - Var declaration
// - ==, !=
// - unary
// - short circuit and, or
// - assignment
// - function calls

fn visit(
    st: &mut SymTab<IRVar>,
    expr: &Box<dyn Expression>,
    ins: &mut Vec<Box<dyn Instruction>>,
    var_types: &mut HashMap<IRVar, Type>,
) -> Result<IRVar, Error> {
    let loc = expr.get_loc().clone();

    if let Some(literal) = expr.as_any().downcast_ref::<Literal>() {
        match &literal.value {
            Some(value) => match value {
                LiteralValue::Int(int_val) => {
                    let var = new_var(Type::Int, var_types);
                    ins.push(Box::new(LoadIntConst::new(*int_val, var.clone(), loc)));
                    return Ok(var);
                }
                LiteralValue::Bool(bool_val) => {
                    let var = new_var(Type::Bool, var_types);
                    ins.push(Box::new(LoadBoolConst::new(*bool_val, var.clone(), loc)));
                    return Ok(var);
                }
            },
            None => {
                return Ok(IRVar {
                    name: "unit".to_string(),
                });
            }
        }
    } else if let Some(identifier) = expr.as_any().downcast_ref::<Identifier>() {
        if let Some(v) = st.get(identifier.name.as_str()) {
            return Ok(v.clone());
        } else {
            return Err(anyhow!("var not found"));
        }
    } else if let Some(bin_op) = expr.as_any().downcast_ref::<BinaryOp>() {
        let v = st.get(&bin_op.op).cloned();
        match v {
            Some(var_op) => {
                let var_left = visit(st, &bin_op.left, ins, var_types)?;
                let var_right = visit(st, &bin_op.right, ins, var_types)?;
                let var_result = new_var(bin_op.t.clone(), var_types);

                ins.push(Box::new(Call::new(
                    var_op.clone(),
                    vec![var_left, var_right],
                    var_result.clone(),
                    loc,
                )));
                return Ok(var_result);
            }
            None => return Err(anyhow!("")),
        }
    } else if let Some(if_expr) = expr.as_any().downcast_ref::<If>() {
        match &if_expr.else_branch {
            Some(else_branch) => {
                let l_then = new_label(ins, "then", if_expr.then_branch.get_loc().clone());
                let l_else = new_label(ins, "else", else_branch.get_loc().clone());
                let l_end = new_label(ins, "if_end", if_expr.loc.clone()); // need to figure out what loc to assign here

                let var_cond = visit(st, &if_expr.cond, ins, var_types)?;
                let if_var = new_var(if_expr.get_type().clone(), var_types);

                ins.push(Box::new(CondJump::new(
                    var_cond,
                    l_then.clone(),
                    l_else.clone(),
                    loc,
                )));

                ins.push(Box::new(l_then));
                let then_var = visit(st, &if_expr.then_branch, ins, var_types)?;
                ins.push(Box::new(Copy::new(
                    then_var,
                    if_var.clone(),
                    if_expr.then_branch.get_loc().clone(),
                )));
                ins.push(Box::new(Jump::new(
                    l_end.clone(),
                    if_expr.then_branch.get_loc().clone(),
                )));

                ins.push(Box::new(l_else));
                let else_var = visit(st, &else_branch, ins, var_types)?;
                ins.push(Box::new(Copy::new(
                    else_var,
                    if_var.clone(),
                    if_expr.then_branch.get_loc().clone(),
                )));

                ins.push(Box::new(l_end));

                // return new IRVar with type of if_else expr
                return Ok(if_var);
            }
            None => {
                let l_then = new_label(ins, "then", if_expr.then_branch.get_loc().clone());
                let l_end = new_label(ins, "if_end", loc.clone());

                let var_cond = visit(st, &if_expr.cond, ins, var_types)?;

                ins.push(Box::new(CondJump::new(
                    var_cond,
                    l_then.clone(),
                    l_end.clone(),
                    loc,
                )));

                ins.push(Box::new(l_then));

                visit(st, &if_expr.then_branch, ins, var_types)?;

                ins.push(Box::new(l_end));

                return Ok(IRVar {
                    name: "unit".to_string(),
                });
            }
        }
    } else if let Some(block) = expr.as_any().downcast_ref::<Block>() {
        for statement in &block.statements {
            visit(st, statement, ins, var_types)?;
        }

        let block_var = new_var(block.t.clone(), var_types);

        Ok(block_var)
    } else if let Some(while_statement) = expr.as_any().downcast_ref::<While>() {
        let l_while = new_label(ins, "while_start", while_statement.loc.clone());
        let l_body = new_label(ins, "while_body", while_statement.body.get_loc().clone());
        let l_end = new_label(ins, "while_end", while_statement.loc.clone());

        ins.push(Box::new(l_while.clone()));
        let var_cond = visit(st, &while_statement.cond, ins, var_types)?;

        ins.push(Box::new(CondJump::new(
            var_cond,
            l_body.clone(),
            l_end.clone(),
            loc.clone(),
        )));

        ins.push(Box::new(l_body));
        visit(st, &while_statement.body, ins, var_types)?;

        ins.push(Box::new(Jump::new(l_while, loc.clone())));
        ins.push(Box::new(l_end));

        Ok(IRVar {
            name: "unit".to_string(),
        })
    } else {
        return Err(anyhow!(
            "{}: Unknown expression {:?}",
            loc.to_string(),
            expr
        ));
    }
}

pub fn generate_ir(
    root_types: HashMap<IRVar, Type>,
    root_expr: Box<dyn Expression>,
) -> Result<Vec<Box<dyn Instruction>>, Error> {
    let mut var_types = root_types.clone();
    var_types.insert(IRVar::new("unit"), Type::Unit);

    let mut ins: Vec<Box<dyn Instruction>> = vec![];
    let mut root_symtab: SymTab<IRVar> = SymTab::new();
    for key in root_types.into_keys() {
        root_symtab.insert(key.name.clone(), key);
    }

    let var_final_result = visit(&mut root_symtab, &root_expr, &mut ins, &mut var_types)?;
    if var_types.get(&var_final_result) == Some(&Type::Int) {
        ins.push(Box::new(Call::new(
            IRVar::new("print_int"),
            vec![var_final_result],
            new_var(Type::Int, &mut var_types),
            Location::special(),
        )));
    } else if var_types.get(&var_final_result) == Some(&Type::Bool) {
        ins.push(Box::new(Call::new(
            IRVar::new("print_bool"),
            vec![var_final_result],
            new_var(Type::Bool, &mut var_types),
            Location::special(),
        )));
    }
    Ok(ins)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::compiler::parser::*;
    use crate::compiler::tokenizer::tokenize;
    use crate::compiler::type_checker::*;

    #[test]
    fn test_gen_bin_op_ir() {
        let tokens1 = tokenize("1 + 2 * 3", "file.txt");
        let tokens2 = tokenize("1 + 2 * 3;", "file.txt");

        let mut p1 = Parser::new(tokens1);
        let mut p2 = Parser::new(tokens2);

        let mut parsed1 = p1.parse().unwrap();
        let mut parsed2 = p2.parse().unwrap();

        let mut st1: SymTab<Type> = SymTab::new();
        let mut st2: SymTab<Type> = SymTab::new();

        init_globals(&mut st1);
        init_globals(&mut st2);

        typecheck(&mut parsed1, &mut st1).unwrap();
        typecheck(&mut parsed2, &mut st2).unwrap();

        let mut root_types1: HashMap<IRVar, Type> = HashMap::new();
        for (name, t) in st1.locals.iter() {
            root_types1.insert(IRVar { name: name.clone() }, t.clone());
        }
        let mut root_types2: HashMap<IRVar, Type> = HashMap::new();
        for (name, t) in st2.locals.iter() {
            root_types2.insert(IRVar { name: name.clone() }, t.clone());
        }

        let ir1 = generate_ir(root_types1, parsed1).unwrap();
        let ir2 = generate_ir(root_types2, parsed2).unwrap();

        let expected1: Vec<Box<dyn Instruction>> = vec![
            Box::new(LoadIntConst::new(
                1,
                IRVar {
                    name: "x0".to_string(),
                },
                Location::special(),
            )),
            Box::new(LoadIntConst::new(
                2,
                IRVar {
                    name: "x1".to_string(),
                },
                Location::special(),
            )),
            Box::new(LoadIntConst::new(
                3,
                IRVar {
                    name: "x2".to_string(),
                },
                Location::special(),
            )),
            Box::new(Call::new(
                IRVar {
                    name: "*".to_string(),
                },
                vec![
                    IRVar {
                        name: "x1".to_string(),
                    },
                    IRVar {
                        name: "x2".to_string(),
                    },
                ],
                IRVar {
                    name: "x3".to_string(),
                },
                Location::special(),
            )),
            Box::new(Call::new(
                IRVar {
                    name: "+".to_string(),
                },
                vec![
                    IRVar {
                        name: "x0".to_string(),
                    },
                    IRVar {
                        name: "x3".to_string(),
                    },
                ],
                IRVar {
                    name: "x4".to_string(),
                },
                Location::special(),
            )),
            Box::new(Call::new(
                IRVar {
                    name: "print_int".to_string(),
                },
                vec![IRVar {
                    name: "x4".to_string(),
                }],
                IRVar {
                    name: "x5".to_string(),
                },
                Location::special(),
            )),
        ];

        let expected2: Vec<Box<dyn Instruction>> = vec![
            Box::new(LoadIntConst::new(
                1,
                IRVar {
                    name: "x0".to_string(),
                },
                Location::special(),
            )),
            Box::new(LoadIntConst::new(
                2,
                IRVar {
                    name: "x1".to_string(),
                },
                Location::special(),
            )),
            Box::new(LoadIntConst::new(
                3,
                IRVar {
                    name: "x2".to_string(),
                },
                Location::special(),
            )),
            Box::new(Call::new(
                IRVar {
                    name: "*".to_string(),
                },
                vec![
                    IRVar {
                        name: "x1".to_string(),
                    },
                    IRVar {
                        name: "x2".to_string(),
                    },
                ],
                IRVar {
                    name: "x3".to_string(),
                },
                Location::special(),
            )),
            Box::new(Call::new(
                IRVar {
                    name: "+".to_string(),
                },
                vec![
                    IRVar {
                        name: "x0".to_string(),
                    },
                    IRVar {
                        name: "x3".to_string(),
                    },
                ],
                IRVar {
                    name: "x4".to_string(),
                },
                Location::special(),
            )),
        ];

        for (actual, expected) in ir1.iter().zip(expected1.iter()) {
            assert_eq!(actual.as_string(), expected.as_string());
        }

        for (actual, expected) in ir2.iter().zip(expected2.iter()) {
            assert_eq!(actual.as_string(), expected.as_string());
        }
    }

    #[test]
    fn test_gen_if_ir() {
        let source_code = "if true then false";
        let tokens = tokenize(source_code, "file.txt");
        let mut p = Parser::new(tokens);
        let mut parsed = p.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut parsed, &mut st).unwrap();
        let mut root_types: HashMap<IRVar, Type> = HashMap::new();
        for (name, t) in st.locals.iter() {
            root_types.insert(IRVar { name: name.clone() }, t.clone());
        }
        let ir = generate_ir(root_types, parsed).unwrap();

        let ir_string = ir
            .iter()
            .map(|arg| arg.as_string())
            .collect::<Vec<_>>()
            .join("\n");
        let expected = "LoadBoolConst(true, x0)
CondJump(x0, Label(then), Label(if_end))
Label(then)
LoadBoolConst(false, x1)
Label(if_end)"
            .to_string();

        assert_eq!(ir_string, expected);
    }

    #[test]
    fn test_gen_if_else_ir() {
        let source_code = "if true then false else true";
        let tokens = tokenize(source_code, "file.txt");
        let mut p = Parser::new(tokens);
        let mut parsed = p.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut parsed, &mut st).unwrap();
        let mut root_types: HashMap<IRVar, Type> = HashMap::new();
        for (name, t) in st.locals.iter() {
            root_types.insert(IRVar { name: name.clone() }, t.clone());
        }
        let ir = generate_ir(root_types, parsed).unwrap();

        let ir_string = ir
            .iter()
            .map(|arg| arg.as_string())
            .collect::<Vec<_>>()
            .join("\n");

        let expected = "LoadBoolConst(true, x0)
CondJump(x0, Label(then), Label(else))
Label(then)
LoadBoolConst(false, x2)
Copy(x2, x1)
Jump(Label(if_end))
Label(else)
LoadBoolConst(true, x3)
Copy(x3, x1)
Label(if_end)
Call(print_bool, [x1], x4)"
            .to_string();
        assert_eq!(ir_string, expected);
    }

    #[test]
    fn test_gen_while_ir() {
        let source_code = "while true do 1 + 1";
        let tokens = tokenize(source_code, "file.txt");
        let mut p = Parser::new(tokens);
        let mut parsed = p.parse().unwrap();
        let mut st = SymTab::new();
        init_globals(&mut st);
        typecheck(&mut parsed, &mut st).unwrap();
        let mut root_types: HashMap<IRVar, Type> = HashMap::new();
        for (name, t) in st.locals.iter() {
            root_types.insert(IRVar { name: name.clone() }, t.clone());
        }
        let ir = generate_ir(root_types, parsed).unwrap();

        let ir_string = ir
            .iter()
            .map(|arg| arg.as_string())
            .collect::<Vec<_>>()
            .join("\n");

        let expected = "Label(while_start)
LoadBoolConst(true, x0)
CondJump(x0, Label(while_body), Label(while_end))
Label(while_body)
LoadIntConst(1, x1)
LoadIntConst(1, x2)
Call(+, [x1, x2], x3)
Jump(Label(while_start))
Label(while_end)"
            .to_string();
        assert_eq!(ir_string, expected);
    }
}
