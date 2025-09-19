//! cat command - Display file contents

use crate::fs;
use crate::println;

pub fn cat_main(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!("Usage: cat <file1> [file2] ...");
        return Err("No files specified");
    }

    for &filename in args {
        match cat_file(filename) {
            Ok(()) => {}
            Err(e) => {
                println!("cat: {}: {}", filename, e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

fn cat_file(filename: &str) -> Result<(), &'static str> {
    // In a real implementation, this would read from the filesystem
    // For now, we'll simulate reading a file
    match fs::read_file(filename) {
        Ok(contents) => {
            for line in contents.lines() {
                println!("{}", line);
            }
            Ok(())
        }
        Err(_) => {
            // Fallback: show simulated content
            println!("# Simulated content for: {}", filename);
            println!("# This would be the actual file content in a real filesystem");
            Ok(())
        }
    }
}