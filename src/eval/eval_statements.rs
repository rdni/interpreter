use std::sync::{Arc, Mutex};

use crate::fatal_error;
use crate::runtime::values::{FunctionValue, NullValue, RuntimeValue};
use crate::frontend::ast::{Expr, FunctionDeclaration, IfStmt, Program, ReturnStmt, VarDeclaration};

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
        declaration_env: Arc::clone(&env),
        body: function_declaration.body
    };

    env.lock().unwrap().declare_var(function.name.clone(), Box::new(function), true);

    return Box::new(NullValue {});
}

pub fn eval_return(return_stmt: ReturnStmt, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    if env.lock().unwrap().is_global() {
        fatal_error("Cannot use return statement outside of function.");
    }

    let return_value = eval(return_stmt.value.to_stmt_from_expr(), Arc::clone(&env));

    env.lock().unwrap().continue_interpreting = false;
    
    return_value
}

pub fn eval_if(if_stmt: IfStmt, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let condition = eval(if_stmt.condition.to_stmt_from_expr(), Arc::clone(&env));

    if condition.as_bool() {
        for stmt in if_stmt.body {
            eval(stmt, Arc::clone(&env));
        }
    }

    Box::new(NullValue {})
}