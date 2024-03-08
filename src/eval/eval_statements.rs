use std::sync::{Arc, Mutex};

use crate::runtime::values::{FunctionValue, NullValue, RuntimeValue};
use crate::frontend::ast::{Expr, FunctionDeclaration, Program, VarDeclaration};

use crate::runtime::interpreter::eval;
use crate::runtime::environment::Environment;

pub fn eval_program(program: Program, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let mut last_evaluated: Box<dyn RuntimeValue> = Box::new(NullValue {});

    for statement in program.body {
        last_evaluated = eval(statement, Arc::clone(&env));
    }
    
    last_evaluated
}

pub fn eval_var_declaration(var_declaration: VarDeclaration, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let value = eval(var_declaration.value.unwrap().to_stmt_from_expr(), Arc::clone(&env));
    env.lock().unwrap().declare_var(var_declaration.identifier, value, var_declaration.constant)
}

pub fn eval_function_declaration(function_declaration: FunctionDeclaration, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let function = FunctionValue {
        name: function_declaration.name,
        parameters: function_declaration.parameters,
        declaration_env: env,
        body: function_declaration.body
    };

    return Box::new(function);
}