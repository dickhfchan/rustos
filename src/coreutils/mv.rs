//! mv command - Move/rename files

use crate::fs;
use crate::println;

pub fn mv_main(args: &[&str]) -> Result<(), &'static str> {
    if args.len() < 2 {
        println!("Usage: mv <source> <destination>");
        return Err("Invalid arguments");
    }

    let source = args[0];
    let dest = args[1];
    
    match move_file(source, dest) {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("mv: cannot move '{}' to '{}': {}", source, dest, e);
            Err(e)
        }
    }
}

fn move_file(source: &str, dest: &str) -> Result<(), &'static str> {
    match fs::move_file(source, dest) {
        Ok(()) => {
            println!("'{}' -> '{}'", source, dest);
            Ok(())
        }
        Err(e) => Err(e)
    }
}