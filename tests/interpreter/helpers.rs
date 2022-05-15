use jlox::Interpreter;

/// Execute the provided lox source code.
/// It returns the program's output stream.
pub fn execute(source: &str) -> String {
    let mut buffer = Vec::new();
    let _ = Interpreter::new(&mut buffer).execute_raw(source);
    String::from_utf8(buffer).unwrap()
}
