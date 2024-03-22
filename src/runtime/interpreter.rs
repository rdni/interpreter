use std::sync::{Arc, Mutex};

use crate::{fatal_error, MK_NUMBER, MK_STRING};
use crate::runtime::values::{NumberValue, RuntimeValue};
use crate::frontend::ast::{AssignmentExpr, BinaryExpr, CallExpr, ComparativeExpr, FunctionDeclaration, Identifier, IfStmt, MemberExpr, NodeType, ObjectLiteral, Program, ReturnStmt, Stmt, StmtValue, StmtWrapper, VarDeclaration};

use super::environment::Environment;
use super::values::StringValue;

use crate::eval::eval_statements::*;
use crate::eval::eval_expressions::*;

pub fn eval(ast_node: StmtWrapper, env: Arc<Mutex<Environment>>) -> Box<dyn RuntimeValue> {
    match ast_node.get_kind() {
        // Handle expressions
        NodeType::NumericLiteral => {
            Box::new(MK_NUMBER!(if let StmtValue::F64(val) = ast_node.get_value().unwrap() { val } else { 0.0 as f64}))},
        NodeType::String => {
            Box::new(MK_STRING!(if let StmtValue::StringVal(val) = ast_node.get_value().unwrap() { val } else { String::new() }))},
        NodeType::BinaryExpr => {
            let bin_expr = ast_node.as_any().downcast_ref::<BinaryExpr>().expect("Failed to downcast to BinaryExpr.");
            eval_binop_expr(bin_expr.clone(), env)
        },
        NodeType::ComparativeExpr => {
            let comp_expr = ast_node.as_any().downcast_ref::<ComparativeExpr>().expect("Failed to downcast to ComparativeExpr.");
            eval_comp_expr(comp_expr.clone(), env)
        }
        NodeType::Identifier => {
            let identifier = ast_node.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.");
            let value = eval_identifier(identifier.clone(), Arc::clone(&env));
            value
        },
        NodeType::Object => {
            let object = ast_node.as_any().downcast_ref::<ObjectLiteral>().expect("Failed to downcast to ObjectLiteral.");
            let value = eval_object_expr(object.clone(), Arc::clone(&env));
            value
        },
        NodeType::MemberExpr => {
            let member_expr = ast_node.as_any().downcast_ref::<MemberExpr>().expect("Failed to downcast to MemberExpr.");
            let value = eval_member_expr(member_expr.clone(), Arc::clone(&env));
            value
        },
        NodeType::AssignmentExpr => {
            let assignment_expr = ast_node.as_any().downcast_ref::<AssignmentExpr>().expect("Failed to downcast to AssignmentExpr.");
            let value = eval_assignment(assignment_expr.clone(), Arc::clone(&env));
            value
        },
        NodeType::CallExpr => {
            let call_expr = ast_node.as_any().downcast_ref::<CallExpr>().expect("Failed to downcast to CallExpr.");
            let value = eval_call(call_expr.clone(), Arc::clone(&env));
            value
        },
        // Handle statements
        NodeType::VarDeclaration => {
            let var_declaration = ast_node.as_any().downcast_ref::<VarDeclaration>().expect("Failed to downcast to VarDeclaration.");
            let value = eval_var_declaration(var_declaration.clone(), Arc::clone(&env));
            value
        },
        NodeType::FunctionDeclaration => {
            let function_declaration = ast_node.as_any().downcast_ref::<FunctionDeclaration>().expect("Failed to downcast to FunctionDeclaration.");
            let value = eval_function_declaration(function_declaration.clone(), Arc::clone(&env));
            value
        },
        NodeType::Return => {
            let return_stmt = ast_node.as_any().downcast_ref::<ReturnStmt>().expect("Failed to downcast to ReturnStmt");
            eval_return(return_stmt.clone(), env)
        },
        NodeType::If => {
            let if_stmt = ast_node.as_any().downcast_ref::<IfStmt>().expect("Failed to downcast to IfStmt");
            eval_if(if_stmt.clone(), env)
        },
        NodeType::Program => {
            let program = ast_node.as_any().downcast_ref::<Program>().expect("Failed to downcast to Program.");
            eval_program(program.clone(), env)
        },
        _ =>  {
            fatal_error(&format!("This statement has not yet been set up for interpretation:\n{:?}", ast_node));
        }
    }
}