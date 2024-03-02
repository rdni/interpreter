use crate::{fatal_error, runtime::values::NullValue};
use std::{thread, time::{Duration, SystemTime}};

use super::{environment::Environment, values::{NumberValue, RuntimeValue, ValueType}};

pub fn native_print(args: Vec<Box<dyn RuntimeValue>>, _env: &mut Environment) -> Box<dyn RuntimeValue> {
    let mut to_print = String::new();

    for arg in args {
        to_print.push_str(&arg.to_string());
        to_print.push(' ');
    }

    println!("{}", to_print);

    Box::new(NullValue {})
}

pub fn native_time(_args: Vec<Box<dyn RuntimeValue>>, _env: &mut Environment) -> Box<dyn RuntimeValue> {
    return Box::new(NumberValue {
        value: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64()
    });
}

pub fn native_sleep(args: Vec<Box<dyn RuntimeValue>>, _env: &mut Environment) -> Box<dyn RuntimeValue> {
    if args.len() != 1 {
        fatal_error(&format!("Expected 1 argument, found {}.", args.len()));
    }

    if args[0].get_type() != ValueType::Number {
        fatal_error(&format!("Expected number, found {}", args[0].get_type()))
    }

    let number = args[0].as_any().downcast_ref::<NumberValue>().unwrap().clone();

    thread::sleep(Duration::from_secs_f64(number.value));

    Box::new(NullValue {})
}