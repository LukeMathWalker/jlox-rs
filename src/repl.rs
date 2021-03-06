use crate::{Environment, Interpreter};
use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;

/// Read-print-evaluation loop.
/// It prompts the user to enter lox code and then interprets it on the fly.
pub fn repl() -> Result<(), std::io::Error> {
    let environment = Rc::new(RefCell::new(Environment::new()));
    let mut interpreter = Interpreter::new(stdout(), environment);
    loop {
        print!("> ");
        stdout().flush()?;
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() || input.is_empty() {
            break Ok(());
        }
        let input = input.trim().to_string();
        if let Err(e) = interpreter.execute_raw(&input) {
            eprintln!("{}", e);
        }
    }
}
