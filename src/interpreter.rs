use crate::scanner::Scanner;

/// Interpret the jlox source code passed as input.
///
/// It returns `Err` if an error was encountered while interpreting the code.
/// The error type does not contain any information since `run` already takes care, internally,
/// to report the errors it has encountered (i.e. print error messages to stdout).
pub fn run(source: String) -> Result<(), ()> {
    for token in Scanner::new(&source) {
        println!("{token}");
    }
    Ok(())
}
