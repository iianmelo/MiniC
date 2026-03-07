//! Program parser for MiniC.

use crate::ir::ast::{Program, UncheckedProgram};
use crate::parser::functions::fun_decl;
use nom::{combinator::map, multi::many0, IResult};

/// Parse a complete MiniC program: zero or more function declarations.
/// Execution starts at the `main` function (validated by the type checker).
pub fn program(input: &str) -> IResult<&str, UncheckedProgram> {
    map(many0(fun_decl), |functions| Program { functions })(input)
}
