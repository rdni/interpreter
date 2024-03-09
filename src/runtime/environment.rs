use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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

#[derive(Debug, Clone)]
pub struct Environment {
    parent: Option<Arc<Mutex<Environment>>>,
    variables: HashMap<String, Box<dyn RuntimeValue>>,
    constants: Vec<String>,
    position: usize,
    pub continue_interpreting: bool
}

impl Environment {
    pub fn new(parent_param: Option<Arc<Mutex<Environment>>>) -> Self {
        let parent;
        if let Some(parent_env) = parent_param {
            parent = Some(parent_env);
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
            constants: Vec::new(),
            position: 0,
            continue_interpreting: true
        };

        if global {
            setup_scope(&mut env)
        }

        env
    }

    pub fn is_global(&self) -> bool {
        if let None = self.parent {
            true
        } else {
            false
        }
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
}

// Entirely here because I can't be bothered to make a better solution
pub enum EnvResolveResult<'a> {
    MutRef(&'a mut Environment),
    ArcMutex(Arc<Mutex<Environment>>)
}

pub struct SharedEnvironment(pub Arc<Mutex<Environment>>);


impl SharedEnvironment {
    pub fn resolve(&mut self, varname: &String) -> Arc<Mutex<Environment>> {
        let inner = &self.0;
        if inner.lock().unwrap().variables.contains_key(varname) {
            Arc::clone(&inner)
        } else {
            let mut parent = SharedEnvironment(match &inner.lock().unwrap().parent {
                Some(v) => Arc::clone(&v),
                None => fatal_error(&format!("Error resolving variable {}", varname))
            });
            parent.resolve(varname)
        }
    }

    pub fn lookup_var(&mut self, varname: String) -> Box<dyn RuntimeValue> {
        let env = self.resolve(&varname);
        let x = env.lock().unwrap().variables.get(&varname).unwrap().clone();
        x
    }

    pub fn assign_var(&mut self , varname: String, value: Box<dyn RuntimeValue>) -> Box<dyn RuntimeValue> {
        let env = self.resolve(&varname);

        let is_constant = env.lock().unwrap().get_constants().contains(&varname);

        if !is_constant {
            fatal_error("Cannot re-assign a constant variable.");
        }

        env.lock().unwrap().variables.insert(varname, value.clone_self());

        value
    }
}