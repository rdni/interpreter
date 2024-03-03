use crate::{fatal_error, runtime::values::NullValue};
use std::{io, process::exit, thread, time::{Duration, SystemTime}};

use super::{environment::Environment, values::{NumberValue, RuntimeValue, StringValue, ValueType}};

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

pub fn native_input(args: Vec<Box<dyn RuntimeValue>>, _env: &mut Environment) -> Box<dyn RuntimeValue> {
    if args.len() > 1 {
        fatal_error(&format!("Expected less than 2 arguments, found {}", args.len()));
    }

    if args.len() == 1 {
        if args[0].get_type() == ValueType::String {
            print!("{}", args[0].as_any().downcast_ref::<StringValue>().expect("Failed to downcast to StringValue.").to_string());
        }
    }

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .unwrap();

    // Remove the \n or \r at end
    let mut chars = input.chars();
    chars.next_back();
    input = String::from(chars.as_str());

    Box::new(StringValue { value: input })

}

pub fn native_exit(args: Vec<Box<dyn RuntimeValue>>, _env: &mut Environment) -> Box<dyn RuntimeValue> {
    let mut code = 0;

    if args.len() == 1 {
        if args[0].get_type() == ValueType::Number {
            code = args[0].as_any().downcast_ref::<NumberValue>().unwrap().value as i32;
        } else {
            fatal_error(&format!("Expected number, found {}", args[0].get_type()));
        }
    }

    exit(code);
}