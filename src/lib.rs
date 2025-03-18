use std::process::exit;

pub mod compiler;

pub fn run() {
    let source_code = "
        1 + 2 * 3;
    ";
    match compiler::compile(source_code, "file.txt") {
        Ok(ir) => println!("{}", ir),
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}
