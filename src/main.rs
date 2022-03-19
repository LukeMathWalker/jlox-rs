use jlox::{repl, run};
use std::path::PathBuf;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        repl()?;
    } else if args.len() == 1 {
        let filepath = PathBuf::from(&args[0]);
        let file = std::fs::read_to_string(filepath)?;
        run(file);
    } else {
        println!("Usage: jlox [script]");
        // Why 64, you ask?
        //
        // If you run (on a Linux machine):
        // ```
        // grep 64 /usr/include/sysexits.h
        // ```
        //
        // You'll find:
        // ```
        // #define EX__BASE        64      /* base value for error messages */
        // #define EX_USAGE        64      /* command line usage error */
        // ```
        std::process::exit(64);
    }
    Ok(())
}
