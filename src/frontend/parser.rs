use crate::frontend::ast::{
    AssignmentExpr, BinaryExpr, CallExpr, ComparativeExpr, Expr, ExprWrapper, FunctionDeclaration, Identifier, NodeType, NumericLiteral, ObjectLiteral, Program, Property, Stmt, StmtWrapper, VarDeclaration
};
use crate::frontend::lexer::{Tokenizer, Token, TokenType};
use crate::*;


use super::ast::{Body, IfStmt, MemberExpr, ReturnStmt, StringLiteral};

pub struct Parser {
    pub tokens: Vec<Token>
}

impl Parser {
    pub fn produce_ast(&mut self, source_code: String) -> Program {
        self.tokens = Tokenizer {}.tokenize(source_code);

        let mut body = Vec::new();

        while self.not_eof() {
            let stmt = self.parse_stmt();
            if let Some(v) = stmt {
                body.push(v);
            }
        }

        let body = Body {
            kind: NodeType::Body,
            body
        };

        Program {
            kind: NodeType::Program,
            body,
        }
    }

    fn at_comparative_expr(&self) -> Option<usize> {

        let token1 = self.at().get_token_type();
        
        if token1 == TokenType::EOF {
            return None;
        }

        let token2 = self.look_ahead(1).get_token_type();

        // ==
        if token1 == TokenType::Equals && token2 == TokenType::Equals {
            return Some(2);
        }
        // >=
        if token1 == TokenType::RightAngleBracket && token2 == TokenType::Equals {
            return Some(2);
        }
        // <=
        if token1 == TokenType::LeftAngleBracket && token2 == TokenType::Equals {
            return Some(2);
        }
        // !=
        if token1 == TokenType::Bang && token2 == TokenType::Equals {
            return Some(2);
        }
        // <
        if token1 == TokenType::LeftAngleBracket {
            return Some(1);
        }
        // >
        if token1 == TokenType::RightAngleBracket {
            return Some(1);
        }

        None
    }

    fn at(&self) -> &Token {
        &self.tokens[0]
    }

    fn look_ahead(&self, amount: usize) -> &Token {
        &self.tokens[amount]
    }

    fn eat(&mut self) -> Token {
        self.tokens.remove(0)
    }

    fn eat_expect(&mut self, token_type: TokenType, error_msg: &str, level: LoggingLevel) -> Token {
        if self.at().get_token_type() != token_type {
            match level {
                LoggingLevel::Info => info(&format!("Parser Error:\n{} {:?}.\nExpecting {:?}", error_msg, self.at(), token_type)),
                LoggingLevel::Warn => warn(&format!("Parser Error:\n{} {:?}.\nExpecting {:?}", error_msg, self.at(), token_type)),
                LoggingLevel::Error => error(&format!("Parser Error:\n{} {:?}.\nExpecting {:?}", error_msg, self.at(), token_type)),
                LoggingLevel::Fatal => fatal_error(&format!("Parser Error:\n{} {:?}.\nExpecting {:?}", error_msg, self.at(), token_type))
            };
            self.at().clone()
        } else {
            self.eat()
        }
    }

    fn not_eof(&self) -> bool {
        self.at().get_token_type() != TokenType::EOF
    }

    fn parse_stmt(&mut self) -> Option<StmtWrapper> {
        match self.at().get_token_type() {
            TokenType::Var => Some(self.parse_var_declaration()),
            TokenType::Const => Some(self.parse_var_declaration()),
            TokenType::Function => Some(self.parse_function_declaration()),
            TokenType::Return => Some(self.parse_return()),
            TokenType::If => Some(self.parse_if()),
            TokenType::Semicolon => {
                self.eat();
                if self.not_eof() && self.at().get_token_type() != TokenType::CloseBrace {
                    self.parse_stmt()
                } else {
                    None
                }
            },
            TokenType::OpenBrace => Some(StmtWrapper::new(Box::new(self.parse_body()))),
            _ => Some(self.parse_expr().to_stmt_from_expr())
        }
    }

