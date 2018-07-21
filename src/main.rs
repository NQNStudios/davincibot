extern crate rusqlite;
extern crate serde_json;

mod error;
mod idea;

use idea::*;

fn main() {
    let tree = IdeaTree::open("test.sqlite").expect("Failed to create Da Vinci tree."); 
}

// TODO Interrupt ^C signal and treat it as "exit" instead of closing program
