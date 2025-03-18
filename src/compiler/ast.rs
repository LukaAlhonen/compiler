use super::tokenizer::Location;
use super::types::Type;
use std::any::Any;
use std::fmt::Debug;

pub trait Expression: Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn eq_expr(&self, other: &dyn Expression) -> bool;
    fn is_block(&self) -> bool; // Kind of weird hack but allows for easy checking if expression ends in "}"
    fn get_loc(&self) -> &Location;
    fn set_type(&mut self, t: Type);
    fn get_type(&self) -> &Type;
}

impl PartialEq for Box<dyn Expression> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_expr(&**other)
    }
}

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Int(i8),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub struct Literal {
    pub value: Option<LiteralValue>,
    pub loc: Location,
    pub t: Type,
}

impl From<i8> for LiteralValue {
    fn from(value: i8) -> Self {
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
            t: Type::Unit,
        }
    }

    pub fn new_with_type<T>(value: T, loc: Location, t: Type) -> Self
    where
        T: Into<LiteralValue>,
    {
        Literal {
            value: Some(value.into()),
            loc,
            t,
        }
    }

    pub fn none(loc: Location) -> Self {
        Literal {
            value: None,
            loc,
            t: Type::Unit,
        }
    }
}

impl Expression for Literal {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
    }
}

#[derive(Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub loc: Location,
    pub t: Type,
}

impl Identifier {
    pub fn new<S: Into<String>>(name: S, loc: Location) -> Self {
        Identifier {
            name: name.into(),
            loc,
            t: Type::Unit,
        }
    }

    pub fn new_with_type<S: Into<String>>(name: S, loc: Location, t: Type) -> Self {
        Identifier {
            name: name.into(),
            loc,
            t,
        }
    }
}

impl Expression for Identifier {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
    }
}

#[derive(Debug)]
pub struct BinaryOp {
    pub left: Box<dyn Expression>,
    pub op: String,
    pub right: Box<dyn Expression>,
    pub loc: Location,
    pub t: Type,
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
            t: Type::Unit,
        }
    }

    pub fn new_with_type<S: Into<String>>(
        left: Box<dyn Expression>,
        op: S,
        right: Box<dyn Expression>,
        t: Type,
    ) -> Self {
        BinaryOp {
            loc: left.get_loc().clone(),
            left,
            op: op.into(),
            right,
            t,
        }
    }
}

impl Expression for BinaryOp {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
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
    pub then_branch: Box<dyn Expression>,
    pub else_branch: Option<Box<dyn Expression>>, // TODO: Find better name
    pub loc: Location,
    pub t: Type,
}

impl If {
    pub fn new(
        cond: Box<dyn Expression>,
        then_branch: Box<dyn Expression>,
        else_branch: Option<Box<dyn Expression>>,
        loc: Location,
    ) -> Self {
        If {
            cond,
            then_branch,
            else_branch,
            loc,
            t: Type::Unit,
        }
    }

    pub fn new_with_type(
        cond: Box<dyn Expression>,
        then_branch: Box<dyn Expression>,
        else_branch: Option<Box<dyn Expression>>,
        loc: Location,
        t: Type,
    ) -> Self {
        If {
            cond,
            then_branch,
            else_branch,
            loc,
            t,
        }
    }
}

impl Expression for If {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn eq_expr(&self, other: &dyn Expression) -> bool {
        other.as_any().downcast_ref::<If>().map_or(false, |other| {
            let cond_eq = self.cond.eq_expr(&*other.cond);
            let then_branch_eq = self.then_branch.eq_expr(&*other.then_branch);
            let else_branch_eq =
                self.else_branch
                    .as_ref()
                    .map_or(other.else_branch.is_none(), |self_else| {
                        other
                            .else_branch
                            .as_ref()
                            .map_or(false, |other_else| self_else.eq_expr(&**other_else))
                    });
            cond_eq && then_branch_eq && else_branch_eq
        })
    }

