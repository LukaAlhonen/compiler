#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Bool,
    Unit,
    FunType {
        args: Vec<Box<Type>>,
        result: Box<Type>,
    },
}

impl Type {
    pub fn as_string(&self) -> String {
        match self {
            Type::Int => "Int".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Unit => "Unit".to_string(),
            Type::FunType { args, result } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.as_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) =>  {}", args_str, result.as_string())
            }
        }
    }
}

// pub trait Type: fmt::Display + fmt::Debug {
//     fn as_any(&self) -> &dyn Any;
//     fn eq_expr(&self, other: &dyn Type) -> bool;
//     fn as_string(&self) -> String;
// }

// impl PartialEq for Box<dyn Type> {
//     fn eq(&self, other: &Self) -> bool {
//         self.eq_expr(&**other)
//     }
// }

// #[derive(PartialEq, Debug, Clone)]
// pub struct Int {}
// impl Type for Int {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn eq_expr(&self, other: &dyn Type) -> bool {
//         other
//             .as_any()
//             .downcast_ref::<Int>()
//             .map_or(false, |other| self.eq_expr(other))
//     }

//     fn as_string(&self) -> String {
//         String::from("Int")
//     }
// }

// impl fmt::Display for Int {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.as_string())
//     }
// }

// #[derive(PartialEq, Debug, Clone)]
// pub struct Bool {}
// impl Type for Bool {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn eq_expr(&self, other: &dyn Type) -> bool {
//         other
//             .as_any()
//             .downcast_ref::<Bool>()
//             .map_or(false, |other| self.eq_expr(other))
//     }

//     fn as_string(&self) -> String {
//         String::from("Bool")
//     }
// }

// impl fmt::Display for Bool {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.as_string())
//     }
// }

// #[derive(PartialEq, Debug, Clone)]
// pub struct Unit {}
// impl Type for Unit {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn eq_expr(&self, other: &dyn Type) -> bool {
//         other
//             .as_any()
//             .downcast_ref::<Unit>()
//             .map_or(false, |other| self.eq_expr(other))
//     }

//     fn as_string(&self) -> String {
//         String::from("Unit")
//     }
// }

// impl fmt::Display for Unit {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.as_string())
//     }
// }

// #[derive(Debug)]
// pub struct FunType {
//     pub params: Vec<Box<dyn Type>>,
//     pub return_type: Box<dyn Type>,
// }

// impl Type for FunType {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn eq_expr(&self, other: &dyn Type) -> bool {
//         other
//             .as_any()
//             .downcast_ref::<FunType>()
//             .map_or(false, |other| self.eq_expr(other))
//     }

//     fn as_string(&self) -> String {
//         let params = self
//             .params
//             .iter()
//             .map(|p| p.to_string())
//             .collect::<Vec<_>>()
//             .join(",");
//         format!("({}) => {}", params, self.return_type.as_string())
//     }
// }

// impl PartialEq for FunType {
//     fn eq(&self, other: &Self) -> bool {
//         other
//             .as_any()
//             .downcast_ref::<FunType>()
//             .map_or(false, |other| self.eq_expr(other))
//     }
// }

// impl fmt::Display for FunType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.as_string())
//     }
// }
