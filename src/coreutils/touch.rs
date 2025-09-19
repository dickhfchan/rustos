//! touch command - Create files

use crate::fs;
use crate::println;

pub fn touch_main(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!("Usage: touch <file1> [file2] ...");
        return Err("No files specified");
    }

    for &filename in args {
        match create_file(filename) {
            Ok(()) => {}
            Err(e) => {
                println!("touch: cannot touch '{}': {}", filename, e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

fn create_file(path: &str) -> Result<(), &'static str> {
    match fs::create_file(path) {
        Ok(()) => {
            println!("Created file: {}", path);
            Ok(())
        }
        Err(e) => Err(e)
    }
}