    fn parse_body(&mut self) -> Body {
        self.eat_expect(TokenType::OpenBrace, "Expected statement body", LoggingLevel::Fatal);

        let mut body = vec![];
        while self.at().get_token_type() != TokenType::CloseBrace && self.not_eof() {
            if let Some(v) = self.parse_stmt() {
                body.push(v);
            } else {
                break
            }
        }

        self.eat_expect(TokenType::CloseBrace, "Expected closing brace in body", LoggingLevel::Fatal);

        Body {
            kind: NodeType::Body,
            body
        }
    }

    fn parse_if(&mut self) -> StmtWrapper {
        self.eat();

        let condition = self.parse_comparative_expr();
        
        let body = self.parse_body();

        let mut else_stmt = None;
        // Check for else / else if
        if self.at().get_token_type() == TokenType::Else {
            self.eat();
            if self.at().get_token_type() == TokenType::OpenBrace {
                else_stmt = Some(self.parse_body())
            } else if self.at().get_token_type() == TokenType::If {
                let if_stmt = self.parse_if();
                else_stmt = Some(Body {
                    kind: NodeType::Body,
                    body: vec![if_stmt]
                });
            }
        }

        StmtWrapper::new(Box::new(IfStmt {
            kind: NodeType::If,
            condition,
            body,
            else_stmt
        }))
    }

    fn parse_return(&mut self) -> StmtWrapper {
        self.eat();

        let value = self.parse_expr();

        self.eat_expect(TokenType::Semicolon, "Expected semicolon after return statement", LoggingLevel::Fatal);

        return StmtWrapper::new(Box::new(ReturnStmt {
            kind: NodeType::Return,
            value
        }));
    }

    fn parse_function_declaration(&mut self) -> StmtWrapper {
        self.eat();

        let name = self.eat_expect(TokenType::Identifier, "Unexpected token after function declaration", LoggingLevel::Fatal).value.unwrap();

        let args = self.parse_args();
        let mut params = Vec::new();

        for arg in args.into_iter() {
            if arg.get_kind() == NodeType::Identifier {
                params.push(arg.as_any().downcast_ref::<Identifier>().expect("Failed to downcast to Identifier.").clone().symbol);
            } else {
                fatal_error("Expected identifier inside function declaration");
            }
        }
        
        let body = self.parse_body();

        return StmtWrapper::new(Box::new(FunctionDeclaration { 
            kind: NodeType::FunctionDeclaration,
            parameters: params,
            name,
            body,
        }));
    }

    // VAR IDENTIFIER;
    // ( CONST | VAR ) IDENTIFIER = EXPR;
    fn parse_var_declaration(&mut self) -> StmtWrapper {
        let is_constant = self.eat().get_token_type() == TokenType::Const;
        let identifier = self.eat_expect(TokenType::Identifier, "Error in var declaration.", LoggingLevel::Fatal).value.unwrap();

        if self.at().get_token_type() == TokenType::Semicolon {
            self.eat();
            if is_constant {
                fatal_error("Must assign value to const expression. No value provided.");
            }

            return StmtWrapper::new(Box::new(VarDeclaration {
                kind: NodeType::VarDeclaration,
                constant: is_constant,
                identifier,
                value: Some(ExprWrapper::new(Box::new(Identifier { kind: NodeType::Identifier, symbol: String::from("null") })))
            }));
        }

        self.eat_expect(TokenType::Equals, "Expected equals token in var declaration.", LoggingLevel::Fatal);

        let declaration = VarDeclaration { 
            kind: NodeType::VarDeclaration,
            constant: is_constant,
            identifier,
            value: Some(self.parse_expr())
        };

        self.eat_expect(TokenType::Semicolon, "Expected semicolon after variable declaration (automatically inserted).", LoggingLevel::Error);

        StmtWrapper::new(Box::new(declaration))
    }

    fn parse_expr(&mut self) -> ExprWrapper {
        self.parse_assignment_expr()
    }

