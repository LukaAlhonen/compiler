pub mod ast;
pub mod ir;
pub mod ir_generator;
pub mod parser;
pub mod symtab;
pub mod tokenizer;
pub mod type_checker;
pub mod types;

use std::collections::HashMap;

use anyhow::Error;
use ir::*;
use ir_generator::generate_ir;
use parser::Parser;
use symtab::SymTab;
use type_checker::*;
use types::Type;

pub fn compile(source_code: &str, file_name: &str) -> Result<String, Error> {
    let tokens = tokenizer::tokenize(source_code, file_name);
    let mut p = Parser::new(tokens);
    let mut parsed = p.parse()?;
    let mut st = SymTab::new();
    init_globals(&mut st);
    typecheck(&mut parsed, &mut st)?;
    let mut root_types: HashMap<IRVar, Type> = HashMap::new();
    for (name, t) in st.locals.iter() {
        root_types.insert(IRVar { name: name.clone() }, t.clone());
    }
    let ir = generate_ir(root_types, parsed)?;

    Ok(ir
        .iter()
        .map(|arg| arg.as_string())
        .collect::<Vec<_>>()
        .join("\n"))
}
