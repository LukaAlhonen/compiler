pub mod compiler;

pub fn run() {
    let source_code = "
    while (a < 10) {
        print(a);
        a = a + 1;
    }
    ";
    compiler::compile(source_code, "file.txt");
}
