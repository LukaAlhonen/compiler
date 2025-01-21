use regex::Regex;
pub mod compiler;

fn regex_test() {
    let re = Regex::new(r"[hH]ell[oO0]").unwrap();

    let helloes = vec!["hello", "Hello", "HELLO", "HellO", "hell0", "Hell0"];

    for hello in helloes {
        if re.is_match(hello) {
            println!("\"{}\" is a match :)", hello);
        } else {
            println!("\"{}\" is not a match :(", hello);
        }
    }
}

fn main() {
    regex_test();
}
