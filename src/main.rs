#[macro_use]
extern crate serde_derive;
extern crate serde;
// TODO use EDN instead?
extern crate serde_json;

use std::collections::VecDeque;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;
use std::env;

// An Idea is the basic building block of Da Vinci Bot.
// TODO explain exactly how Ideas work and why

#[derive(Serialize, Deserialize, Debug)]
struct Idea {
    // TODO is 64 bits enough for Da Vinci ID's?
    id: i64,
    name: String,
    description: String,
    tags: Vec<String>,
    child_ids: Vec<i64>,
    // TODO add attachments to Ideas
    // TODO add extra serde-serializable data to Ideas
}

fn main() {
    println!("Hello, Mx. Da Vinci!");

    // The first argument is the path to a Da Vinci Json file
    // (default='~/.davincibot.json')
    let mut args: VecDeque<String> = env::args().collect();
    let home_dir: Option<PathBuf> = env::home_dir();
    let home_dir: PathBuf = home_dir.unwrap();
    let home_dir: Option<&str> = home_dir.to_str();
    let home_dir: &str = home_dir.unwrap();
    println!("{}", &home_dir);
    let default_path = format!("{}/.davincibot.json", &home_dir);

    // Ignore the program name arg
    args.pop_front();
    let path = args.pop_front().unwrap_or(default_path);

    // Open the save file, or create it

    println!("Loading Da Vinci file: {}", path);
    let mut file = OpenOptions::new().read(true).write(true).create(true).open(&path).unwrap();


    // Open the Da Vinci file
    //let path = Path::new(&path);
    //let mut file = OpenOptions::new().read(true).write(true).create(true).open(path).unwrap();

    println!("successfully opened file");
    let mut file_buffer = String::new();
    file.read_to_string(&mut file_buffer).expect("Hello this didn't work");

    // TODO load ideas from file, or create vec containing root idea and write
    // to new file

    let root_idea = Idea { id: 0, name: "Do All the Vastly Impractical Nonsense Conceivable In (short) Bursts Of Time".to_string(), description: "Here's the root of all your brilliant Ideas.".to_string(), tags: vec!(), child_ids: vec!() };
    let next_idea = Idea { id: 1, name: "SLIGHTLY DIFFERENT Do All the Vastly Impractical Nonsense Conceivable In (short) Bursts Of Time".to_string(), description: "Here's the root of all your brilliant Ideas.".to_string(), tags: vec!(), child_ids: vec!() };
    let idea_vec = vec!(root_idea, next_idea);

    let serialized = serde_json::to_string(&idea_vec).unwrap();
    println!("serialized={}", serialized);
    let deserialized: Vec<Idea> = serde_json::from_str(&serialized).unwrap();
    println!("deserialized={:?}", deserialized);
}
