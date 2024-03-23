use std::{any::Any, collections::HashMap, fmt::{Debug, Display}, rc::Rc, sync::{Arc, Mutex}};

use crate::{fatal_error, frontend::ast::Body, pad_each_line, runtime::interpreter::eval};

use super::environment::Environment;

#[derive(PartialEq, Debug)]
pub enum ValueType {
    Null,
    Number,
    String,
    Boolean,
    Object,
    NativeFn,
    Function
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean => write!(f, "bool"),
            Self::NativeFn => write!(f, "native_func"),
            Self::Null => write!(f, "null"),
            Self::Number => write!(f, "number"),
            Self::Object => write!(f, "object"),
            Self::String => write!(f, "string"),
            Self::Function => write!(f, "function")
        }?;

        Ok(())
    }
}

pub trait RuntimeValue: Debug + Any + 'static {
    fn get_type(&self) -> ValueType;
    fn as_any(&self) -> &dyn Any;
    fn clone_self(&self) -> Box<dyn RuntimeValue>;
    fn to_string(&self) -> String;
    fn as_bool(&self) -> bool;
    fn equals(&self, other: Box<dyn RuntimeValue>) -> bool;
    fn less_than(&self, _other: Box<dyn RuntimeValue>) -> bool {
        fatal_error("Cannot compare with this operator");
    }
    fn greater_than(&self, _other: Box<dyn RuntimeValue>) -> bool {
        fatal_error("Cannot compare with this operator");
    }
}

