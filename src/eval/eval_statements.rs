use std::sync::{Arc, Mutex};

use crate::fatal_error;
use crate::runtime::values::{FunctionValue, ListValue, NullValue, RuntimeValue, ValueType};
use crate::frontend::ast::{Expr, ForStmt, FunctionDeclaration, Identifier, IfStmt, Program, ReturnStmt, Stmt, VarDeclaration, WhileStmt};

use crate::runtime::interpreter::eval;
use crate::runtime::environment::{Environment, SharedEnvironment};

pub fn eval_program(program: Program, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    program.body.run(env, false).0
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
        if_stmt.body.run(env, true);
    } else if let Some(v) = if_stmt.else_stmt {
        v.run(env, true);
    }

    Box::new(NullValue {})
}

pub fn eval_while(while_stmt: WhileStmt, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let mut last_env = Arc::clone(&env);
    while eval(while_stmt.condition.to_stmt_from_expr(), last_env).as_bool() == true {
        last_env = while_stmt.body.run(Arc::clone(&env), true).1;
    }

    Box::new(NullValue {})
}

pub fn eval_for(for_stmt: ForStmt, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let iterable = eval(for_stmt.iterable.to_stmt_from_expr(), Arc::clone(&env));
    
    if iterable.get_type() == ValueType::List {
        let iterable = iterable.as_any().downcast_ref::<ListValue>().unwrap().clone();

        if iterable.elements.len() == 0 {
            return Box::new(NullValue {});
        } else {
            let ident = for_stmt.variable.as_any().downcast_ref::<Identifier>().expect("Expected identifier in for loop").clone().symbol;

            let mut index = 0;

            let mut parent_env = SharedEnvironment(Arc::clone(&env));

            while index != iterable.elements.len() {
                parent_env.assign_var(ident.clone(), iterable.elements[index].clone(), true);
                
                for_stmt.body.run(Arc::clone(&env), true);

                index += 1;
            }
        }
    } else {
        fatal_error("Cannot iterate over non-iterable thing (duh)");
    }

    Box::new(NullValue {})
}