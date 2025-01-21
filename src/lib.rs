pub mod compiler;

pub fn run() {
    let source_code = String::from(
        "
    while (a < 10) {
        print(a);
        a = a + 1;
    }
    ",
    );
    compiler::compile(source_code);
}
