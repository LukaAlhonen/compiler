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
                let l_end = new_label(ins, "if_end", Location::special()); // need to figure out what loc to assign here

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

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::compiler::tokenizer;
//     use crate::compiler::parser::*;
//     use crate::compiler::type_checker;

//     #[test]
//     fn test_gen_if_ir() {

//     }
// }
