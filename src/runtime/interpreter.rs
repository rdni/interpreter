use std::sync::{Arc, Mutex};

use crate::fatal_error;
use crate::runtime::values::{NumberValue, RuntimeValue};
use crate::frontend::ast::{AssignmentExpr, BinaryExpr, CallExpr, FunctionDeclaration, Identifier, MemberExpr, NodeType, ObjectLiteral, Program, Stmt, StmtValue, StmtWrapper, VarDeclaration};

use super::environment::Environment;
use super::values::StringValue;

use crate::eval::eval_statements::*;
use crate::eval::eval_expressions::*;

pub fn eval(ast_node: StmtWrapper, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    match ast_node.get_kind() {
        // Handle expressions
        NodeType::NumericLiteral => Box::new(NumberValue {
            value: if let StmtValue::F64(val) = ast_node.get_value().unwrap() { val } else { 0.0 as f64}
        }),
        NodeType::String => Box::new(StringValue {
            value: if let StmtValue::StringVal(val) = ast_node.get_value().unwrap() { val } else { String::new() }
        }),
        NodeType::BinaryExpr => {
            let bin_expr = ast_node.as_any().downcast_ref::<BinaryExpr>().expect("Failed to downcast to BinaryExpr.");
            eval_binop_expr(bin_expr.clone(), env)
        },
        NodeType::Identifier => {
            let identifier = ast_node.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.");
            eval_identifier(identifier.clone(), env)
        },
        NodeType::Object => {
            let object = ast_node.as_any().downcast_ref::<ObjectLiteral>().expect("Failed to downcast to ObjectLiteral.");
            eval_object_expr(object.clone(), env)
        },
        NodeType::MemberExpr => {
            let member_expr = ast_node.as_any().downcast_ref::<MemberExpr>().expect("Failed to downcast to MemberExpr.");
            eval_member_expr(member_expr.clone(), env)
        },
        NodeType::AssignmentExpr => {
            let assignment_expr = ast_node.as_any().downcast_ref::<AssignmentExpr>().expect("Failed to downcast to AssignmentExpr.");
            eval_assignment(assignment_expr.clone(), env)
        },
        NodeType::CallExpr => {
            let call_expr = ast_node.as_any().downcast_ref::<CallExpr>().expect("Failed to downcast to CallExpr.");
            eval_call(call_expr.clone(), env)
        },
        // Handle statements
        NodeType::VarDeclaration => {
            let var_declaration = ast_node.as_any().downcast_ref::<VarDeclaration>().expect("Failed to downcast to VarDeclaration.");
            eval_var_declaration(var_declaration.clone(), env)
        },
        NodeType::FunctionDeclaration => {
            let function_declaration = ast_node.as_any().downcast_ref::<FunctionDeclaration>().expect("Failed to downcast to FunctionDeclaration.");
            eval_function_declaration(function_declaration.clone(), env)
        }
        NodeType::Program => {
            let program = ast_node.as_any().downcast_ref::<Program>().expect("Failed to downcast to Program.");
            eval_program(program.clone(), env)
        },
        _ =>  {
            fatal_error(&format!("This statement has not yet been set up for interpretation:\n{:?}", ast_node));
        }
    }
}