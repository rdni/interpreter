use std::collections::HashMap;
use std::rc::Rc;

use crate::fatal_error;

use super::values::{BooleanValue, FunctionCall, NativeFnValue, NullValue, RuntimeValue};
use super::native_funcs::{native_exit, native_input, native_print, native_sleep, native_time};

pub fn setup_scope(env: &mut Environment) {
    env.declare_var(String::from("null"), Box::new(NullValue {}), true);
    env.declare_var(String::from("true"), Box::new(BooleanValue { value: true }), true);
    env.declare_var(String::from("false"), Box::new(BooleanValue { value: false }), true);

    env.declare_var(String::from("print"), Box::new(NativeFnValue {
        call: FunctionCall {
            func: Rc::new(native_print)
        }
    }), true);
    env.declare_var(String::from("time"), Box::new(NativeFnValue {
        call: FunctionCall {
            func: Rc::new(native_time)
        }
    }), true);
    env.declare_var(String::from("sleep"), Box::new(NativeFnValue {
        call: FunctionCall {
            func: Rc::new(native_sleep)
        }
    }), true);
    env.declare_var(String::from("input"), Box::new(NativeFnValue {
        call: FunctionCall {
            func: Rc::new(native_input)
        }
    }), true);
    env.declare_var(String::from("exit"), Box::new(NativeFnValue {
        call: FunctionCall {
            func: Rc::new(native_exit)
        }
    }), true);
}

pub struct Environment {
    parent: Option<Box<Environment>>,
    variables: HashMap<String, Box<dyn RuntimeValue>>,
    constants: Vec<String>
}

impl Environment {
    pub fn new(parent_param: Option<Environment>) -> Self {
        let parent;
        if let Some(parent_env) = parent_param {
            parent = Some(Box::new(parent_env));
        } else {
            parent = None;
        }

        let global;
        if let None = parent {
            global = true;
        } else {
            global = false;
        }

        let mut env = Environment {
            parent,
            variables: HashMap::new(),
            constants: Vec::new()
        };

        if global {
            setup_scope(&mut env)
        }

        env
    }
    
    pub fn get_constants(&self) -> &Vec<String> {
        &self.constants
    }

    pub fn declare_var(&mut self, varname: String, value: Box<dyn RuntimeValue>, constant: bool) -> Box<dyn RuntimeValue> {
        if self.variables.contains_key(&varname) {
            fatal_error(&format!("Cannot declare variable {} as it is already defined.", varname));
        }

        if constant {
            self.constants.insert(0, varname.clone());
        }
        self.variables.insert(varname, value.clone_self());

        value
    }

    pub fn assign_var(&mut self , varname: String, value: Box<dyn RuntimeValue>) -> Box<dyn RuntimeValue> {
        let env = self.resolve(&varname);

        if env.get_constants().contains(&varname) {
            fatal_error("Cannot re-assign a constant variable.");
        }

        env.variables.insert(varname, value.clone_self());

        value
    }

    pub fn lookup_var(&mut self, varname: String) -> Box<dyn RuntimeValue> {
        let env = self.resolve(&varname);
        (*env.variables.get(&varname).unwrap()).clone()
    }

    pub fn resolve(&mut self, varname: &String) -> &mut Environment {
        if self.variables.contains_key(varname) {
            self
        } else {
            if self.parent.is_none() {
                fatal_error(&format!("Cannot resolve {}", varname));
            } else {
                self.parent.as_mut().unwrap().resolve(varname)
            }
        }
    }
}