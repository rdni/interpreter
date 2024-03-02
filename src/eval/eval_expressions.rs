use std::collections::HashMap;

use crate::{error, fatal_error};
use crate::runtime::values::{NativeFnValue, NullValue, NumberValue, ObjectValue, RuntimeValue, StringValue, ValueType};
use crate::frontend::ast::{AssignmentExpr, BinaryExpr, CallExpr, Expr, Identifier, MemberExpr, NodeType, ObjectLiteral, Stmt};
use crate::runtime::environment::Environment;
use crate::runtime::interpreter::eval;

pub fn eval_binop_expr(binop: BinaryExpr, env: &mut Environment) -> Box<dyn RuntimeValue> {
    let lhs = eval(binop.left.to_stmt_from_expr(), env);
    let rhs = eval(binop.right.to_stmt_from_expr(), env);

    if lhs.get_type() == ValueType::Number && rhs.get_type() == ValueType::Number {
        let lhs = lhs.as_any().downcast_ref::<NumberValue>().expect("Failed to downcast to NumberValue");
        let rhs = rhs.as_any().downcast_ref::<NumberValue>().expect("Failed to downcast to NumberValue");
        eval_numeric_binary_expr(*lhs, *rhs, binop.operator)
    } else if lhs.get_type() == ValueType::String && rhs.get_type() == ValueType::String {
        let lhs = lhs.as_any().downcast_ref::<StringValue>().expect("Failed to downcast to StringValue");
        let rhs = rhs.as_any().downcast_ref::<StringValue>().expect("Failed to downcast to StringValue");
        eval_string_binary_expr(lhs.clone(), rhs.clone(), binop.operator)
    } else if (lhs.get_type() == ValueType::String && rhs.get_type() == ValueType::Number) || (rhs.get_type() == ValueType::String && lhs.get_type() == ValueType::Number) {
        let string;
        let number;
        if lhs.get_type() == ValueType::String {
            string = lhs.as_any().downcast_ref::<StringValue>().expect("Failed to downcast to StringValue").clone();
            number = rhs.as_any().downcast_ref::<NumberValue>().expect("Failed to downcast to NumberValue").clone();
        } else {
            string = rhs.as_any().downcast_ref::<StringValue>().expect("Failed to downcast to StringValue").clone();
            number = lhs.as_any().downcast_ref::<NumberValue>().expect("Failed to downcast to NumberValue").clone();
        }

        eval_string_numeric_binary_expr(string, number, binop.operator)
    } else{
        Box::new(NullValue {})
    }
}

pub fn eval_numeric_binary_expr(lhs: NumberValue, rhs: NumberValue, operator: String) -> Box<NumberValue> {
    match &*operator {
        "+" => Box::new(NumberValue { value: lhs.value + rhs.value }),
        "-" => Box::new(NumberValue { value: lhs.value - rhs.value }),
        "*" => Box::new(NumberValue { value: lhs.value * rhs.value }),
        "/" => Box::new(NumberValue { value: lhs.value / rhs.value }),
        "%" => Box::new(NumberValue { value: lhs.value % rhs.value }),
        _ => Box::new(NumberValue { value: 0.0 })
    }
}

pub fn eval_string_binary_expr(lhs: StringValue, rhs: StringValue, operator: String) -> Box<StringValue> {
    match &*operator {
        "+" => Box::new(StringValue { value: lhs.value + &rhs.value }),
        _ => {
            error("Invalid operator between string and string");
            Box::new(StringValue { value: String::from("") })
        }
    }
}

pub fn eval_string_numeric_binary_expr(string: StringValue, number: NumberValue, operator: String) -> Box<StringValue> {
    match &*operator {
        "+" => Box::new(StringValue { value: string.value + &number.value.to_string() }),
        "*" => Box::new(StringValue { value: string.value.repeat(number.value as usize) }),
        _ => {
            error("Invalid operator between string and number");
            Box::new(StringValue { value: String::from("") })
        }
    }
}

pub fn eval_identifier(identifier: Identifier, env: &mut Environment) -> Box<dyn RuntimeValue> {
    env.lookup_var(identifier.symbol)
}

pub fn eval_assignment(node: AssignmentExpr, env: &mut Environment) -> Box<dyn RuntimeValue> {
    match node.assignee.get_kind() {
        NodeType::Identifier => {
            let identifier = node.assignee.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.").clone();
            let value = eval(node.value.to_stmt_from_expr(), env);
            env.assign_var(identifier.symbol, value)
        },
        NodeType::MemberExpr => {
            let member_expr = node.assignee.as_any().downcast_ref::<MemberExpr>().expect("Failed to downcast to MemberExpr.").clone();
            let object_identifier = member_expr.object.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.").clone();
            let property_identifer = member_expr.property.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.").clone();
            let mut obj = env.lookup_var(object_identifier.symbol.clone()).as_any().downcast_ref::<ObjectValue>().expect("Failed to downcast to ObjectValue.").clone();
            let value = eval(node.value.to_stmt_from_expr(), env);

            obj.properties.insert(property_identifer.symbol, value);
            env.assign_var(object_identifier.symbol, Box::new(obj))
        },
        _ => {
            fatal_error(&format!("Invalid LHS inside assignment expression: {:?}", node.assignee));
        }
    }
}

pub fn eval_object_expr(obj: ObjectLiteral, env: &mut Environment) -> Box<dyn RuntimeValue> {
    let mut object = ObjectValue { properties: HashMap::<String, Box<dyn RuntimeValue>>::new() };

    for i in obj.properties {
        if let Some(value) = i.value {
            object.properties.insert(i.key.unwrap(), eval(value.to_stmt_from_expr(), env));
        } else {
            object.properties.insert(i.key.clone().unwrap(), env.lookup_var(i.key.unwrap()));
        }
    }

    return Box::new(object);
}

pub fn eval_member_expr(node: MemberExpr, env: &mut Environment) -> Box<dyn RuntimeValue> {
    let obj = eval(node.object.to_stmt_from_expr(), env).as_any().downcast_ref::<ObjectValue>().unwrap().clone();
    if !node.computed {
        if node.property.get_expr_kind() != NodeType::Identifier {
            fatal_error("Unexpected value found in member expression.");
        }
        let identifier = node.property.as_any().downcast_ref::<Identifier>().unwrap().clone();

        return obj.properties.get(&identifier.symbol).unwrap().clone();
    }

    let property = eval(node.property.to_stmt_from_expr(), env);

    if property.get_type() != ValueType::String {
        fatal_error("Unexpected value found in member expression.");
    }

    let property = property.as_any().downcast_ref::<StringValue>().expect("Failed to downcast to StringValue.");

    obj.properties.get(&property.value).unwrap().clone()
}

pub fn eval_call(expr: CallExpr, env: &mut Environment) -> Box<dyn RuntimeValue> {
    let mut evaluated_args = vec![];

    for arg in expr.args {
        evaluated_args.push(eval(arg.to_stmt_from_expr(), env));
    }

    let func = eval(expr.caller.to_stmt_from_expr(), env);

    if func.get_type() == ValueType::NativeFn {
        let func = func.as_any().downcast_ref::<NativeFnValue>().expect("Failed to downcast to NativeFnValue.").clone();
        return (func.call.func)(evaluated_args, env);
    }

    fatal_error(&format!("Cannot call {:?}", func.get_type()));
}