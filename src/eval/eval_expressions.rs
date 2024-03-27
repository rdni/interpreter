use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{error, fatal_error, MK_BOOL, MK_NULL, MK_NUMBER, MK_STRING};
use crate::runtime::values::{BooleanValue, FunctionValue, ListValue, NativeFnValue, NullValue, NumberValue, ObjectValue, RuntimeValue, StringValue, ValueType};
use crate::frontend::ast::{AssignmentExpr, BinaryExpr, CallExpr, ComparativeExpr, Expr, Identifier, ListLiteral, MemberExpr, NodeType, ObjectLiteral, Stmt};
use crate::runtime::environment::{Environment, SharedEnvironment};
use crate::runtime::interpreter::eval;

pub fn eval_binop_expr(binop: BinaryExpr, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let lhs = eval(binop.left.to_stmt_from_expr(), Arc::clone(&env));
    let rhs = eval(binop.right.to_stmt_from_expr(), Arc::clone(&env));

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
        Box::new(MK_NULL!())
    }
}

pub fn eval_numeric_binary_expr(lhs: NumberValue, rhs: NumberValue, operator: String) -> Box<NumberValue> {
    match &*operator {
        "+" => Box::new(MK_NUMBER!(lhs.value + rhs.value)),
        "-" => Box::new(MK_NUMBER!(lhs.value - rhs.value)),
        "*" => Box::new(MK_NUMBER!(lhs.value * rhs.value)),
        "/" => Box::new(MK_NUMBER!(lhs.value / rhs.value)),
        "%" => Box::new(MK_NUMBER!(lhs.value % rhs.value)),
        _ => Box::new(MK_NUMBER!(0.0))
    }
}

pub fn eval_string_binary_expr(lhs: StringValue, rhs: StringValue, operator: String) -> Box<StringValue> {
    match &*operator {
        "+" => Box::new(MK_STRING!(lhs.value + &rhs.value)),
        _ => {
            error("Invalid operator between string and string");
            Box::new(MK_STRING!(String::from("") ))
        }
    }
}

pub fn eval_string_numeric_binary_expr(string: StringValue, number: NumberValue, operator: String) -> Box<StringValue> {
    match &*operator {
        "+" => Box::new(MK_STRING!(string.value + &number.value.to_string())),
        "*" => Box::new(MK_STRING!(string.value.repeat(number.value as usize))),
        _ => {
            error("Invalid operator between string and number");
            Box::new(MK_STRING!(String::from("")))
        }
    }
}

pub fn eval_comp_expr(comp: ComparativeExpr, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let left = eval(comp.left.to_stmt_from_expr(), Arc::clone(&env));
    let right = eval(comp.right.to_stmt_from_expr(), Arc::clone(&env));
    match &*comp.operator {
        "==" => {
            if left.get_type() != right.get_type() {
                Box::new(MK_BOOL!(false))
            } else {
                if left.equals(right) {
                    Box::new(MK_BOOL!(true))
                } else {
                    Box::new(MK_BOOL!(false))
                }
            }
        },
        ">" => {
            if left.get_type() != right.get_type() {
                Box::new(MK_BOOL!(false))
            } else {
                if left.greater_than(right) {
                    Box::new(MK_BOOL!(true))
                } else {
                    Box::new(MK_BOOL!(false))
                }
            }
        },
        "<" => {
            if left.get_type() != right.get_type() {
                Box::new(MK_BOOL!(false))
            } else {
                if left.less_than(right) {
                    Box::new(MK_BOOL!(true))
                } else {
                    Box::new(MK_BOOL!(false))
                }
            }
        },
        ">=" => {
            if left.get_type() != right.get_type() {
                Box::new(MK_BOOL!(false))
            } else {
                if left.greater_than(right.clone()) || left.equals(right) {
                    Box::new(MK_BOOL!(true))
                } else {
                    Box::new(MK_BOOL!(false))
                }
            }
        },
        "<=" => {
            if left.get_type() != right.get_type() {
                Box::new(MK_BOOL!(false))
            } else {
                if left.less_than(right.clone()) || left.equals(right) {
                    Box::new(MK_BOOL!(true))
                } else {
                    Box::new(MK_BOOL!(false))
                }
            }
        },
        "!=" => {
            if left.get_type() != right.get_type() {
                Box::new(MK_BOOL!(true))
            } else {
                if left.equals(right.clone()) {
                    Box::new(MK_BOOL!(false))
                } else {
                    Box::new(MK_BOOL!(true))
                }
            }
        }
        _ => {
            error("Invalid operator in comparative expression.");
            Box::new(NullValue {})
        }
    }
}

