pub mod ast;
pub mod parser;
pub mod tokenizer;

pub fn compile(source_code: &str, file_name: &str) {
    tokenizer::tokenize(source_code, file_name);
}
