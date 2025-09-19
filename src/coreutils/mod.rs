//! RustOS Coreutils - No-std kernel-space implementations of Unix utilities
//! Inspired by uutils/coreutils but adapted for bare-metal kernel environment

use crate::println;

pub mod cat;
pub mod ls;
pub mod echo;
pub mod pwd;
pub mod mkdir;
pub mod touch;
pub mod rm;
pub mod cp;
pub mod mv;
pub mod wc;
pub mod head;
pub mod tail;

/// Execute a coreutils command with arguments
pub fn execute_command(command: &str, args: &[&str]) -> Result<(), &'static str> {
    match command {
        "cat" => cat::cat_main(args),
        "ls" => ls::ls_main(args),
        "echo" => echo::echo_main(args),
        "pwd" => pwd::pwd_main(args),
        "mkdir" => mkdir::mkdir_main(args),
        "touch" => touch::touch_main(args),
        "rm" => rm::rm_main(args),
        "cp" => cp::cp_main(args),
        "mv" => mv::mv_main(args),
        "wc" => wc::wc_main(args),
        "head" => head::head_main(args),
        "tail" => tail::tail_main(args),
        "help" | "--help" => {
            show_help();
            Ok(())
        }
        _ => {
            println!("rustos: command not found: {}", command);
            println!("Type 'help' for available commands");
            Err("Command not found")
        }
    }
}

fn show_help() {
    println!("RustOS Coreutils - Available commands:");
    println!("  cat     - Display file contents");
    println!("  ls      - List directory contents");
    println!("  echo    - Display text");
    println!("  pwd     - Show current directory");
    println!("  mkdir   - Create directories");
    println!("  touch   - Create files");
    println!("  rm      - Remove files/directories");
    println!("  cp      - Copy files");
    println!("  mv      - Move/rename files");
    println!("  wc      - Word count");
    println!("  head    - Show first lines of file");
    println!("  tail    - Show last lines of file");
    println!("  help    - Show this help");
}

/// Initialize coreutils subsystem
pub fn init() {
    println!("Coreutils initialized - {} commands available", 12);
}