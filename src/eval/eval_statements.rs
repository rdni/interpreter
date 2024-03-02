use crate::runtime::values::{NullValue, RuntimeValue};
use crate::frontend::ast::{Expr, Program, VarDeclaration};

use crate::runtime::interpreter::eval;
use crate::runtime::environment::Environment;

pub fn eval_program(program: Program, env: &mut Environment) -> Box<dyn RuntimeValue> {
    let mut last_evaluated: Box<dyn RuntimeValue> = Box::new(NullValue {});

    for statement in program.body {
        last_evaluated = eval(statement, env);
    }
    
    last_evaluated
}

pub fn eval_var_declaration(var_declaration: VarDeclaration, env: &mut Environment) -> Box<dyn RuntimeValue> {
    let value = eval(var_declaration.value.unwrap().to_stmt_from_expr(), env);
    env.declare_var(var_declaration.identifier, value, var_declaration.constant)
}