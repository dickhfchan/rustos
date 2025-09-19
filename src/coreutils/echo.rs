//! echo command - Display text

use alloc::string::String;
use crate::println;

pub fn echo_main(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!();
        return Ok(());
    }

    let mut output = String::new();
    let mut first = true;
    
    for &arg in args {
        if !first {
            output.push(' ');
        }
        output.push_str(arg);
        first = false;
    }
    
    println!("{}", output);
    Ok(())
}