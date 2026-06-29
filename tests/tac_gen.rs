//! Integration tests for the MiniC TAC code generator.

use mini_c::codegen::tac_code_gen::{translate_program, translate_statement, Environment};
use mini_c::ir::ast::{
    CheckedExpr, CheckedProgram, CheckedStmt, Expr, ExprD, FunDecl, Literal, Statement,
    StatementD, Type,
};
use mini_c::ir::tac::{Address, Instruction, Operator};

fn int_var(name: &str) -> CheckedExpr {
    ExprD {
        exp: Expr::Ident(name.to_string()),
        ty: Type::Int,
    }
}

fn int_lit(value: i64) -> CheckedExpr {
    ExprD {
        exp: Expr::Literal(Literal::Int(value)),
        ty: Type::Int,
    }
}

fn add(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD {
        exp: Expr::Add(Box::new(left), Box::new(right)),
        ty: Type::Int,
    }
}

fn lt(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD {
        exp: Expr::Lt(Box::new(left), Box::new(right)),
        ty: Type::Bool,
    }
}

fn assign(name: &str, value: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Assign {
            target: Box::new(ExprD {
                exp: Expr::Ident(name.to_string()),
                ty: value.ty.clone(),
            }),
            value: Box::new(value),
        },
        ty: Type::Unit,
    }
}

fn decl(name: &str, init: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Decl {
            name: name.to_string(),
            ty: init.ty.clone(),
            init: Box::new(init),
        },
        ty: Type::Unit,
    }
}

#[test]
fn test_if_else_with_relational_condition() {
    let stmt = StatementD {
        stmt: Statement::If {
            cond: Box::new(lt(int_var("x"), int_var("y"))),
            then_branch: Box::new(assign("z", add(int_var("x"), int_var("y")))),
            else_branch: Some(Box::new(assign("z", int_var("x")))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let x = Address::Variable("x".to_string(), Type::Int);
    let y = Address::Variable("y".to_string(), Type::Int);
    let z = Address::Variable("z".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(
        instructions,
        vec![
            Instruction::ConditionalJMPRelational(
                Operator::GTE,
                x.clone(),
                y.clone(),
                "Label1:".to_string()
            ),
            Instruction::BinaryAssignment(Operator::Add, temp.clone(), x.clone(), y.clone()),
            Instruction::CopyAssignment(z.clone(), temp),
            Instruction::JMP("Label2:".to_string()),
            Instruction::Label("Label1:".to_string()),
            Instruction::CopyAssignment(z, x),
            Instruction::Label("Label2:".to_string()),
        ]
    );
}

#[test]
fn test_for_simple_counted_loop() {
    let stmt = StatementD {
        stmt: Statement::For {
            init: Some(Box::new(decl("i", int_lit(0)))),
            cond: Some(Box::new(lt(int_var("i"), int_lit(10)))),
            update: Some(Box::new(assign("i", add(int_var("i"), int_lit(1))))),
            body: Box::new(assign("sum", add(int_var("sum"), int_var("i")))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let i = Address::Variable("i".to_string(), Type::Int);
    let sum = Address::Variable("sum".to_string(), Type::Int);
    let zero = Address::Constant(Literal::Int(0), Type::Int);
    let one = Address::Constant(Literal::Int(1), Type::Int);
    let ten = Address::Constant(Literal::Int(10), Type::Int);
    let temp1 = Address::Temporary("temp1".to_string(), Type::Int);
    let temp2 = Address::Temporary("temp2".to_string(), Type::Int);

    assert_eq!(
        instructions,
        vec![
            Instruction::CopyAssignment(i.clone(), zero),
            Instruction::Label("Label1:".to_string()),
            Instruction::ConditionalJMPRelational(Operator::GTE, i.clone(), ten, "Label2:".to_string()),
            Instruction::BinaryAssignment(Operator::Add, temp1.clone(), sum.clone(), i.clone()),
            Instruction::CopyAssignment(sum.clone(), temp1),
            Instruction::BinaryAssignment(Operator::Add, temp2.clone(), i.clone(), one.clone()),
            Instruction::CopyAssignment(i.clone(), temp2),
            Instruction::JMP("Label1:".to_string()),
            Instruction::Label("Label2:".to_string()),
        ]
    );
}

#[test]
fn test_for_zero_iterations_emits_full_loop_structure() {
    let stmt = StatementD {
        stmt: Statement::For {
            init: Some(Box::new(decl("i", int_lit(5)))),
            cond: Some(Box::new(lt(int_var("i"), int_lit(0)))),
            update: Some(Box::new(assign("i", add(int_var("i"), int_lit(1))))),
            body: Box::new(assign("x", int_lit(0))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    assert!(matches!(
        instructions.get(2),
        Some(Instruction::ConditionalJMPRelational(Operator::GTE, _, _, label)) if label == "Label2:"
    ));
    assert!(instructions.iter().any(|i| matches!(
        i,
        Instruction::CopyAssignment(
            Address::Variable(name, Type::Int),
            Address::Constant(Literal::Int(0), Type::Int)
        ) if name == "x"
    )));
    assert_eq!(instructions.last(), Some(&Instruction::Label("Label2:".to_string())));
}

#[test]
fn test_for_infinite_loop_without_condition() {
    let stmt = StatementD {
        stmt: Statement::For {
            init: None,
            cond: None,
            update: None,
            body: Box::new(assign("x", int_lit(1))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::Label("Label1:".to_string()),
            Instruction::CopyAssignment(
                Address::Variable("x".to_string(), Type::Int),
                Address::Constant(Literal::Int(1), Type::Int)
            ),
            Instruction::JMP("Label1:".to_string()),
            Instruction::Label("Label2:".to_string()),
        ]
    );
}

#[test]
fn test_for_program_end_to_end() {
    let program = CheckedProgram {
        functions: vec![FunDecl {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Unit,
            body: Box::new(StatementD {
                stmt: Statement::Block {
                    seq: vec![
                        decl("sum", int_lit(0)),
                        StatementD {
                            stmt: Statement::For {
                                init: Some(Box::new(decl("i", int_lit(0)))),
                                cond: Some(Box::new(lt(int_var("i"), int_lit(3)))),
                                update: Some(Box::new(assign(
                                    "i",
                                    add(int_var("i"), int_lit(1)),
                                ))),
                                body: Box::new(assign("sum", add(int_var("sum"), int_var("i")))),
                            },
                            ty: Type::Unit,
                        },
                    ],
                },
                ty: Type::Unit,
            }),
        }],
    };

    let instructions = translate_program(program);
    assert!(instructions.first().is_some_and(|i| matches!(i, Instruction::Label(l) if l == "main")));
    assert!(instructions.iter().any(|i| matches!(
        i,
        Instruction::ConditionalJMPRelational(Operator::GTE, _, _, _)
    )));
    assert!(instructions.iter().filter(|i| matches!(i, Instruction::JMP(_))).count() >= 1);
}
