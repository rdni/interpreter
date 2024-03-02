use crate::runtime::values::NullValue;

use super::{environment::Environment, values::RuntimeValue};

pub fn native_print(args: Vec<Box<dyn RuntimeValue>>, env: &mut Environment) -> Box<dyn RuntimeValue> {
    let mut to_print = String::new();

    for arg in args {
        to_print.push_str(&arg.to_string());
    }

    println!("{}", to_print);

    Box::new(NullValue {})
}