use std::{any::Any, fmt::Debug, sync::{Arc, Mutex}};

use crate::runtime::{environment::Environment, interpreter::eval, values::{NullValue, RuntimeValue}};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeType {
    Program,
    Body,

    // Statements
    VarDeclaration,

    FunctionDeclaration,

    Return,

    If,

    While,
    For,

    // Expressions
    Identifier,
    BinaryExpr,
    ComparativeExpr,
    AssignmentExpr,
    MemberExpr,
    CallExpr,
    
    // Literals
    NumericLiteral,
    NullLiteral,
    Property,
    Object,
    List,
    String
}

pub enum StmtValue {
    StringVal(String),
    F64(f64)
}

pub trait Stmt: Debug + Any + 'static {
    fn get_kind(&self) -> NodeType;
    fn get_value(&self) -> Option<StmtValue>;
    fn clone_boxed(&self) -> Box<dyn Stmt>;
    fn clone_as_wrapper(&self) -> StmtWrapper;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn Stmt> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

#[derive(Debug, Clone)]
pub struct StmtWrapper {
    inner: Box<dyn Stmt>
}

impl StmtWrapper {
    pub fn new(stmt: Box<dyn Stmt>) -> Self {
        StmtWrapper {
            inner: stmt
        }
    }
}

impl Stmt for StmtWrapper {
    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        self.inner.clone_boxed()
    }
    fn get_kind(&self) -> NodeType {
        self.inner.get_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        self.inner.get_value()
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub kind: NodeType,
    pub body: Body
}

