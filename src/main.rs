use jlox::{repl, run};
use std::path::PathBuf;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();
    // The first element in the arguments list is the name of the binary.
    // Then the actual binary arguments, flags and options.
    if args.len() == 1 {
        repl()?;
    } else if args.len() == 2 {
        let filepath = PathBuf::from(&args[1]);
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