impl Clone for Box<dyn RuntimeValue> {
    fn clone(&self) -> Self {
        self.clone_self()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BooleanValue {
    pub value: bool
}

impl RuntimeValue for BooleanValue {
    fn get_type(&self) -> ValueType {
        ValueType::Boolean
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_self(&self) -> Box<dyn RuntimeValue> {
        Box::new(self.clone())
    }
    fn to_string(&self) -> String {
        if self.value {
            String::from("true")
        } else {
            String::from("false")
        }
    }
    fn as_bool(&self) -> bool {
        self.value
    }
    fn equals(&self, other: Box<dyn RuntimeValue>) -> bool {
        self.value == other.as_any().downcast_ref::<BooleanValue>().unwrap().value
    }
}

#[derive(Debug, Clone)]
pub struct NullValue {}

impl RuntimeValue for NullValue {
    fn get_type(&self) -> ValueType {
        ValueType::Null
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_self(&self) -> Box<dyn RuntimeValue> {
        Box::new(self.clone())
    }
    fn to_string(&self) -> String {
        String::from("null")
    }
    fn as_bool(&self) -> bool {
        false
    }
    fn equals(&self, _other: Box<dyn RuntimeValue>) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NumberValue {
    pub value: f64
}

impl RuntimeValue for NumberValue {
    fn get_type(&self) -> ValueType {
        ValueType::Number
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_self(&self) -> Box<dyn RuntimeValue> {
        Box::new(self.clone())
    }
    fn to_string(&self) -> String {
        if self.value.fract() == 0.0 {
            String::from(self.value.to_string().replace(".0", ""))
        } else {
            String::from(self.value.to_string())
        }
    }
    fn as_bool(&self) -> bool {
        self.value != 0.0
    }
    fn equals(&self, other: Box<dyn RuntimeValue>) -> bool {
        self.value == other.as_any().downcast_ref::<NumberValue>().unwrap().value
    }
    fn greater_than(&self, other: Box<dyn RuntimeValue>) -> bool {
        self.value > other.as_any().downcast_ref::<NumberValue>().unwrap().value
    }
    fn less_than(&self, other: Box<dyn RuntimeValue>) -> bool {
        self.value < other.as_any().downcast_ref::<NumberValue>().unwrap().value
    }
}

#[derive(Debug, Clone)]
pub struct ObjectValue {
    pub properties: HashMap<String, Box<dyn RuntimeValue>>
}

impl RuntimeValue for ObjectValue {
    fn get_type(&self) -> ValueType {
        ValueType::Object
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_self(&self) -> Box<dyn RuntimeValue> {
        Box::new(self.clone())
    }
    fn to_string(&self) -> String {
        let mut value = String::new();

        value.push('{');
        value.push('\n');

        for property in self.properties.iter() {
            value += &pad_each_line(4, format!("{}: {}", property.0, property.1.to_string()));
            value.push('\n');
        }
        
        value.push('}');
        
        value
    }
    fn as_bool(&self) -> bool {
        self.properties.len() != 0
    }
    fn equals(&self, other: Box<dyn RuntimeValue>) -> bool {
        let other = other.as_any().downcast_ref::<ObjectValue>().unwrap();
        if self.properties.len() != other.properties.len() {
            return false;
        }
        self.to_string() == other.to_string()
    }
}

pub struct FunctionCall {
    pub func: Rc<dyn Fn(Vec<Box<dyn RuntimeValue>>, &Mutex<Environment>) -> Box<dyn RuntimeValue> + 'static>,
}

impl Clone for FunctionCall {
    fn clone(&self) -> Self {
        Self {
            func: Rc::clone(&self.func)
        }
    }
}

impl Debug for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionCall")
    }
}

#[derive(Debug, Clone)]
pub struct NativeFnValue {
    pub call: FunctionCall,
}

impl RuntimeValue for NativeFnValue {
    fn get_type(&self) -> ValueType {
        ValueType::NativeFn
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_self(&self) -> Box<dyn RuntimeValue> {
        Box::new(self.clone())
    }
    fn to_string(&self) -> String {
        String::from("NativeFn")
    }
    fn as_bool(&self) -> bool {
        true
    }
    fn equals(&self, other: Box<dyn RuntimeValue>) -> bool {
        Rc::ptr_eq(&self.call.func, &other.as_any().downcast_ref::<NativeFnValue>().unwrap().call.func)
    }
}

#[derive(Debug, Clone)]
pub struct StringValue {
    pub value: String
}

impl RuntimeValue for StringValue {
    fn get_type(&self) -> ValueType {
        ValueType::String
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_self(&self) -> Box<dyn RuntimeValue> {
        Box::new(self.clone())
    }
    fn to_string(&self) -> String {
        self.value.clone()
    }
    fn as_bool(&self) -> bool {
        self.value.len() != 0
    }
    fn equals(&self, other: Box<dyn RuntimeValue>) -> bool {
        self.value == other.as_any().downcast_ref::<StringValue>().unwrap().value
    }
}

#[derive(Debug)]
pub struct FunctionValue {
    pub name: String,
    pub parameters: Vec<String>,
    pub declaration_env: Arc<Mutex<Environment>>,
    pub body: Body
}

impl FunctionValue {
    pub fn call(&self, env: Arc<Mutex<Environment>>, args: Vec<Box<dyn RuntimeValue>>) -> Box<dyn RuntimeValue> {
        let new_env = Arc::new(Mutex::new(Environment::new(Some(Arc::clone(&env)))));

        if args.len() != self.parameters.len() {
            fatal_error(&format!("Expected {} arguments, found {}", self.parameters.len(), args.len()));
        }

        for i in 0..(self.parameters.len()) {
            new_env.lock().unwrap().declare_var(self.parameters[i].clone(), args.get(i).unwrap().clone(), false);
        }

        let mut last_evaluated: Box<dyn RuntimeValue> = Box::new(NullValue {});
        for stmt in self.body.body.clone() {
            if new_env.lock().unwrap().continue_interpreting {
                last_evaluated = eval(stmt, Arc::clone(&new_env));
            } else {
                break;
            }
        }

        return last_evaluated;
    }
}

impl RuntimeValue for FunctionValue {
    fn get_type(&self) -> ValueType {
        ValueType::Function
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_self(&self) -> Box<dyn RuntimeValue> {
        Box::new(self.clone())
    }
    fn to_string(&self) -> String {
        self.name.clone()
    }
    fn as_bool(&self) -> bool {
        true
    }
    fn equals(&self, other: Box<dyn RuntimeValue>) -> bool {
        let other = other.as_any().downcast_ref::<FunctionValue>().unwrap();

        Arc::ptr_eq(&other.declaration_env, &self.declaration_env)
    }
}

impl Clone for FunctionValue {
    fn clone(&self) -> Self {
        FunctionValue {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            declaration_env: Arc::clone(&self.declaration_env),
            body: self.body.clone()
        }
    }
}