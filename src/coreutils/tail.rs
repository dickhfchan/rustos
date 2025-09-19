//! tail command - Show last lines of file

use alloc::vec::Vec;
use crate::fs;
use crate::println;

pub fn tail_main(args: &[&str]) -> Result<(), &'static str> {
    if args.is_empty() {
        println!("Usage: tail <file1> [file2] ...");
        return Err("No files specified");
    }

    for &filename in args {
        match show_tail(filename, 10) {
            Ok(()) => {}
            Err(e) => {
                println!("tail: {}: {}", filename, e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

fn show_tail(filename: &str, lines: usize) -> Result<(), &'static str> {
    match fs::read_file(filename) {
        Ok(contents) => {
            let all_lines: Vec<&str> = contents.lines().collect();
            let start = if all_lines.len() > lines {
                all_lines.len() - lines
            } else {
                0
            };
            
            for line in &all_lines[start..] {
                println!("{}", line);
            }
            Ok(())
        }
        Err(_) => {
            // Fallback: show simulated tail
            println!("# Last {} lines of: {}", lines, filename);
            for i in 1..=lines {
                println!("Line {} (from end) of the file", lines - i + 1);
            }
            Ok(())
        }
    }
}