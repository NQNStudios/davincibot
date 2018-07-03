#[macro_use]
extern crate serde_derive;
extern crate serde;
// TODO use EDN instead?
extern crate serde_json;

#[macro_use]
extern crate clap;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
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
    // Use Clap to parse the command-line arguments defined in start.yml
    use clap::App;
    let args_yaml = load_yaml!("start.yml");
    let args = App::from_yaml(args_yaml).get_matches();

    // The default Da Vinci File is hidden in the home directory.
    let home_dir = env::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap();
    let default_path = format!("{}/.davincibot.json", &home_dir);
    let default_path = default_path.as_str();

    // The file where Da Vinci Bot's Ideas are stored can be optionally
    // specified as the first argument.
    let path = args.value_of("file").unwrap_or(default_path);

    println!("Hello, Mx. Da Vinci!");

 
    // Open the save file, or create it if it doesn't exist
    println!("Loading Da Vinci file: {}", path);
    let mut file = OpenOptions::new().read(true).write(true).create(true).open(&path).unwrap();

    let mut file_buffer = String::new();
    file.read_to_string(&mut file_buffer).expect("Failed to read from Da Vinci file.");

    // This is the first Idea a new user will see
    let default_root_idea: Idea = Idea {
        id: 0,
        name: "Do All the Vastly Impractical Nonsense Conceivable In (short) Bursts Of Time".to_string(),
        description: "Here's the root of all your brilliant Ideas.".to_string(),
        tags: vec![],
        child_ids: vec![]
    };

    // Load ideas from file, or create vec containing root idea and write
    // to new file
    let ideas = match file_buffer.len() {
        0 => vec![default_root_idea],
        _ => serde_json::from_str(&file_buffer).unwrap(),
    };

    // TODO main command loop

    // TODO use subcommand_name() and subcommand_matches() to pipe REPL
    // commands from commands.yml through handler functions

    // When the user is finished, write the idea vector to the Da Vinci file,
    // overwriting old contents
    println!("Writing to Da Vinci File: {}", path);
    file.set_len(0).expect("Failed to overwrite existing Da Vinci file.");
    file.seek(SeekFrom::Start(0)).expect("Failed to write at beginning of Da Vinci file.");
    serde_json::to_writer(file, &ideas).expect("Failed to write to Da Vinci file.");
}
