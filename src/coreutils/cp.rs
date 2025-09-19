//! cp command - Copy files

use crate::fs;
use crate::println;

pub fn cp_main(args: &[&str]) -> Result<(), &'static str> {
    if args.len() < 2 {
        println!("Usage: cp <source> <destination>");
        return Err("Invalid arguments");
    }

    let source = args[0];
    let dest = args[1];
    
    match copy_file(source, dest) {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("cp: cannot copy '{}' to '{}': {}", source, dest, e);
            Err(e)
        }
    }
}

fn copy_file(source: &str, dest: &str) -> Result<(), &'static str> {
    match fs::copy_file(source, dest) {
        Ok(()) => {
            println!("'{}' -> '{}'", source, dest);
            Ok(())
        }
        Err(e) => Err(e)
    }
}