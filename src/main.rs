use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
#[macro_use]
extern crate serde_derive;
extern crate serde;
// TODO use EDN instead?
extern crate serde_json;

#[macro_use]
extern crate clap;
use clap::{App, Arg, SubCommand, AppSettings, ArgMatches};

#[macro_use]
extern crate text_io;

use std::vec::Vec;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::io::Write;
use std::env;

static IDEA_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

// An Idea is the basic building block of Da Vinci Bot.
// TODO explain exactly how Ideas work and why
#[derive(Serialize, Deserialize, Debug)]
struct Idea {
    // TODO is 64 bits enough for Da Vinci ID's?
    id: usize,
    name: String,
    description: String,
    tags: Vec<String>,

    parent_id: Option<usize>,
    child_ids: Vec<usize>,
    // TODO add attachments to Ideas
    // TODO add extra serde-serializable data to Ideas
}

impl Idea {
    // TODO the ideas vector shouldn't be necessary in this when we use a database
    pub fn get(ideas: &mut Vec<Idea>, id: usize) -> &mut Idea {
        &mut ideas[id]
    }

    pub fn get_child_names(ideas: &mut Vec<Idea>, id: usize) -> Vec<String> {
        let child_ids = {
            let parent_idea = Idea::get(ideas, id);
            parent_idea.child_ids.clone()
        };

        child_ids.into_iter().map(|id| Idea::get(ideas, id).name.clone()).collect()
    }

    pub fn new(ideas: &mut Vec<Idea>, parent_id: usize) -> &mut Idea {
        let new_idea = Idea {
            id: IDEA_COUNT.load(Ordering::SeqCst),
            name: "".to_string(),
            description: "".to_string(),
            tags: vec![],
            parent_id: Some(parent_id),
            child_ids: vec![],
        };

        let index = new_idea.id;
        // Make sure the parent references the child
        ideas[parent_id].child_ids.push(index);
        ideas.push(new_idea);
        IDEA_COUNT.fetch_add(1, Ordering::SeqCst);

        &mut ideas[index]
    }

    // TODO with a database, this shouldn't be necessary. Instead we'll just
    // use get() to lookup by ID
    // This function loads the user's Da Vinci Ideas
    pub fn load(path: &str) -> Vec<Idea> {
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
            parent_id: None,
            child_ids: vec![]
        };
        IDEA_COUNT.fetch_add(1, Ordering::SeqCst);

        // Load ideas from file, or create vec containing root idea and write
        // to new file
        let ideas = match file_buffer.len() {
            0 => vec![default_root_idea],
            _ => {
                let idea_list: Vec<Idea> = serde_json::from_str(&file_buffer).unwrap();
                IDEA_COUNT.store(idea_list.len(), Ordering::SeqCst);
                idea_list
            },
        };

        ideas
    }
    // TODO implement this with a database
    // This function writes the user's Da Vinci Ideas persistently to a file
    pub fn write(ideas: &Vec<Idea>, path: &str) {
        // When the user is finished, write the idea vector to the Da Vinci file,
        // overwriting old contents
        println!("Writing to Da Vinci File: {}", path);
        let mut file = OpenOptions::new().write(true).open(&path).unwrap();
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

    let mut ideas = Idea::load(&path);

    let mut repl = App::new("davincibot repl")
        .setting(AppSettings::NoBinaryName)
        .subcommand(SubCommand::with_name("add")
                    .arg(Arg::with_name("idea_name").index(1).multiple(true).require_delimiter(false)))
        .subcommand(SubCommand::with_name("list"))
        .subcommand(SubCommand::with_name("select")
                    .arg(Arg::with_name("index").index(1)))
        .subcommand(SubCommand::with_name("up"))
        // TODO need a print command to print name, description (markdown
        // formatted)
        // TODO need a traverse command which traverses children of node
        // TODO need an add-tag, remove-tag command
        // TODO need a search by tag command
        // (all of these will inevitably be rewritten post-Mentat)
        .subcommand(SubCommand::with_name("exit"));

    let mut selected_id: usize = 0;
    // In REPL fashion, allow the user to type as many Da Vinci commands as
    // they want until "exit"
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let command: String = read!("{}\n");

        let matches = repl.get_matches_from_safe_borrow(command.split(" ")).unwrap();

        match matches.subcommand() {
            ("add", Some(sub_matches)) => add(sub_matches, &mut ideas, selected_id),
            ("list", _) => list(&mut ideas, selected_id),
            ("select", Some(sub_matches)) => select(sub_matches, &mut ideas, &mut selected_id),
            ("up", _) => up(&mut ideas, &mut selected_id),
            ("exit", Some(_)) => break,
            _ => panic!("not a valid REPL command")
        };

        // Ensure the Ideas are saved after every operation (TODO this may be
        // inefficient. A database may help
        Idea::write(&ideas, &path);
    }

}

fn add(matches: &ArgMatches, ideas: &mut Vec<Idea>, selected_id: usize) {
    let mut child_names: Vec<String> = vec![];

    if matches.is_present("idea_name") {
        let values: Vec<&str> = matches.values_of("idea_name").unwrap().collect();
        child_names.push(values.join(" "));
    }
    else {
        // add to child_names until "exit" is encountered
        loop {
            let idea_name: String = read!("{}\n");
            if idea_name == "exit" {
                break;
            }
            child_names.push(idea_name);
        }
    }

    // TODO loop through child names creating ideas and storing their ids in
    // a vector
    for name in child_names {
        let child = Idea::new(ideas, selected_id);
        child.name = name;
    }
}

// TODO it seems to me ideas shouldn't require mut here -- am I wrong?
fn list(ideas: &mut Vec<Idea>, selected_id: usize) {
    let child_names = Idea::get_child_names(ideas, selected_id);
    let mut idx = 1;
    for name in child_names {
        println!("{}. {}", idx, name);
        idx += 1;
    }
}

// TODO rather than call static Idea::get(), define an Ideas type that is
// Vec<Idea>.

fn select(matches: &ArgMatches, ideas: &mut Vec<Idea>, selected_id: &mut usize) {
    let index = usize::from_str_radix(matches.value_of("index").unwrap(), 10).unwrap();

    let idea = Idea::get(ideas, *selected_id);

    let new_selected_id = idea.child_ids[index - 1];
    *selected_id = new_selected_id;
}

fn up(ideas: &mut Vec<Idea>, selected_id: &mut usize) {
    let idea = Idea::get(ideas, *selected_id);
    if let Some(parent_id) = idea.parent_id {
        *selected_id = parent_id;
    }
}

// TODO wishlist
// scan command allows user to scan an ISBN barcode to add a Book Idea
