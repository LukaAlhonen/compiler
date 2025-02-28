use super::tokenizer::Location;
use std::any::Any;
use std::fmt::Debug;

pub trait Expression: Debug {
    fn as_any(&self) -> &dyn Any;
    fn eq_expr(&self, other: &dyn Expression) -> bool;
    fn is_block(&self) -> bool; // Kind of weird hack but allows for easy checking if expression ends in "}"
    fn get_loc(&self) -> &Location;
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
pub enum VarType {
    Int,
    Bool,
    Unit,
}

#[derive(Debug, PartialEq)]
pub struct Literal {
    pub value: Option<LiteralValue>,
    pub loc: Location,
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

impl Literal {
    pub fn new<T>(value: T, loc: Location) -> Self
    where
        T: Into<LiteralValue>,
    {
        Literal {
            value: Some(value.into()),
            loc,
        }
    }

    pub fn none(loc: Location) -> Self {
        Literal { value: None, loc }
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

    fn is_block(&self) -> bool {
        false
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }
}

#[derive(Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub loc: Location,
}

impl Identifier {
    pub fn new<S: Into<String>>(name: S, loc: Location) -> Self {
        Identifier {
            name: name.into(),
            loc,
        }
    }
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

    fn is_block(&self) -> bool {
        false
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }
}

#[derive(Debug)]
pub struct BinaryOp {
    pub left: Box<dyn Expression>,
    pub op: String,
    pub right: Box<dyn Expression>,
    pub loc: Location,
}

impl BinaryOp {
    pub fn new<S: Into<String>>(
        left: Box<dyn Expression>,
        op: S,
        right: Box<dyn Expression>,
    ) -> Self {
        BinaryOp {
            loc: left.get_loc().clone(),
            left,
            op: op.into(),
            right,
        }
    }
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

    fn is_block(&self) -> bool {
        self.right.is_block()
    }

    fn get_loc(&self) -> &Location {
        &self.loc
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
    pub loc: Location,
}

impl If {
    pub fn new(
        cond: Box<dyn Expression>,
        then: Box<dyn Expression>,
        if_else: Option<Box<dyn Expression>>,
        loc: Location,
    ) -> Self {
        If {
            cond,
            then,
            if_else,
            loc,
        }
    }
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

    fn is_block(&self) -> bool {
        if let Some(if_else) = &self.if_else {
            if_else.is_block()
        } else {
            self.then.is_block()
        }
    }

    fn get_loc(&self) -> &Location {
        &self.loc
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
    pub loc: Location,
}

impl FunctionCall {
    pub fn new(name: Identifier, args: Vec<Box<dyn Expression>>) -> Self {
        FunctionCall {
            loc: name.get_loc().clone(),
            name,
            args,
        }
    }
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

    fn is_block(&self) -> bool {
        false
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Box<dyn Expression>>,
    pub loc: Location,
}

impl Block {
    pub fn new(statements: Vec<Box<dyn Expression>>, loc: Location) -> Self {
        Block { statements, loc }
    }
}

impl Expression for Block {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<Block>()
            .map_or(false, |other| self.statements == other.statements)
    }

    fn is_block(&self) -> bool {
        true
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        other
            .as_any()
            .downcast_ref::<Block>()
            .map_or(false, |other| self.statements == other.statements)
    }
}

#[derive(Debug)]
pub struct VarDeclaration {
    pub var: Identifier,
    pub declared_type: Option<VarType>,
    pub initializer: Box<dyn Expression>,
    pub loc: Location,
}

impl VarDeclaration {
    pub fn new(
        var: Identifier,
        declared_type: Option<VarType>,
        initializer: Box<dyn Expression>,
    ) -> Self {
        VarDeclaration {
            loc: var.get_loc().clone(),
            var,
            initializer,
            declared_type,
        }
    }
}

impl Expression for VarDeclaration {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<VarDeclaration>()
            .map_or(false, |other| {
                self.var.eq_expr(&other.var)
                    && self.initializer.eq_expr(&*other.initializer)
                    && self.declared_type == other.declared_type
            })
    }

    fn is_block(&self) -> bool {
        self.initializer.is_block()
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }
}

impl PartialEq for VarDeclaration {
    fn eq(&self, other: &Self) -> bool {
        other
            .as_any()
            .downcast_ref::<VarDeclaration>()
            .map_or(false, |other| {
                self.var.eq_expr(&other.var)
                    && self.initializer.eq_expr(&*other.initializer)
                    && self.declared_type == other.declared_type
            })
    }
}

#[derive(Debug)]
pub struct While {
    pub cond: Box<dyn Expression>,
    pub body: Box<dyn Expression>,
    pub loc: Location,
}

impl While {
    pub fn new(cond: Box<dyn Expression>, body: Box<dyn Expression>, loc: Location) -> Self {
        While { cond, body, loc }
    }
}

impl Expression for While {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<While>()
            .map_or(false, |other| {
                self.cond.eq_expr(&*other.cond) && self.body.eq_expr(&*other.body)
            })
    }

    fn is_block(&self) -> bool {
        self.body.is_block()
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }
}

impl PartialEq for While {
    fn eq(&self, other: &Self) -> bool {
        other
            .as_any()
            .downcast_ref::<While>()
            .map_or(false, |other| {
                self.cond.eq_expr(&*other.cond) && self.body.eq_expr(&*other.body)
            })
    }
}

#[derive(Debug)]
pub struct UnaryOp {
    pub op: String,
    pub right: Box<dyn Expression>,
    pub loc: Location,
}

impl UnaryOp {
    pub fn new<S: Into<String>>(op: S, right: Box<dyn Expression>, loc: Location) -> Self {
        UnaryOp {
            op: op.into(),
            right,
            loc,
        }
    }
}

impl Expression for UnaryOp {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other
            .as_any()
            .downcast_ref::<UnaryOp>()
            .map_or(false, |other| {
                self.right.eq_expr(&*other.right) && self.op == other.op
            })
    }

    fn is_block(&self) -> bool {
        self.right.is_block()
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }
}

impl PartialEq for UnaryOp {
    fn eq(&self, other: &Self) -> bool {
        other
            .as_any()
            .downcast_ref::<UnaryOp>()
            .map_or(false, |other| {
                self.right.eq_expr(&*other.right) && self.op == other.op
            })
    }
}