impl Stmt for Program {
    fn get_kind(&self) -> NodeType {
        NodeType::Program
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub kind: NodeType,
    body: Vec<StmtWrapper>
}

impl Stmt for Body {
    fn get_kind(&self) -> NodeType {
        self.kind
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for Body {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

impl Body {
    pub fn new(body: Vec<StmtWrapper>) -> Self {
        Body {
            body,
            kind: NodeType::Body
        }
    }

    pub fn run(&self, env: Arc<Mutex<Environment>>, make_env: bool) -> (Box<dyn RuntimeValue>, Arc<Mutex<Environment>>) {
        if make_env {
            let new_env = Arc::new(Mutex::new(Environment::new(Some(Arc::clone(&env)))));

            let mut last_value: Box<dyn RuntimeValue> = Box::new(NullValue {});
            for stmt in self.body.iter() {
                last_value = eval(stmt.clone(), Arc::clone(&new_env));
            }

            (last_value, new_env)
        } else {
            let mut last_value: Box<dyn RuntimeValue> = Box::new(NullValue {});
            for stmt in self.body.iter() {
                last_value = eval(stmt.clone(), Arc::clone(&env));
            }

            (last_value, env)
        }
    }
}

// var x; means x is undefined
#[derive(Debug, Clone)]
pub struct VarDeclaration {
    pub kind: NodeType,
    pub constant: bool,
    pub identifier: String,
    pub value: Option<ExprWrapper>
}

impl Stmt for VarDeclaration {
    fn get_kind(&self) -> NodeType {
        NodeType::VarDeclaration
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub kind: NodeType,
    pub parameters: Vec<String>,
    pub name: String,
    pub body: Body
}

impl Stmt for FunctionDeclaration {
    fn get_kind(&self) -> NodeType {
        NodeType::FunctionDeclaration
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

pub trait Expr: Stmt {
    fn get_expr_kind(&self) -> NodeType;
    fn get_expr_value(&self) -> Option<StmtValue>;
    fn clone_box(&self) -> Box<dyn Expr>;
    fn to_stmt_from_expr(&self) -> StmtWrapper;
}

impl Clone for Box<dyn Expr> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct ExprWrapper {
    inner: Box<dyn Expr>
}

impl ExprWrapper {
    pub fn new(expr: Box<dyn Expr>) -> Self {
        ExprWrapper {
            inner: expr
        }
    }
}

impl Stmt for ExprWrapper {
    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        self.inner.clone_boxed()
    }
    fn get_kind(&self) -> NodeType {
        self.inner.get_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        self.inner.get_value()
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for ExprWrapper {
    fn clone_box(&self) -> Box<dyn Expr> {
        self.inner.clone_box()
    }
    fn get_expr_kind(&self) -> NodeType {
        self.inner.get_expr_kind()
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        self.inner.get_expr_value()
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        self.inner.to_stmt_from_expr()
    }
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub kind: NodeType,
    pub assignee: ExprWrapper,
    pub value: ExprWrapper
}

impl Stmt for AssignmentExpr {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for AssignmentExpr {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

// 10 - 5 is binary expression
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub kind: NodeType,
    pub left: ExprWrapper,
    pub right: ExprWrapper,
    pub operator: String
}

impl Stmt for BinaryExpr {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for BinaryExpr {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct ComparativeExpr {
    pub kind: NodeType,
    pub left: ExprWrapper,
    pub right: ExprWrapper,
    pub operator: String
}

impl Stmt for ComparativeExpr {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for ComparativeExpr {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub kind: NodeType,
    pub symbol: String
}

impl Stmt for Identifier {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for Identifier {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct NumericLiteral {
    pub kind: NodeType,
    pub value: f64
}

impl Stmt for NumericLiteral {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for NumericLiteral {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        Some(StmtValue::F64(self.value))
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct Property {
    pub kind: NodeType,
    pub key: Option<String>,
    pub value: Option<ExprWrapper>
}

impl Stmt for Property {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for Property {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct ObjectLiteral {
    pub kind: NodeType,
    pub properties: Vec<Property>
}

impl Stmt for ObjectLiteral {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for ObjectLiteral {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct ListLiteral {
    pub kind: NodeType,
    pub elements: Vec<ExprWrapper>
}

impl Stmt for ListLiteral {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for ListLiteral {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub kind: NodeType,
    pub args: Vec<ExprWrapper>,
    pub caller: ExprWrapper
}

impl Stmt for CallExpr {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for CallExpr {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub kind: NodeType,
    pub object: ExprWrapper,
    pub property: ExprWrapper,
    pub computed: bool
}

impl Stmt for MemberExpr {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for MemberExpr {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub kind: NodeType,
    pub string: String,
}

impl Stmt for StringLiteral {
    fn get_kind(&self) -> NodeType {
        self.get_expr_kind()
    }
    fn get_value(&self) -> Option<StmtValue> {
        Some(self.get_expr_value().unwrap())
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

impl Expr for StringLiteral {
    fn get_expr_kind(&self) -> NodeType {
        self.kind
    }
    fn get_expr_value(&self) -> Option<StmtValue> {
        Some(StmtValue::StringVal(self.string.clone()))
    }
    fn clone_box(&self) -> Box<dyn Expr> {
        Box::new(self.clone())
    }
    fn to_stmt_from_expr(&self) -> StmtWrapper {
        StmtWrapper::new(Box::new(self.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub kind: NodeType,
    pub value: ExprWrapper
}

impl Stmt for ReturnStmt {
    fn get_kind(&self) -> NodeType {
        self.kind
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub kind: NodeType,
    pub condition: ExprWrapper,
    pub body: Body,
    pub else_stmt: Option<Body>
}

impl Stmt for IfStmt {
    fn get_kind(&self) -> NodeType {
        self.kind
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub kind: NodeType,
    pub condition: ExprWrapper,
    pub body: Body
}

impl Stmt for WhileStmt {
    fn get_kind(&self) -> NodeType {
        self.kind
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub kind: NodeType,
    pub iterable: ExprWrapper,
    pub variable: ExprWrapper,
    pub body: Body
}

impl Stmt for ForStmt {
    fn get_kind(&self) -> NodeType {
        self.kind
    }
    fn get_value(&self) -> Option<StmtValue> {
        None
    }
    fn clone_boxed(&self) -> Box<dyn Stmt> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_as_wrapper(&self) -> StmtWrapper {
        StmtWrapper::new(self.clone_boxed())
    }
}