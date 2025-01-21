pub mod location;
pub mod token;
pub mod tokenizer;

pub fn compile(source_code: String) {
    tokenizer::tokenize(source_code);
}
