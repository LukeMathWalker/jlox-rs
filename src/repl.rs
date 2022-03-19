use crate::interpreter::run;
use std::io::{stdout, Write};

/// Read-print-evaluation loop.
/// It prompts the user to enter lox code and then interprets it on the fly.
pub fn repl() -> Result<(), std::io::Error> {
    loop {
        print!("> ");
        stdout().flush()?;
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break Ok(());
        }
        let input = input.trim().to_string();
        run(input);
    }
}
