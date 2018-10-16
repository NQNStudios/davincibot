use std::env;
use std::path::{Path,PathBuf};

extern crate dirs;

extern crate davincibot;
use davincibot::idea::IdeaTree;
use davincibot::repl::{Repl, VERSION};

fn main() {
    let home_path = dirs::home_dir().unwrap_or(PathBuf::new());
    let home_path = home_path.to_str().unwrap();

    let version_commands: Vec<&str> = vec!["-v", "-version", "--v", "--version", ];

    let default_tree_file = match Path::new(&format!("{}/yggdrasil/", home_path)).exists() {
        true => {
            println!("WARNING! Opening dv file from a repository. Have you pulled?");
            "yggdrasil/project.dv"
        },
        false => "project.dv",
    };
    let default_tree_file = format!("{}/{}", home_path, default_tree_file);

    let arg = env::args().skip(1).next().unwrap_or(default_tree_file);
    println!("{}", arg);

    if arg.chars().next() == Some('-') && version_commands.contains(&arg.as_str()) {
        println!("Da Vinci Bot version: {}", VERSION);
    } else {

        println!("Loading Da Vinci file: {}", arg);
        let mut tree = IdeaTree::open(arg).expect("Failed to create Da Vinci tree."); 

        Repl::new().run(&mut tree);
    }
}

// TODO Interrupt ^C signal and treat it as "exit" instead of closing program
// TODO Interrupt ^D signal and close program
