use std::any::Any;
use std::fmt::Debug;

pub trait Expression: Debug {
    fn as_any(&self) -> &dyn Any;
    fn eq_expr(&self, other: &dyn Expression) -> bool;
}

impl PartialEq for Box<dyn Expression> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_expr(&**other)
    }
}

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Int(i32),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub struct Literal {
    pub value: Option<LiteralValue>,
}

impl From<i32> for LiteralValue {
    fn from(value: i32) -> Self {
        LiteralValue::Int(value)
    }
}

impl From<bool> for LiteralValue {
    fn from(value: bool) -> Self {
        LiteralValue::Bool(value)
    }
}

impl<T> From<T> for Literal
where
    T: Into<LiteralValue>,
{
    fn from(value: T) -> Self {
        Literal {
            value: Some(value.into()),
        }
    }
}

impl Expression for Literal {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<Literal>()
            .map_or(false, |other| self.value == other.value)
    }
}

#[derive(Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
}

impl Expression for Identifier {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<Identifier>()
            .map_or(false, |other| self.name == other.name)
    }
}

#[derive(Debug)]
pub struct BinaryOp {
    pub left: Box<dyn Expression>,
    pub op: String,
    pub right: Box<dyn Expression>,
}

impl Expression for BinaryOp {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<BinaryOp>()
            .map_or(false, |other| {
                self.left.eq_expr(&*other.left)
                    && self.op == other.op
                    && self.right.eq_expr(&*other.right)
            })
    }
}

impl PartialEq for BinaryOp {
    fn eq(&self, other: &Self) -> bool {
        self.left.eq_expr(&*other.left) && self.op == other.op && self.right.eq_expr(&*other.right)
    }
}

#[derive(Debug)]
pub struct If {
    pub cond: Box<dyn Expression>,
    pub then: Box<dyn Expression>,
    pub if_else: Option<Box<dyn Expression>>, // TODO: Find better name
}

impl Expression for If {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other.as_any().downcast_ref::<If>().map_or(false, |other| {
            let cond_eq = self.cond.eq_expr(&*other.cond);
            let then_eq = self.then.eq_expr(&*other.then);
            let if_else_eq = self
                .if_else
                .as_ref()
                .map_or(other.if_else.is_none(), |self_else| {
                    other
                        .if_else
                        .as_ref()
                        .map_or(false, |other_else| self_else.eq_expr(&**other_else))
                });
            cond_eq && then_eq && if_else_eq
        })
    }
}

impl PartialEq for If {
    fn eq(&self, other: &Self) -> bool {
        let cond_eq = self.cond.eq_expr(&*other.cond);
        let then_eq = self.then.eq_expr(&*other.then);
        let if_else_eq = self
            .if_else
            .as_ref()
            .map_or(other.if_else.is_none(), |self_else| {
                other
                    .if_else
                    .as_ref()
                    .map_or(false, |other_else| self_else.eq_expr(&**other_else))
            });
        cond_eq && then_eq && if_else_eq
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub args: Vec<Box<dyn Expression>>,
}

impl Expression for FunctionCall {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<FunctionCall>()
            .map_or(false, |other| {
                self.name == other.name && self.args == other.args
            })
    }
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Box<dyn Expression>>,
    pub result: Box<dyn Expression>,
}

impl Expression for Block {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<Block>()
            .map_or(false, |other| {
                self.statements == other.statements && self.result.eq_expr(&*other.result)
            })
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        other
            .as_any()
            .downcast_ref::<Block>()
            .map_or(false, |other| {
                self.statements == other.statements && self.result.eq_expr(&*other.result)
            })
    }
}
