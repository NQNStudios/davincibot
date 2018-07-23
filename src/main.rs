use std::env;
use std::path::PathBuf;

extern crate rusqlite;
extern crate serde_json;

mod error;
mod idea;

use idea::*;

fn main() {
    let home_path = env::home_dir().unwrap_or(PathBuf::new());
    let home_path = home_path.to_str().unwrap();
    let filename = env::args().skip(1).next().unwrap_or(format!("{}/project.dv", home_path));

    println!("Loading Da Vinci file: {}", filename);
    let mut tree = IdeaTree::open(filename).expect("Failed to create Da Vinci tree."); 


}

// TODO Interrupt ^C signal and treat it as "exit" instead of closing program
