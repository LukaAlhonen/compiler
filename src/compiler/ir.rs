use super::tokenizer::Location;
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct IRVar {
    pub name: String,
}

impl IRVar {
    pub fn new<S: Into<String>>(name: S) -> Self {
        IRVar { name: name.into() }
    }

    pub fn as_string(&self) -> String {
        format!("{}", self.name)
    }
}

pub trait Instruction: Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_string(&self) -> String;
}

impl PartialEq for Box<dyn Instruction> {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Label {
    pub name: String,
    pub loc: Location,
}

impl Label {
    pub fn new<S: Into<String>>(name: S, loc: Location) -> Self {
        Label {
            name: name.into(),
            loc,
        }
    }
}

impl Instruction for Label {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_string(&self) -> String {
        format!("Label({})", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct LoadBoolConst {
    pub value: bool,
    pub dest: IRVar,
    pub loc: Location,
}

impl LoadBoolConst {
    pub fn new(value: bool, dest: IRVar, loc: Location) -> Self {
        LoadBoolConst { value, dest, loc }
    }
}

impl Instruction for LoadBoolConst {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_string(&self) -> String {
        format!("LoadBoolConst({}, {})", self.value, self.dest.as_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct LoadIntConst {
    pub value: i8,
    pub dest: IRVar,
    pub loc: Location,
}

impl LoadIntConst {
    pub fn new(value: i8, dest: IRVar, loc: Location) -> Self {
        LoadIntConst { value, dest, loc }
    }
}

impl Instruction for LoadIntConst {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_string(&self) -> String {
        format!("LoadIntConst({}, {})", self.value, self.dest.as_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct Copy {
    pub source: IRVar,
    pub dest: IRVar,
    pub loc: Location,
}

impl Copy {
    pub fn new(source: IRVar, dest: IRVar, loc: Location) -> Self {
        Copy { source, dest, loc }
    }
}

impl Instruction for Copy {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_string(&self) -> String {
        format!(
            "Copy({}, {})",
            self.source.as_string(),
            self.dest.as_string()
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Call {
    pub fun: IRVar,
    pub args: Vec<IRVar>,
    pub dest: IRVar,
    pub loc: Location,
}

impl Call {
    pub fn new(fun: IRVar, args: Vec<IRVar>, dest: IRVar, loc: Location) -> Self {
        Call {
            fun,
            args,
            dest,
            loc,
        }
    }
}

impl Instruction for Call {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_string(&self) -> String {
        let args_str = self
            .args
            .iter()
            .map(|arg| arg.as_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Call({}, [{}], {})",
            self.fun.as_string(),
            args_str,
            self.dest.as_string()
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Jump {
    pub label: Label,
    pub loc: Location,
}

impl Jump {
    pub fn new(label: Label, loc: Location) -> Self {
        Jump { label, loc }
    }
}

impl Instruction for Jump {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_string(&self) -> String {
        format!("Jump({})", self.label.as_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct CondJump {
    pub cond: IRVar,
    pub then_label: Label,
    pub else_label: Label,
    pub loc: Location,
}

impl CondJump {
    pub fn new(cond: IRVar, then_label: Label, else_label: Label, loc: Location) -> Self {
        CondJump {
            cond,
            then_label,
            else_label,
            loc,
        }
    }
}

impl Instruction for CondJump {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_string(&self) -> String {
        format!(
            "CondJump({}, {}, {})",
            self.cond.as_string(),
            self.then_label.as_string(),
            self.else_label.as_string()
        )
    }
}
