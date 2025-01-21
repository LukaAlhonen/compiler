#[derive(Debug)]
pub struct Location {
    file: String,
    line: String,
    column: String,
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.is_special()
            || other.is_special()
            || (self.file == other.file && self.line == other.line && self.column == other.column)
    }
}

impl Location {
    pub fn special() -> Self {
        Location {
            file: String::from("SPECIAL"),
            line: String::from("SPECIAL"),
            column: String::from("SPECIAL"),
        }
    }

    fn is_special(&self) -> bool {
        self.file == String::from("SPECIAL")
            && self.line == String::from("SPECIAL")
            && self.column == String::from("SPECIAL")
    }
}
