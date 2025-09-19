//! mkdir command - Create directories

use crate::fs;
use crate::println;

pub fn mkdir_main(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!("Usage: mkdir <directory1> [directory2] ...");
        return Err("No directories specified");
    }

    for &dirname in args {
        match create_directory(dirname) {
            Ok(()) => {}
            Err(e) => {
                println!("mkdir: cannot create directory '{}': {}", dirname, e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

fn create_directory(path: &str) -> Result<(), &'static str> {
    match fs::create_directory(path) {
        Ok(()) => {
            println!("Created directory: {}", path);
            Ok(())
        }
        Err(e) => Err(e)
    }
}