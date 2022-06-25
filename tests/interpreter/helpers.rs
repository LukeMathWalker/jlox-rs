use jlox::{Environment, ExecuteRawError, Interpreter};
use std::cell::RefCell;
use std::rc::Rc;

/// Execute the provided lox source code.
/// It returns the program's output stream.
/// Panics if the interpreter runs into an error, either at runtime or parsing time.
pub fn execute(source: &str) -> String {
    try_execute(source).unwrap()
}

/// Execute the provided lox source code.
/// It returns the program's output stream.
pub fn try_execute(source: &str) -> Result<String, ExecuteRawError> {
    let mut buffer = Vec::new();
    let environment = Rc::new(RefCell::new(Environment::new()));
    let outcome = Interpreter::new(&mut buffer, environment).execute_raw(source);
    outcome.map(|_| String::from_utf8(buffer).unwrap())
}
