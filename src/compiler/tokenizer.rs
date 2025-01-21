use regex::Regex;

pub fn tokenize(source_code: String) -> Vec<String> {
    let re = Regex::new(r"([a-zA-Z_]+[a-zA-Z0-9]*)|([0-9]+)|(-)").unwrap();

    re.find_iter(&source_code)
        .map(|mat| mat.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_tokenize() {
        assert_eq!(
            tokenize(String::from("if 3\nwhile")),
            vec!["if", "3", "while"]
        );
        assert_ne!(
            tokenize(String::from("if 3a\nwhile")),
            vec!["if", "3", "while"]
        );
        assert_eq!(
            tokenize(String::from("int a = 3\n1 2 3")),
            vec!["int", "a", "3", "1", "2", "3"]
        );
        assert_ne!(
            tokenize(String::from("int 1a = 3\n1 2 3")),
            vec!["int", "a", "3", "1", "2", "3"]
        );
        assert_eq!(tokenize(String::from("3-2")), vec!["3", "-", "2"]);
    }
}
