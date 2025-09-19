//! pwd command - Print working directory

use crate::fs;
use crate::println;

pub fn pwd_main(_args: &[&str]) -> Result<(), &'static str> {
    match fs::get_current_directory() {
        Ok(path) => {
            println!("{}", path);
            Ok(())
        }
        Err(_) => {
            // Fallback: show simulated current directory
            println!("/");
            Ok(())
        }
    }
}