    fn is_block(&self) -> bool {
        if let Some(else_branch) = &self.else_branch {
            else_branch.is_block()
        } else {
            self.then_branch.is_block()
        }
    }

    fn get_loc(&self) -> &Location {
        &self.loc
    }

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
    }
}

impl PartialEq for If {
    fn eq(&self, other: &Self) -> bool {
        let cond_eq = self.cond.eq_expr(&*other.cond);
        let then_branch_eq = self.then_branch.eq_expr(&*other.then_branch);
        let else_branch_eq =
            self.else_branch
                .as_ref()
                .map_or(other.else_branch.is_none(), |self_else| {
                    other
                        .else_branch
                        .as_ref()
                        .map_or(false, |other_else| self_else.eq_expr(&**other_else))
                });
        cond_eq && then_branch_eq && else_branch_eq
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub args: Vec<Box<dyn Expression>>,
    pub loc: Location,
    pub t: Type,
}

impl FunctionCall {
    pub fn new(name: Identifier, args: Vec<Box<dyn Expression>>) -> Self {
        FunctionCall {
            loc: name.get_loc().clone(),
            name,
            args,
            t: Type::Unit,
        }
    }

    pub fn new_with_type(name: Identifier, args: Vec<Box<dyn Expression>>, t: Type) -> Self {
        FunctionCall {
            loc: name.get_loc().clone(),
            name,
            args,
            t,
        }
    }
}

impl Expression for FunctionCall {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
    }
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Box<dyn Expression>>,
    pub loc: Location,
    pub t: Type,
}

impl Block {
    pub fn new(statements: Vec<Box<dyn Expression>>, loc: Location) -> Self {
        Block {
            statements,
            loc,
            t: Type::Unit,
        }
    }

    pub fn new_with_type(statements: Vec<Box<dyn Expression>>, loc: Location, t: Type) -> Self {
        Block { statements, loc, t }
    }
}

impl Expression for Block {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
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
    pub declared_type: Option<Type>,
    pub initializer: Box<dyn Expression>,
    pub loc: Location,
    pub t: Type,
}

impl VarDeclaration {
    pub fn new(
        var: Identifier,
        declared_type: Option<Type>,
        initializer: Box<dyn Expression>,
    ) -> Self {
        VarDeclaration {
            loc: var.get_loc().clone(),
            var,
            initializer,
            declared_type,
            t: Type::Unit,
        }
    }

    pub fn new_with_type(
        var: Identifier,
        declared_type: Option<Type>,
        initializer: Box<dyn Expression>,
        t: Type,
    ) -> Self {
        VarDeclaration {
            loc: var.get_loc().clone(),
            var,
            initializer,
            declared_type,
            t,
        }
    }
}

impl Expression for VarDeclaration {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
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
    pub t: Type,
}

impl While {
    pub fn new(cond: Box<dyn Expression>, body: Box<dyn Expression>, loc: Location) -> Self {
        While {
            cond,
            body,
            loc,
            t: Type::Unit,
        }
    }
}

impl Expression for While {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        drop(t);
    }

    fn get_type(&self) -> &Type {
        &self.t
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
    pub t: Type,
}

impl UnaryOp {
    pub fn new<S: Into<String>>(op: S, right: Box<dyn Expression>, loc: Location) -> Self {
        UnaryOp {
            op: op.into(),
            right,
            loc,
            t: Type::Unit,
        }
    }

    pub fn new_with_type<S: Into<String>>(
        op: S,
        right: Box<dyn Expression>,
        loc: Location,
        t: Type,
    ) -> Self {
        UnaryOp {
            op: op.into(),
            right,
            loc,
            t,
        }
    }
}

impl Expression for UnaryOp {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn set_type(&mut self, t: Type) {
        self.t = t;
    }

    fn get_type(&self) -> &Type {
        &self.t
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
