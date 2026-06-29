use crate::ir::ast::{Literal, Name, Type};

type Label = String;

pub type TACProgram = Vec<Instruction>;

#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Variable(Name, Type),
    Constant(Literal, Type),
    Temporary(Name, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Label(Label),
    CopyAssignment(Address, Address),
    UnaryAssignment(Operator, Address, Address),
    BinaryAssignment(Operator, Address, Address, Address),
    JMP(Label),
    ConditionalJMP(Address, Label),
    ConditionalJMPFalse(Address, Label),
    ConditionalJMPRelational(Operator, Address, Address, Label),
    Param(Address),
    Call(Option<Address>, Name, usize),
    Store(Address, Address, Address),
    Load(Address, Address, Address),
    Return(Option<Address>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    LT,
    LTE,
    GT,
    GTE,
    EQ,
    NE,
    SL,
    SR,
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Variable(name, _) => write!(f, "{name}"),
            Address::Temporary(name, _) => write!(f, "{name}"),
            Address::Constant(lit, _) => match lit {
                Literal::Int(n) => write!(f, "{n}"),
                Literal::Float(x) => write!(f, "{x}"),
                Literal::Str(s) => write!(f, "\"{s}\""),
                Literal::Bool(b) => write!(f, "{b}"),
            },
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Label(label) => write!(f, "{label}"),
            Instruction::CopyAssignment(dest, src) => write!(f, "{dest} = {src}"),
            Instruction::UnaryAssignment(op, dest, src) => {
                write!(f, "{dest} = {op} {src}")
            }
            Instruction::BinaryAssignment(op, dest, left, right) => {
                write!(f, "{dest} = {left} {op} {right}")
            }
            Instruction::JMP(label) => write!(f, "goto {label}"),
            Instruction::ConditionalJMP(addr, label) => write!(f, "if {addr} goto {label}"),
            Instruction::ConditionalJMPFalse(addr, label) => {
                write!(f, "if_false {addr} goto {label}")
            }
            Instruction::ConditionalJMPRelational(op, left, right, label) => {
                write!(f, "if {left} {op} {right} goto {label}")
            }
            Instruction::Param(addr) => write!(f, "param {addr}"),
            Instruction::Call(result, name, arity) => match result {
                Some(dest) => write!(f, "{dest} = call {name}, {arity}"),
                None => write!(f, "call {name}, {arity}"),
            },
            Instruction::Store(base, index, value) => write!(f, "{base}[{index}] = {value}"),
            Instruction::Load(dest, base, index) => write!(f, "{dest} = {base}[{index}]"),
            Instruction::Return(value) => match value {
                Some(addr) => write!(f, "return {addr}"),
                None => write!(f, "return"),
            },
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Neg => "-",
            Operator::LT => "<",
            Operator::LTE => "<=",
            Operator::GT => ">",
            Operator::GTE => ">=",
            Operator::EQ => "==",
            Operator::NE => "!=",
            Operator::SL => "<<",
            Operator::SR => ">>",
        };
        write!(f, "{s}")
    }
}
