#![allow(unused_imports)]

use interpreter::frontend::parser::Parser;
use interpreter::runtime::environment::setup_scope;
use interpreter::runtime::environment::Environment;
use interpreter::runtime::interpreter::eval;
use interpreter::runtime::values::BooleanValue;
use interpreter::runtime::values::NullValue;
use interpreter::runtime::values::NumberValue;
use interpreter::frontend::ast::StmtWrapper;
use std::fs;
use std::io;
use std::io::Write;

fn main() {
    let mut program = Parser { tokens: vec![] };

    // println!("{:?}", tokenizer.tokenize(fs::read_to_string("src/testingfile.tl").unwrap()));

    let mut env = Environment::new(None);
    let mut exit = false;
    while exit == false {
        let mut input = String::new();

        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut input)
            .unwrap();

        if input.trim() == "exit" {
            exit = true;
        } else if input.trim() == "file" {
            // let mut input = String::new();

            // print!("File name > ");
            // io::stdout().flush().unwrap();
            // io::stdin()
            //     .read_line(&mut input)
            //     .unwrap();

            let ast = program.produce_ast(fs::read_to_string("src/testingfile.txt").unwrap());
            // println!("AST: {:?}", ast);
            eval(StmtWrapper::new(Box::new(ast)), &mut env).to_string();
        } else {
            let ast = program.produce_ast(input);
            // println!("AST: {:?}", ast);
            println!("{}", &eval(StmtWrapper::new(Box::new(ast)), &mut env).to_string());
        }
    }

}