    fn parse_assignment_expr(&mut self) -> ExprWrapper {
        let left = self.parse_comparative_expr();
        
        if self.at().get_token_type() == TokenType::Equals {
            self.eat();
            let value = self.parse_assignment_expr();

            if self.at().get_token_type() == TokenType::Semicolon {
                self.eat();
            }
            return ExprWrapper::new(Box::new(AssignmentExpr {
                kind: NodeType::AssignmentExpr,
                assignee: left,
                value: value
            }));
        }

        left
    }

    fn parse_object_expr(&mut self) -> ExprWrapper {
        if self.at().get_token_type() != TokenType::OpenBrace {
            return self.parse_additive_expr();
        }

        self.eat();

        let mut properties = Vec::new();

        while self.not_eof() && self.at().get_token_type() != TokenType::CloseBrace {
            let key = self.eat_expect(TokenType::Identifier, "Unexpected token in object literal creation.", LoggingLevel::Fatal).value;

            if self.at().get_token_type() == TokenType::Comma {
                self.eat();
                properties.push(Property { kind: NodeType::Property, key, value: None });
                continue;
            }
            if self.at().get_token_type() == TokenType::CloseBrace {
                properties.push(Property { kind: NodeType::Property, key, value: None });
                continue;
            }
            
            self.eat_expect(TokenType::Colon, "Missing colon following identifier in object literal creation.", LoggingLevel::Fatal);
            let value = self.parse_expr();

            properties.push(Property { kind: NodeType::Property, key, value: Some(value) });

            if self.at().get_token_type() != TokenType::CloseBrace {
                self.eat_expect(TokenType::Comma, "Object literal missing comma.", LoggingLevel::Fatal);
            }
        }

        self.eat_expect(TokenType::CloseBrace, "Object literal missing closing brace.", LoggingLevel::Error);

        ExprWrapper::new(Box::new(ObjectLiteral {
            kind: NodeType::Object,
            properties
        }))
    }

    // Prescidence Order
    // AssignmentExpr
    // MemberExpr
    // FunctionCall
    // LogicalExpr
    // ComparativeExpr
    // AdditiveExpr
    // MultiplicativeExpr
    // UnaryExpr
    // PrimaryExpr

    fn parse_primary_expr(&mut self) -> ExprWrapper {
        let token = self.at();

        match token.get_token_type() {
            TokenType::Identifier => ExprWrapper::new(Box::new(Identifier { kind: NodeType::Identifier, symbol: self.eat().value.unwrap() })),
            TokenType::Number => ExprWrapper::new(Box::new(NumericLiteral { kind: NodeType::NumericLiteral, value: self.eat().value.unwrap().parse().expect("Problem converting numeric literal") })),
            TokenType::String => ExprWrapper::new(Box::new(StringLiteral { kind: NodeType::String, string: self.eat().value.unwrap()})),
            TokenType::OpenParen => {
                self.eat();
                let value = self.parse_expr();
                self.eat_expect(TokenType::CloseParen, "Unexpected token found inside parenthesis.", LoggingLevel::Fatal);
                value
            },
            _ => fatal_error(&format!("Unexpected token found during parsing: {:?}", self.at()))
        }
    }

    fn parse_comparative_expr(&mut self) -> ExprWrapper {
        let mut left = self.parse_object_expr();
        
        if !self.not_eof() && !self.at_comparative_expr().is_none() {
            return left;
        }

        while self.at_comparative_expr().is_some() && self.not_eof() {
            let mut operator = String::new();

            for _ in 0..(self.at_comparative_expr().unwrap()) {
                operator += &self.eat().value.unwrap()
            }

            let right = self.parse_object_expr();

            left = ExprWrapper::new(Box::new(ComparativeExpr {
                kind: NodeType::ComparativeExpr,
                left,
                right,
                operator
            }));
        }

        left
    }

