use crate::scanner::Scanner;

/// Interpret the jlox source code passed as input.
///
/// It returns `Err` if an error was encountered while interpreting the code.
/// The error type does not contain any information since `run` already takes care, internally,
/// to report the errors it has encountered (i.e. print error messages to stdout).
pub fn run(source: String) -> Result<(), ()> {
    let (tokens, has_errored) = Scanner::new(&source).scan_tokens();
    for token in tokens {
        println!("{token}");
    }
    if has_errored {
        Err(())
    } else {
        Ok(())
    }
}

pub(crate) fn error(line: u64, message: &str) {
    report(line, "", message)
}

pub(crate) fn report(line: u64, where_: &str, message: &str) {
    println!("[line {line}] Error{where_}: {message}");
}
