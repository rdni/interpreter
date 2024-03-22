// var x = 45;
// [ VarToken, IdentifierToken, EqualsToken, NumberToken, SemicolonToken ]

#![allow(dead_code)]

pub mod frontend;
pub mod runtime;
pub mod eval;
pub mod macros;

pub enum LoggingLevel {
    Info,
    Warn,
    Error,
    Fatal
}

fn warn(information: &str) {
    println!("[-] {}", information);
}

fn info(information: &str) {
    println!("[+] {}", information);
}

fn error(information: &str) {
    println!("[-] ERROR: {}", information);
}

fn fatal_error(information: &str) -> ! {
    println!("[-] FATAL ERROR: {}", information);
    panic!("Fatal error.");
}

fn is_skippable(src: char) -> bool {
    src == ' ' || src == '\n' || src == '\t' || src == '\r'
}

fn pad_each_line(amount: usize, string: String) -> String {
    string
        .lines()
        .map(|line| format!("{}{}", " ".repeat(amount), line))
        .collect::<Vec<_>>()
        .join("\n")
}