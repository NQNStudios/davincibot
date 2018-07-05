#[macro_use]
extern crate serde_derive;
extern crate serde;
// TODO use EDN instead?
extern crate serde_json;

#[macro_use]
extern crate clap;
use clap::{App, Arg, SubCommand, AppSettings, ArgSettings, ArgMatches, Values};

#[macro_use]
extern crate text_io;

use std::collections::Vec;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::io::SeekFrom;
use std::io::Write;
use std::env;

// An Idea is the basic building block of Da Vinci Bot.
// TODO explain exactly how Ideas work and why
#[derive(Serialize, Deserialize, Debug)]
struct Idea {
    // TODO is 64 bits enough for Da Vinci ID's?
    id: usize,
    name: String,
    description: String,
    tags: Vec<String>,
    child_ids: Vec<usize>,
    // TODO add attachments to Ideas
    // TODO add extra serde-serializable data to Ideas
}

// TODO this is bad practice -- use a database instead
static mut ideas: Vec<Idea> = vec![];

impl Idea {
    pub fn get(id: usize): Idea {
        ideas[id]
    }

    pub fn new(): Idea {
        Idea {
            id: ideas.len(),
            name: "",
            description: "",
            tags: vec![],
            child_ids: vec![],
        }
    }

    pub fn children(&self) {
        self.child_ids.map(|id| Idea::get(id));
    }

    // TODO implement this to use a database
    // This function loads the user's Da Vinci Ideas
    pub fn load(&str file) {
        // Open the save file, or create it if it doesn't exist
        println!("Loading Da Vinci file: {}", path);
        let mut file = OpenOptions::new().read(true).write(true).create(true).open(&file).unwrap();
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

        ideas
    }
    // TODO implement this with a database
    // This function writes the user's Da Vinci Ideas persistently to a file
    pub fn write(&str file) {
        // When the user is finished, write the idea vector to the Da Vinci file,
        // overwriting old contents
        println!("Writing to Da Vinci File: {}", path);
        file.set_len(0).expect("Failed to overwrite existing Da Vinci file.");
        file.seek(SeekFrom::Start(0)).expect("Failed to write at beginning of Da Vinci file.");
        serde_json::to_writer(file, &ideas).expect("Failed to write to Da Vinci file.");
    }

}


fn main() {
    // Use Clap to parse the command-line arguments
    let app = app_from_crate!("\n")
        .arg(Arg::with_name("file").index(1).required(false));

    let args = app.get_matches();

    // The default Da Vinci File is hidden in the home directory.
    let home_dir = env::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap();
    let default_path = format!("{}/.davincibot.json", &home_dir);
    let default_path = default_path.as_str();

    // The file where Da Vinci Bot's Ideas are stored can be optionally
    // specified as the first argument.
    let path = args.value_of("file").unwrap_or(default_path);

    Idea::load(&path);

    let mut repl = App::new("davincibot repl")
        .setting(AppSettings::NoBinaryName)
        .subcommand(SubCommand::with_name("add")
                    .arg(Arg::with_name("idea_name").index(1).multiple(true).require_delimiter(false)))
        .subcommand(SubCommand::with_name("exit"));

    // In REPL fashion, allow the user to type as many Da Vinci commands as
    // they want until "exit"
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let command: String = read!("{}\n");

        let matches = repl.get_matches_from_safe_borrow(command.split(" ")).unwrap();

        match matches.subcommand() {
            ("add", Some(sub_matches)) => add(sub_matches),
            ("exit", Some(_)) => break,
            _ => panic!("not a valid REPL command")
        };

        // Ensure the Ideas are saved after every operation (TODO this may be
        // inefficient. A database may help
        Idea::write(&path);
    }

}

fn add(matches: &ArgMatches) {
    println!("Adding!");
    println!("{}", matches.is_present("idea_name"));

    Idea::get(0);
    let values: Vec<&str> = matches.values_of("idea_name").unwrap().collect();
    println!("{:?}", values);
}
