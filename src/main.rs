use std::env;
use std::path::PathBuf;

extern crate rusqlite;
extern crate serde_json;
extern crate edit_rs;
extern crate conv;

mod error;
mod idea;
mod repl;
mod core_commands;
mod core_printers;

use idea::*;
use repl::*;

fn main() {
    let home_path = env::home_dir().unwrap_or(PathBuf::new());
    let home_path = home_path.to_str().unwrap();
    let filename = env::args().skip(1).next().unwrap_or(format!("{}/project.dv", home_path));

    println!("Loading Da Vinci file: {}", filename);
    let mut tree = IdeaTree::open(filename).expect("Failed to create Da Vinci tree."); 

    Repl::new().run(&mut tree);
}

// TODO Interrupt ^C signal and treat it as "exit" instead of closing program
