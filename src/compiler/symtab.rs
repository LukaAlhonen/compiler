use std::collections::HashMap;
use std::fmt::Debug;

#[derive(PartialEq, Debug)]
pub struct SymTab<'a, T: Clone + Debug> {
    pub locals: HashMap<String, T>,
    parent: Option<&'a SymTab<'a, T>>,
}

impl<'a, T: Clone + Debug> SymTab<'a, T> {
    pub fn new() -> Self {
        SymTab {
            locals: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_with_parent(parent: &'a SymTab<'a, T>) -> Self {
        SymTab {
            locals: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn insert<S: Into<String>>(&mut self, name: S, var_type: T) {
        self.locals.insert(name.into(), var_type);
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        if let Some(var_type) = self.locals.get(name) {
            return Some(var_type);
        }

        if let Some(parent) = &self.parent {
            return parent.get(name);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Type;

    #[test]
    fn test_symtable() {
        let mut symtab = SymTab::new();
        symtab.insert("a", Type::Int);
        symtab.insert("b", Type::Bool);
        symtab.insert("e", Type::Int);

        assert_eq!(symtab.get("a"), Some(&Type::Int));
        assert_eq!(symtab.get("b"), Some(&Type::Bool));
        assert_eq!(symtab.get("e"), Some(&Type::Int));
        {
            let mut child_symtab = SymTab::new_with_parent(&symtab);
            child_symtab.insert("c", Type::Int);
            child_symtab.insert("d", Type::Bool);
            child_symtab.insert("e", Type::Bool);

            assert_eq!(child_symtab.get("c"), Some(&Type::Int));
            assert_eq!(child_symtab.get("d"), Some(&Type::Bool));
            assert_eq!(child_symtab.get("a"), Some(&Type::Int));
            assert_eq!(child_symtab.get("b"), Some(&Type::Bool));
            assert_eq!(child_symtab.get("e"), Some(&Type::Bool));
            {
                let mut grandchild_symtab = SymTab::new_with_parent(&child_symtab);

                grandchild_symtab.insert("f", Type::Int);
                grandchild_symtab.insert("g", Type::Bool);

                assert_eq!(grandchild_symtab.get("f"), Some(&Type::Int));
                assert_eq!(grandchild_symtab.get("g"), Some(&Type::Bool));
                assert_eq!(grandchild_symtab.get("e"), Some(&Type::Bool));
                assert_eq!(grandchild_symtab.get("a"), Some(&Type::Int));
            }
        }

        symtab.insert(
            "print_int",
            Type::FunType {
                args: vec![Box::new(Type::Int)],
                result: Box::new(Type::Unit),
            },
        );

        assert_eq!(
            symtab.get("print_int"),
            Some(&Type::FunType {
                args: vec![Box::new(Type::Int)],
                result: Box::new(Type::Unit)
            })
        );
    }
}
