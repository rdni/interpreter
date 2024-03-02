use crate::frontend::ast::{
    AssignmentExpr, BinaryExpr, CallExpr, Expr, ExprWrapper, Identifier, NodeType, NumericLiteral, ObjectLiteral, Program, Property, Stmt, StmtWrapper, VarDeclaration
};
use crate::frontend::lexer::{Tokenizer, Token, TokenType};
use crate::*;

use super::ast::MemberExpr;

pub struct Parser {
    pub tokens: Vec<Token>
}

impl Parser {
    pub fn produce_ast(&mut self, source_code: String) -> Program {
        self.tokens = Tokenizer {}.tokenize(source_code);

        let mut program = Program {
            kind: NodeType::Program,
            body: Vec::new()
        };

        while self.not_eof() {
            program.body.push(self.parse_stmt());
        }

        program
    }

    fn at(&self) -> Token {
        self.tokens[0].clone()
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
            self.at()
        } else {
            self.eat()
        }
    }

    fn not_eof(&self) -> bool {
        self.at().get_token_type() != TokenType::EOF
    }

    fn parse_stmt(&mut self) -> StmtWrapper {
        match self.at().get_token_type() {
            TokenType::Var => self.parse_var_declaration(),
            TokenType::Const => self.parse_var_declaration(),
            _ => self.parse_expr().to_stmt_from_expr()
        }
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
        let left = self.parse_object_expr();
        
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
            TokenType::OpenParen => {
                self.eat();
                let value = self.parse_expr();
                self.eat_expect(TokenType::CloseParen, "Unexpected token found inside parenthesis.", LoggingLevel::Fatal);
                value
            },
            TokenType::Semicolon => {
                self.eat();
                ExprWrapper::new(Box::new(Identifier { kind: NodeType::Identifier, symbol: String::from("null") }))
            }
            _ => fatal_error(&format!("Unexpected token found during parsing: {:?}", self.at()))
        }
    }

    fn parse_additive_expr(&mut self) -> ExprWrapper {
        let mut left = self.parse_multiplicative_expr();

        while self.at().value.unwrap() == "+" || self.at().value.unwrap() == "-" {
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

        while self.at().value.unwrap() == "*" || self.at().value.unwrap() == "/" || self.at().value.unwrap() == "%" {
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
            // else if self.at().get_token_type() == TokenType::OpenBracket {
            //     self.eat();
            //     property = self.parse_primary_expr();
            //     computed = true;
            // }
            else {
                return object;
            }

            let mut member_expr = ExprWrapper::new(Box::new(MemberExpr {
                kind: NodeType::MemberExpr,
                object,
                property,
                computed
            }));

            while self.at().get_token_type() == TokenType::Dot {
                self.eat();
                member_expr = ExprWrapper::new(Box::new(MemberExpr {
                    kind: NodeType::MemberExpr,
                    object: member_expr,
                    property: self.parse_primary_expr(),
                    computed: false
                }));
            }

            return member_expr;
        }

        self.parse_primary_expr()
    }
    
}