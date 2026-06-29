use crate::ir::ast::{
    CheckedExpr, CheckedFunDecl, CheckedProgram, CheckedStmt, Expr, Literal, Statement, Type,
};
use crate::ir::tac::{Address, Instruction, Operator, TACProgram};

#[derive(Clone)]
pub struct Environment {
    current_label: usize,
    current_temporary: usize,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_label: 0,
            current_temporary: 0,
        }
    }

    fn new_label(&mut self) -> String {
        self.current_label += 1;
        format!("Label{}:", self.current_label)
    }

    fn new_temporary(&mut self) -> String {
        self.current_temporary += 1;
        format!("temp{}", self.current_temporary)
    }
}

pub fn translate_program(program: CheckedProgram) -> TACProgram {
    let mut env = Environment::new();
    let main_fn = program.main_function();
    match main_fn {
        None => unreachable!("[Impossible] program must have a main function"),
        Some(f) => translate_function(f.clone(), &mut env),
    }
}

fn translate_function(function: CheckedFunDecl, env: &mut Environment) -> TACProgram {
    let mut instructions = if let Statement::Block { seq: stmts } = function.body.stmt {
        stmts
            .into_iter()
            .flat_map(|stmt| translate_statement(stmt, env))
            .collect::<Vec<_>>()
    } else {
        translate_statement(*function.body, env)
    };
    instructions.insert(0, Instruction::Label(function.name.clone()));
    instructions
}

pub fn translate_statement(statement: CheckedStmt, env: &mut Environment) -> Vec<Instruction> {
    match statement.stmt {
        Statement::Block { seq } => seq
            .into_iter()
            .flat_map(|s| translate_statement(s, env))
            .collect(),
        Statement::Decl { name, ty, init } => {
            let var_address = Address::Variable(name, ty);
            let (expression_address, instructions) = translate_expression(*init, env);
            let mut res = instructions;
            res.push(Instruction::CopyAssignment(var_address, expression_address));
            res
        }
        Statement::Assign { target, value } => {
            if let Expr::Ident(name) = &target.exp {
                let var_type = target.ty.clone();
                let var_address = Address::Variable(name.clone(), var_type);
                let (expression_address, instructions) = translate_expression(*value, env);
                let mut res = instructions;
                res.push(Instruction::CopyAssignment(var_address, expression_address));
                res
            } else {
                todo!("indexed assignment TAC not yet implemented")
            }
        }
        Statement::Call { name, args } => {
            let addresses_and_instructions = args
                .into_iter()
                .map(|expr| translate_expression(expr, env))
                .collect::<Vec<_>>();
            let mut instructions = addresses_and_instructions
                .iter()
                .fold(vec![], |mut acc, (_, inst)| {
                    acc.extend(inst.clone());
                    acc
                });

            for (addr, _) in &addresses_and_instructions {
                instructions.push(Instruction::Param(addr.clone()));
            }
            instructions.push(Instruction::Call(
                None,
                name,
                addresses_and_instructions.len(),
            ));
            instructions
        }
        Statement::If {
            cond,
            then_branch: then_body,
            else_branch: Some(else_body),
        } => {
            let label_else = env.new_label();
            let label_end = env.new_label();
            let mut instructions = translate_conditional(*cond, env, label_else.clone());
            instructions.extend(translate_statement(*then_body, env));
            instructions.push(Instruction::JMP(label_end.clone()));
            instructions.push(Instruction::Label(label_else));
            instructions.extend(translate_statement(*else_body, env));
            instructions.push(Instruction::Label(label_end));
            instructions
        }
        Statement::If {
            cond,
            then_branch,
            else_branch: None,
        } => {
            let label_end = env.new_label();
            let mut instructions = translate_conditional(*cond, env, label_end.clone());
            instructions.extend(translate_statement(*then_branch, env));
            instructions.push(Instruction::Label(label_end));
            instructions
        }
        Statement::While { cond, body } => {
            let label_test = env.new_label();
            let label_end = env.new_label();
            let mut instructions = vec![Instruction::Label(label_test.clone())];
            instructions.extend(translate_conditional(*cond, env, label_end.clone()));
            instructions.extend(translate_statement(*body, env));
            instructions.push(Instruction::JMP(label_test));
            instructions.push(Instruction::Label(label_end));
            instructions
        }
        Statement::For {
            init,
            cond,
            update,
            body,
        } => {
            let label_test = env.new_label();
            let label_end = env.new_label();
            let mut instructions = Vec::new();

            if let Some(init_stmt) = init {
                instructions.extend(translate_statement(*init_stmt, env));
            }

            instructions.push(Instruction::Label(label_test.clone()));

            if let Some(c) = cond {
                instructions.extend(translate_conditional(*c, env, label_end.clone()));
            }

            instructions.extend(translate_statement(*body, env));

            if let Some(update_stmt) = update {
                instructions.extend(translate_statement(*update_stmt, env));
            }

            instructions.push(Instruction::JMP(label_test));
            instructions.push(Instruction::Label(label_end));
            instructions
        }
        Statement::Return(expr) => match expr {
            None => vec![Instruction::Return(None)],
            Some(e) => {
                let (addr, mut instructions) = translate_expression(*e, env);
                instructions.push(Instruction::Return(Some(addr)));
                instructions
            }
        },
    }
}

