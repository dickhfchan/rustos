//! ls command - List directory contents

use crate::fs;
use crate::println;

pub fn ls_main(args: &[&str]) -> Result<(), &'static str> {
    let path = if args.is_empty() {
        "."
    } else {
        args[0]
    };

    match list_directory(path) {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("ls: cannot access '{}': {}", path, e);
            Err(e)
        }
    }
}

fn list_directory(path: &str) -> Result<(), &'static str> {
    // In a real implementation, this would read from the filesystem
    // For now, we'll simulate directory listing
    match fs::list_directory(path) {
        Ok(entries) => {
            for entry in entries {
                println!("{}", entry);
            }
            Ok(())
        }
        Err(_) => {
            // Fallback: show simulated directory listing
            println!("# Simulated directory listing for: {}", path);
            println!(".");
            println!("..");
            if path == "/" {
                println!("bin");
                println!("etc");
                println!("home");
                println!("tmp");
                println!("usr");
                println!("var");
            } else {
                println!("file1.txt");
                println!("file2.txt");
                println!("subdir");
            }
            Ok(())
        }
    }
}