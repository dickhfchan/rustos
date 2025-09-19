//! rm command - Remove files and directories

use crate::fs;
use crate::println;

pub fn rm_main(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!("Usage: rm <file1> [file2] ...");
        return Err("No files specified");
    }

    for &filename in args {
        match remove_file(filename) {
            Ok(()) => {}
            Err(e) => {
                println!("rm: cannot remove '{}': {}", filename, e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

fn remove_file(path: &str) -> Result<(), &'static str> {
    match fs::remove_file(path) {
        Ok(()) => {
            println!("Removed: {}", path);
            Ok(())
        }
        Err(e) => Err(e)
    }
}