pub fn eval_identifier(identifier: Identifier, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    SharedEnvironment(env).lookup_var(identifier.symbol)
}

pub fn eval_assignment(node: AssignmentExpr, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let mut shared_env = SharedEnvironment(Arc::clone(&env));
    match node.assignee.get_kind() {
        NodeType::Identifier => {
            let identifier = node.assignee.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.").clone();
            let value = eval(node.value.to_stmt_from_expr(), Arc::clone(&env));
            shared_env.assign_var(identifier.symbol, value)
        },
        NodeType::MemberExpr => {
            let member_expr = node.assignee.as_any().downcast_ref::<MemberExpr>().expect("Failed to downcast to MemberExpr.").clone();
            let object_identifier = member_expr.object.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.").clone();
            let property;

            if member_expr.property.get_kind() == NodeType::Identifier {
                property = member_expr.property.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.").clone().symbol;
            } else if member_expr.property.get_kind() == NodeType::String {
                property = eval(member_expr.property.to_stmt_from_expr(), Arc::clone(&env)).as_any().downcast_ref::<StringValue>().expect("Failed to downcast to StrinvValue.").clone().value;
            } else {
                fatal_error("Unexpected value in member assignment expr");
            }

            let value = eval(node.value.to_stmt_from_expr(), Arc::clone(&env));
            let mut obj = shared_env.lookup_var(object_identifier.symbol.clone()).as_any().downcast_ref::<ObjectValue>().expect("Failed to downcast to ObjectValue.").clone();

            obj.properties.insert(property, value);
            shared_env.assign_var(object_identifier.symbol, Box::new(obj))
        },
        _ => {
            fatal_error(&format!("Invalid LHS inside assignment expression: {:?}", node.assignee));
        }
    }
}

pub fn eval_object_expr(obj: ObjectLiteral, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let mut object = ObjectValue { properties: HashMap::<String, Box<dyn RuntimeValue>>::new() };

    for i in obj.properties {
        if let Some(value) = i.value {
            object.properties.insert(i.key.unwrap(), eval(value.to_stmt_from_expr(), Arc::clone(&env)));
        } else {
            object.properties.insert(i.key.clone().unwrap(), SharedEnvironment(Arc::clone(&env)).lookup_var(i.key.unwrap()));
        }
    }

    return Box::new(object);
}

pub fn eval_list_expr(list: ListLiteral, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let mut elements = vec![];

    for i in list.elements {
        elements.push(eval(i.to_stmt_from_expr(), Arc::clone(&env)));
    }

    Box::new(ListValue {
        elements
    })
}

pub fn eval_member_expr(node: MemberExpr, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let obj = eval(node.object.to_stmt_from_expr(), Arc::clone(&env));
    if obj.get_type() == ValueType::Object {
        let obj = obj.as_any().downcast_ref::<ObjectValue>().unwrap().clone();
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
    } else if obj.get_type() == ValueType::List {
        if !node.computed {
            fatal_error("List cannot be indexed like this");
        }

        let value = eval(node.property.to_stmt_from_expr(), Arc::clone(&env));

        if value.get_type() != ValueType::Number {
            fatal_error("List can only be indexed by numbers");
        }

        let index = value.as_any().downcast_ref::<NumberValue>().expect("Failed to downcast to number").value as i32;

        let obj = obj.as_any().downcast_ref::<ListValue>().unwrap().clone();

        if (obj.elements.len() as i32) < index + 1 || index <= -2 {
            fatal_error("Index out of range");
        }
        
        if index == -1 {
            return obj.elements.get(obj.elements.len() - 1).unwrap().clone();
        }

        obj.elements.get(index as usize).unwrap().clone()
    } else {
        fatal_error("Invalid member expression");
    }
}

pub fn eval_call(expr: CallExpr, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    let mut evaluated_args = vec![];

    for arg in expr.args {
        evaluated_args.push(eval(arg.to_stmt_from_expr(), Arc::clone(&env)));
    }

    let func = eval(expr.caller.to_stmt_from_expr(), Arc::clone(&env));

    if func.get_type() == ValueType::NativeFn {
        let func = func.as_any().downcast_ref::<NativeFnValue>().expect("Failed to downcast to NativeFnValue.").clone();
        return (func.call.func)(evaluated_args, &env);
    } else if func.get_type() == ValueType::Function {
        let func = func.as_any().downcast_ref::<FunctionValue>().expect("Failed to downcast to FunctionValue.").clone();
        return func.call(env, evaluated_args);
    }

    fatal_error(&format!("Cannot call {:?}", func.get_type()));
}