fn translate_expression(expression: CheckedExpr, env: &mut Environment) -> (Address, Vec<Instruction>) {
    match expression.exp {
        Expr::Literal(value) => (Address::Constant(value, expression.ty), vec![]),
        Expr::Ident(name) => (
            Address::Variable(name, expression.ty),
            vec![],
        ),
        Expr::Not(exp) => {
            let (addr, mut instructions) = translate_expression(*exp, env);
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            instructions.push(Instruction::ConditionalJMPFalse(
                addr,
                label_false.clone(),
            ));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(false), Type::Bool),
            ));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        }
        Expr::Or(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let label_true = env.new_label();
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMPFalse(
                l_addr,
                label_false.clone(),
            ));
            instructions.push(Instruction::JMP(label_true.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.extend(r_instructions);
            instructions.push(Instruction::ConditionalJMP(r_addr, label_true.clone()));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(false), Type::Bool),
            ));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_true));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        }
        Expr::And(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMPFalse(
                l_addr,
                label_false.clone(),
            ));
            instructions.extend(r_instructions);
            instructions.push(Instruction::ConditionalJMPFalse(
                r_addr,
                label_false.clone(),
            ));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(false), Type::Bool),
            ));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        }
        Expr::Add(left, right) => binary_arithmetic(Operator::Add, *left, *right, expression.ty, env),
        Expr::Sub(left, right) => binary_arithmetic(Operator::Sub, *left, *right, expression.ty, env),
        Expr::Mul(left, right) => binary_arithmetic(Operator::Mul, *left, *right, expression.ty, env),
        Expr::Div(left, right) => binary_arithmetic(Operator::Div, *left, *right, expression.ty, env),
        Expr::Neg(exp) => {
            let (addr, mut instructions) = translate_expression(*exp, env);
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::UnaryAssignment(
                Operator::Neg,
                temp.clone(),
                addr,
            ));
            (temp, instructions)
        }
        _ => todo!("expression TAC not yet implemented: {:?}", expression.exp),
    }
}

fn binary_arithmetic(
    op: Operator,
    left: CheckedExpr,
    right: CheckedExpr,
    result_ty: Type,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    let (l_addr, l_instructions) = translate_expression(left, env);
    let (r_addr, r_instructions) = translate_expression(right, env);
    let mut instructions = l_instructions;
    instructions.extend(r_instructions);
    let temp = Address::Temporary(env.new_temporary(), result_ty);
    instructions.push(Instruction::BinaryAssignment(
        op,
        temp.clone(),
        l_addr,
        r_addr,
    ));
    (temp, instructions)
}

fn translate_conditional(
    expression: CheckedExpr,
    env: &mut Environment,
    false_label: String,
) -> Vec<Instruction> {
    match expression.exp {
        Expr::Literal(Literal::Bool(true)) => vec![],
        Expr::Literal(Literal::Bool(false)) => vec![Instruction::JMP(false_label)],
        Expr::Ident(name) => {
            let addr = Address::Variable(name, expression.ty);
            vec![Instruction::ConditionalJMPFalse(addr, false_label)]
        }
        Expr::And(left, right) => {
            let label_right = env.new_label();
            let mut instructions = translate_conditional(*left, env, false_label.clone());
            instructions.push(Instruction::Label(label_right));
            instructions.extend(translate_conditional(*right, env, false_label));
            instructions
        }
        Expr::Or(left, right) => {
            let label_skip = env.new_label();
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMP(l_addr, label_skip.clone()));
            instructions.extend(translate_conditional(*right, env, false_label));
            instructions.push(Instruction::Label(label_skip));
            instructions
        }
        Expr::Not(expr) => {
            let (addr, mut instructions) = translate_expression(*expr, env);
            instructions.push(Instruction::ConditionalJMP(addr, false_label));
            instructions
        }
        Expr::Lt(left, right) => {
            translate_relational(*left, *right, Operator::LT, false_label, env)
        }
        Expr::Le(left, right) => {
            translate_relational(*left, *right, Operator::LTE, false_label, env)
        }
        Expr::Gt(left, right) => {
            translate_relational(*left, *right, Operator::GT, false_label, env)
        }
        Expr::Ge(left, right) => {
            translate_relational(*left, *right, Operator::GTE, false_label, env)
        }
        Expr::Eq(left, right) => {
            translate_relational(*left, *right, Operator::EQ, false_label, env)
        }
        Expr::Ne(left, right) => {
            translate_relational(*left, *right, Operator::NE, false_label, env)
        }
        _ => {
            let (addr, mut instructions) = translate_expression(expression, env);
            instructions.push(Instruction::ConditionalJMPFalse(addr, false_label));
            instructions
        }
    }
}

fn negate_op(op: Operator) -> Operator {
    match op {
        Operator::LT => Operator::GTE,
        Operator::LTE => Operator::GT,
        Operator::GT => Operator::LTE,
        Operator::GTE => Operator::LT,
        Operator::EQ => Operator::NE,
        Operator::NE => Operator::EQ,
        other => other,
    }
}

fn translate_relational(
    left: CheckedExpr,
    right: CheckedExpr,
    op: Operator,
    false_label: String,
    env: &mut Environment,
) -> Vec<Instruction> {
    let (l_addr, l_instructions) = translate_expression(left, env);
    let (r_addr, r_instructions) = translate_expression(right, env);
    let mut instructions = l_instructions;
    instructions.extend(r_instructions);
    instructions.push(Instruction::ConditionalJMPRelational(
        negate_op(op),
        l_addr,
        r_addr,
        false_label,
    ));
    instructions
}
