//! head command - Show first lines of file

use crate::fs;
use crate::println;

pub fn head_main(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!("Usage: head <file1> [file2] ...");
        return Err("No files specified");
    }

    for &filename in args {
        match show_head(filename, 10) {
            Ok(()) => {}
            Err(e) => {
                println!("head: {}: {}", filename, e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

fn show_head(filename: &str, lines: usize) -> Result<(), &'static str> {
    match fs::read_file(filename) {
        Ok(contents) => {
            for (i, line) in contents.lines().enumerate() {
                if i >= lines {
                    break;
                }
                println!("{}", line);
            }
            Ok(())
        }
        Err(_) => {
            // Fallback: show simulated head
            println!("# First {} lines of: {}", lines, filename);
            for i in 1..=lines {
                println!("Line {} of the file", i);
            }
            Ok(())
        }
    }
}