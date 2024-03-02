use std::{any::Any, collections::HashMap, fmt::Debug, rc::Rc};

use crate::pad_each_line;

use super::environment::Environment;

#[derive(PartialEq, Debug)]
pub enum ValueType {
    Null,
    Number,
    Boolean,
    Object,
    NativeFn
}

pub trait RuntimeValue: Debug + Any + 'static {
    fn get_type(&self) -> ValueType;
    fn as_any(&self) -> &dyn Any;
    fn clone_self(&self) -> Box<dyn RuntimeValue>;
    fn to_string(&self) -> String;
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
}

pub struct FunctionCall {
    pub func: Rc<dyn Fn(Vec<Box<dyn RuntimeValue>>, &mut Environment) -> Box<dyn RuntimeValue> + 'static>,
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
}