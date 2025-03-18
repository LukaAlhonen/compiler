pub mod compiler;

pub fn run() {
    let source_code = "
        if true then false else true
    ";
    match compiler::compile(source_code, "file.txt") {
        Ok(ir) => println!("{}", ir),
        Err(e) => eprint!("{}", e),
    }
}
