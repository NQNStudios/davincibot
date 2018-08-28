use std::env;
use std::path::PathBuf;

extern crate davincibot;
use davincibot::idea::IdeaTree;
use davincibot::repl::Repl;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");


fn main() {
    let home_path = env::home_dir().unwrap_or(PathBuf::new());
    let home_path = home_path.to_str().unwrap();

    let VERSION_COMMANDS: Vec<&str> = vec!["-v", "-version", "--v", "--version", ];

    let arg = env::args().skip(1).next().unwrap_or(format!("{}/project.dv", home_path));

    if arg.chars().next() == Some('-') && VERSION_COMMANDS.contains(&arg.as_str()) {
        println!("Da Vinci Bot version: {}", VERSION);
    } else {

        println!("Loading Da Vinci file: {}", arg);
        let mut tree = IdeaTree::open(arg).expect("Failed to create Da Vinci tree."); 

        Repl::new().run(&mut tree);
    }
}

// TODO Interrupt ^C signal and treat it as "exit" instead of closing program