    fn parse_additive_expr(&mut self) -> ExprWrapper {
        let mut left = self.parse_multiplicative_expr();

        while self.at().value.clone().unwrap() == "+" || self.at().value.clone().unwrap() == "-" {
            let operator = self.eat().value.unwrap();
            let right = self.parse_multiplicative_expr();

            left = ExprWrapper::new(Box::new(BinaryExpr {
                kind: NodeType::BinaryExpr,
                left,
                right,
                operator
            }))
        }

        left
    }

    fn parse_multiplicative_expr(&mut self) -> ExprWrapper {
        let mut left = self.parse_call_member_expr();

        while self.at().value.clone().unwrap() == "*" || self.at().value.clone().unwrap() == "/" || self.at().value.clone().unwrap() == "%" {
            let operator = self.eat().value.unwrap();
            let right = self.parse_call_member_expr();

            left = ExprWrapper::new(Box::new(BinaryExpr {
                kind: NodeType::BinaryExpr,
                left,
                right,
                operator
            }))
        }

        left
    }

    fn parse_call_member_expr(&mut self) -> ExprWrapper {
        let member = self.parse_member_expr();
        
        if self.at().get_token_type() == TokenType::OpenParen {
            return self.parse_call_expr(member);
        }

        member
    }

    fn parse_call_expr(&mut self, caller: ExprWrapper) -> ExprWrapper {
        let mut call_expr = CallExpr {
            kind: NodeType::CallExpr,
            caller,
            args: self.parse_args()
        };

        if self.at().get_token_type() == TokenType::OpenParen {
            call_expr = self.parse_call_expr(ExprWrapper::new(Box::new(call_expr))).as_any().downcast_ref::<CallExpr>().unwrap().clone();
        }

        ExprWrapper::new(Box::new(call_expr))
    }

    fn parse_args(&mut self) -> Vec<ExprWrapper> {
        self.eat_expect(TokenType::OpenParen, "Expected open parenthesis when parsing call arguments", LoggingLevel::Fatal);

        let args = if self.at().get_token_type() == TokenType::CloseParen {
            Vec::new()
        } else {
            self.parse_arguments_list()
        };

        self.eat_expect(TokenType::CloseParen, "Expected closing parenthesis when parsing call arguments", LoggingLevel::Fatal);

       return args;
    }

    fn parse_arguments_list(&mut self) -> Vec<ExprWrapper> {
        let mut args = vec![self.parse_assignment_expr()];

        while self.at().get_token_type() == TokenType::Comma && self.not_eof() {
            self.eat();
            args.push(self.parse_assignment_expr());
        }

        return args;
    }

    fn parse_member_expr(&mut self) -> ExprWrapper {
        if self.at().get_token_type() == TokenType::Identifier {
            let object = self.parse_primary_expr();
            let property;
            let computed;

            if self.at().get_token_type() == TokenType::Dot {
                self.eat();
                property = self.parse_primary_expr();
                computed = false;
            } 
            else if self.at().get_token_type() == TokenType::OpenBracket {
                self.eat();
                property = self.parse_expr();
                computed = true;
                self.eat();
            }
            else {
                return object;
            }

            let mut member_expr = ExprWrapper::new(Box::new(MemberExpr {
                kind: NodeType::MemberExpr,
                object,
                property,
                computed
            }));

            while self.at().get_token_type() == TokenType::Dot || self.at().get_token_type() == TokenType::OpenBracket {
                if self.at().get_token_type() == TokenType::Dot {
                    self.eat();
                    member_expr = ExprWrapper::new(Box::new(MemberExpr {
                        kind: NodeType::MemberExpr,
                        object: member_expr,
                        property: self.parse_primary_expr(),
                        computed: false
                    }));
                } else {
                    self.eat();
                    member_expr = ExprWrapper::new(Box::new(MemberExpr {
                        kind: NodeType::MemberExpr,
                        object: member_expr,
                        property: self.parse_expr(),
                        computed: true
                    }));
                    self.eat();
                }
            }

            return member_expr;
        }

        self.parse_primary_expr()
    }
}