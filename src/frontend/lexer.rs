use std::{collections::HashMap, fmt::Debug};

use crate::{fatal_error, is_skippable, is_valid_ident_char, is_valid_ident_char_start};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Identifier,
    Number,
    String,

    Semicolon,

    Var,
    Const,

    Function,
    Return,

    If,
    Else,

    While,
    For,
    In,

    Comma,
    Colon,
    Dot,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    BinaryOperator,
    Equals,
    RightAngleBracket,
    LeftAngleBracket,
    Bang,

    EOF, // End of file
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: Option<String>,
    token_type: TokenType
}

impl Token {
    pub fn get_value(&self) -> Option<&String> {
        self.value.as_ref()
    }

    pub fn get_token_type(&self) -> TokenType {
        self.token_type
    }
}

pub struct Tokenizer;

impl Tokenizer {
    fn get_keywords(&self) -> HashMap<&str, TokenType> {
        let mut keywords = HashMap::new();

        keywords.insert("var", TokenType::Var);
        keywords.insert("const", TokenType::Const);
        keywords.insert("function", TokenType::Function);
        keywords.insert("return", TokenType::Return);
        keywords.insert("if", TokenType::If);
        keywords.insert("else", TokenType::Else);
        keywords.insert("while", TokenType::While);
        keywords.insert("for", TokenType::For);
        keywords.insert("in", TokenType::In);

        keywords
    }

    pub fn tokenize(&self, source: String) -> Vec<Token> {
        let mut token_output: Vec<Token> = Vec::new();
        let mut src = source.chars().collect::<Vec<char>>();

        while src.len() > 0 {
            if src[0] == '/' {
                if src.len() > 1 {
                    if src[1] == '/' {
                        while src[0] != '\n' || src[0] != '\r' {
                            src.remove(0);
                        }
                    }
                }
            }
            if src[0] == '(' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::OpenParen });
            } else if src[0] == ')' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::CloseParen });
            } else if src[0] == '{' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::OpenBrace });
            } else if src[0] == '}' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::CloseBrace });
            } else if src[0] == '[' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::OpenBracket });
            } else if src[0] == ']' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::CloseBracket });
            } else if src[0] == ',' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::Comma });
            } else if src[0] == '.' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::Dot });
            } else if src[0] == ':' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::Colon });
            } else if src[0] == '+' || src[0] == '-' ||
                      src[0] == '*' || src[0] == '/' ||
                      src[0] == '%' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::BinaryOperator });
            } else if src[0] == '=' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::Equals });
            } else if src[0] == '<' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::LeftAngleBracket });
            } else if src[0] == '>' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::RightAngleBracket });
            } else if src[0] == '!' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::Bang });
            } else if src[0] == ';' {
                token_output.push(Token { value: Some(src.remove(0).to_string()), token_type: TokenType::Semicolon });
            } else if src[0] == '"' {
                src.remove(0);

                let mut escaped = false;
                let mut string = String::new();
                while src.len() > 0 && (src[0] != '"' || escaped) {
                    if src[0] == '\\' && !escaped {
                        escaped = true;
                        src.remove(0);
                    } else if escaped {
                        match src[0] {
                            '\\' => string.push(src.remove(0)),
                            '\"' => string.push(src.remove(0)),
                            '\'' => string.push(src.remove(0)),
                            'n' => {
                                src.remove(0);
                                string.push('\n');
                            },
                            't' => {
                                src.remove(0);
                                string.push('\t')
                            },
                            _ => fatal_error("Unexpected escaped token.")
                        };
                        escaped = false;
                    } else {
                        string.push(src.remove(0));
                    }
                }

                src.remove(0);

                token_output.push(Token {
                    value: Some(string),
                    token_type: TokenType::String
                });
            } else { 
                // Build number
                if src[0].is_numeric() {
                    let mut num = String::new();
                    
                    while src.len() > 0 && (src[0].is_numeric() || src[0] == '.') {
                        num += &src.remove(0).to_string();
                    }

                    token_output.push(Token { value: Some(num), token_type: TokenType::Number });
                } else if is_valid_ident_char_start(src[0]) {
                    let mut identifier = String::new();
                    
                    while src.len() > 0 && (is_valid_ident_char(src[0])) {
                        identifier += &src.remove(0).to_string();
                    }

                    // Check for reserved keyword
                    if let Some(token_type) = self.get_keywords().get(&*identifier) {
                        token_output.push(Token { value: Some(identifier), token_type: *token_type });
                    } else {
                        token_output.push(Token { value: Some(identifier), token_type: TokenType::Identifier });
                    }
                } else if is_skippable(src[0]) {
                    src.remove(0);
                } else {
                    fatal_error(&format!("Unknown character found ('{}').", src[0]));
                }

            }
        }

        token_output.push(Token { value: Some(String::from("EndOfFile")), token_type: TokenType::EOF });
        token_output
    }
}