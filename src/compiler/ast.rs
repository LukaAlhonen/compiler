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
    pub value: LiteralValue,
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
            value: value.into(),
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
