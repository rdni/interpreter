#![allow(unused_imports)]

use interpreter::frontend::parser::Parser;
use interpreter::runtime::environment::setup_scope;
use interpreter::runtime::environment::Environment;
use interpreter::runtime::interpreter::eval;
use interpreter::runtime::values::BooleanValue;
use interpreter::runtime::values::NullValue;
use interpreter::runtime::values::NumberValue;
use interpreter::frontend::ast::StmtWrapper;
use interpreter::MK_BOOL;
use std::fs;
use std::io;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    let mut program = Parser { tokens: vec![] };

    // println!("{:?}", tokenizer.tokenize(fs::read_to_string("src/testingfile.tl").unwrap()));

    let env = Arc::new(Mutex::new(Environment::new(None)));
    loop {
        let mut input = String::new();

        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut input)
            .unwrap();

        if input.trim() == "file" {
            // let mut input = String::new();

            // print!("File name > ");
            // io::stdout().flush().unwrap();
            // io::stdin()
            //     .read_line(&mut input)
            //     .unwrap();

            let mut debug = false;

            if env.lock().unwrap().variables.get("debug").unwrap().equals(Box::new(MK_BOOL!(true))) {
                debug = true;
            }

            *env.lock().unwrap() = Environment::new(None);

            let ast = program.produce_ast(fs::read_to_string("src/testingfile.txt").unwrap());

            if debug {
                println!("AST: {:?}", ast);
            }
            eval(StmtWrapper::new(Box::new(ast)), Arc::clone(&env)).to_string();
        } else {
            let ast = program.produce_ast(input);

            let mut debug = false;

            if env.lock().unwrap().variables.get("debug").unwrap().equals(Box::new(MK_BOOL!(true))) {
                debug = true;
            }

            if debug {
                println!("AST: {:?}", ast);
            }
            let result = &eval(StmtWrapper::new(Box::new(ast)), Arc::clone(&env)).to_string();
            if result != "null" {
                println!("{}", result);
            }
        }
    }

}