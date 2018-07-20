use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

extern crate rusqlite;
use rusqlite::{Connection, Row};
use rusqlite::types::{ToSql, FromSql};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate clap;
use clap::{App, Arg, SubCommand, AppSettings, ArgMatches};

#[macro_use]
extern crate text_io;

extern crate subprocess;
use subprocess::{Exec, Redirection};

use std::vec::Vec;
use std::path::Path;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::io::Write;
use std::env;

static IDEA_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

struct DaVinciTree {
    conn: Connection,
}

mod error;
use error::*;


impl DaVinciTree {
    pub fn open<P: AsRef<Path>>(path: P) -> DVResult<DaVinciTree> {
        let conn = Connection::open(path)?;
        let tree = DaVinciTree { conn };

        // Create the Idea table in the database if one doesn't exist. 
        tree.conn.execute("CREATE TABLE IF NOT EXISTS ideas (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            tags TEXT NOT NULL,

            parent_id INTEGER,
            child_ids TEXT NOT NULL,
            attachments TEXT NOT NULL,
            extras TEXT NOT NULL)", &[])?;

        // Create the root Idea in the database if one doesn't exist.
        // Also create a .ignore Idea to ignore the default ignore tags
        if let Err(_) = tree.get_idea(0) {
            println!("No root idea");
        }

        Ok(tree)
    }

    pub fn get_name(&self, id: i64) -> DVResult<String> {
        let name: String = self.conn.query_row("SELECT name FROM ideas WHERE id=?", &[&id], |row| { row.get(1) })?;
        Ok(name)
    }

    pub fn get_idea(&self, id: u64) -> DVResult<Idea> { 
        Ok(Idea { .. Default::default() })
    }
}

// An Idea is the basic building block of Da Vinci Bot.
// TODO explain exactly how Ideas work and why
#[derive(Serialize, Deserialize, Debug, Default)]
struct Idea {
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

    fn from<Row>(row: Row) -> Idea {
        Idea { .. Default::default() }
    }

    // TODO print() is probably a bad function name
    pub fn print(ideas: &mut Vec<Idea>, id: usize) {
        let idea = &ideas[id];
        // TODO format this printout in a visually appealing way
        println!("#{}: {}", id, idea.name);
        for tag in &idea.tags {
            print!("[{}] ", tag);
        }
        println!();
        println!("---");
        println!("{}", idea.description);
        println!("{} children", idea.child_ids.len());
        // TODO print how many children are hidden
    }

    // TODO the ideas vector shouldn't be a necessary parameter when we use a database
    pub fn get(ideas: &mut Vec<Idea>, id: usize) -> &mut Idea {
        &mut ideas[id]
    }

    pub fn get_child_ids(ideas: &mut Vec<Idea>, id: usize) -> Vec<usize> {
        let parent_idea = Idea::get(ideas, id);
        parent_idea.child_ids.clone()
    }

    pub fn get_child_names(ideas: &mut Vec<Idea>, id: usize) -> Vec<String> {
        let child_ids = Idea::get_child_ids(ideas, id);

        child_ids.into_iter().map(|id| Idea::get(ideas, id).name.clone()).collect()
    }

    pub fn get_child_id_by_name(ideas: &mut Vec<Idea>, id: usize, name: &str) -> Option<usize> {
        let child_names = Idea::get_child_names(ideas, id);
        let child_ids = Idea::get_child_ids(ideas, id);

        for (idx, child_name) in child_names.into_iter().enumerate() {
            if child_name == name {
                return Some(child_ids[idx]);
            }
        }

        None
    }

    // Gets the list of tags that this Idea ignores in its children.
    // These are defined as the tags applied to this idea's child with the name
    // of .ignore, or if it doesn't, exist, the tags of its parent.
    pub fn get_ignore_tags(ideas: &mut Vec<Idea>, id: usize) -> Vec<String> {
        let parent_id = {
            let idea = Idea::get(ideas, id);
            idea.parent_id.clone()
        };

        let child_names = Idea::get_child_names(ideas, id);
        let child_ids = Idea::get_child_ids(ideas, id);

        if let Some(ignore_child_index) = child_names.into_iter().position(|name| name == ".ignore".to_string()) {
            /*println!("found ignore child index");*/

            return {
                let idea = Idea::get(ideas, child_ids[ignore_child_index]);
                idea.tags.clone()
            }
        }
        else if let Some(parent_id) = parent_id {
            /*println!("checking parent");*/

            return Idea::get_ignore_tags(ideas, parent_id);
        }
        else {
            /*println!("No ignore node found all the way up");*/
            return vec![];
        }
    }

    pub fn has_tag(ideas: &mut Vec<Idea>, id: usize, tag: &str) -> bool {
        let tags = {
            let idea = Idea::get(ideas, id);
            &idea.tags
        };

        tags.contains(&tag.to_string())
    }

    pub fn has_one_of_tags(ideas: &mut Vec<Idea>, id: usize, tags: &Vec<String>) -> bool {
        for tag in tags {
            if Idea::has_tag(ideas, id, tag) {
                return true;
            }
        }
        false
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

    /*pub fn set_parent(ideas: &mut Vec<Idea>, id_to_set: usize, new_parent_id: usize) {*/
        /*if let Some(old_parent_id) = {*/
            /*let idea = &ideas[id_to_move];*/
            /*idea.parent_id*/
        /*} {} else {*/
            /*panic!("Tried to move root Idea to a new parent!");*/
        /*}*/
    /*}*/

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
            child_ids: vec![1]
        };

        let default_ignore_idea = Idea {
            id: 1,
            name: ".ignore".to_string(),
            description: "".to_string(),
            tags: vec!["done".to_string(), "paused".to_string(), "archived".to_string(), "hidden".to_string()],
            parent_id: Some(0),
            child_ids: vec![],
        };

        IDEA_COUNT.fetch_add(2, Ordering::SeqCst);

        // Load ideas from file, or create vec containing root idea and write
        // to new file
        let ideas = match file_buffer.len() {
            0 => vec![default_root_idea, default_ignore_idea],
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
        /*println!("Writing to Da Vinci File: {}", path);*/
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        file.set_len(0).expect("Failed to overwrite Da Vinci file");
        serde_json::to_writer(file, &ideas).expect("Failed to write to Da Vinci file.");
    }

}


fn main() {
    let tree = DaVinciTree::open("test.sqlite").expect("Failed to create Da Vinci tree."); 

    /*// Use Clap to parse the command-line arguments*/
    /*let app = app_from_crate!("\n")*/
        /*.arg(Arg::with_name("file").index(1).required(false));*/

    /*let args = app.get_matches();*/

    /*// The default Da Vinci File is hidden in the home directory.*/
    /*let home_dir = env::home_dir().unwrap();*/
    /*let home_dir = home_dir.to_str().unwrap();*/
    /*let default_path = format!("{}/.davincibot.json", &home_dir);*/
    /*let default_path = default_path.as_str();*/

    /*// The file where Da Vinci Bot's Ideas are stored can be optionally*/
    /*// specified as the first argument.*/
    /*let path = args.value_of("file").unwrap_or(default_path);*/

    /*let mut ideas = Idea::load(&path);*/

    /*// TODO this is starting to get really ugly, perhaps with little benefit.*/
    /*// Maybe remove the clap.rs dependency.*/
    /*let mut repl = App::new("davincibot repl")*/
        /*.setting(AppSettings::NoBinaryName)*/
        /*.subcommand(SubCommand::with_name("add")*/
                    /*.arg(Arg::with_name("idea_name").index(1).multiple(true).require_delimiter(false)))*/
        /*.subcommand(SubCommand::with_name("describe"))*/
        /*.subcommand(SubCommand::with_name("list"))*/
        /*.subcommand(SubCommand::with_name("print"))*/
        /*.subcommand(SubCommand::with_name("select")*/
                    /*.arg(Arg::with_name("index").index(1)))*/
        /*.subcommand(SubCommand::with_name("up"))*/


        /*// TODO need a traverse command which traverses children of node*/
        /*.subcommand(SubCommand::with_name("tag")*/
                    /*.arg(Arg::with_name("tags").index(1).multiple(true).require_delimiter(false)))*/
        /*.subcommand(SubCommand::with_name("untag")*/
                    /*.arg(Arg::with_name("tags").index(1).multiple(true).require_delimiter(false)))*/
        /*// TODO need a clear all tags command*/
        /*// TODO need a search by tag command*/
        /*// (all of these will inevitably be rewritten post-Mentat)*/
        /*.subcommand(SubCommand::with_name("exit"));*/

    /*let mut selected_id: usize = 0;*/
    /*// In REPL fashion, allow the user to type as many Da Vinci commands as*/
    /*// they want until "exit"*/
    /*loop {*/
        /*print!("$ ");*/
        /*io::stdout().flush().unwrap();*/
        /*let command: String = read!("{}\n");*/

        /*let matches = repl.get_matches_from_safe_borrow(command.split(" ")).unwrap();*/

        /*match matches.subcommand() {*/
            /*("add", Some(sub_matches)) => add(sub_matches, &mut ideas, selected_id),*/
            /*("describe", _) => describe(&mut ideas, selected_id),*/
            /*("list", _) => list(&mut ideas, selected_id, false), // TODO add a way to list all, even hidden*/
            /*("print", _) => Idea::print(&mut ideas, selected_id),*/
            /*("select", Some(sub_matches)) => select(sub_matches, &mut ideas, &mut selected_id),*/
            /*("up", _) => up(&mut ideas, &mut selected_id),*/
            /*("tag", Some(sub_matches)) => set_tags(sub_matches, &mut ideas, selected_id, true),*/
            /*("untag", Some(sub_matches)) => set_tags(sub_matches, &mut ideas, selected_id, false),*/
            /*("exit", Some(_)) => break,*/
            /*_ => panic!("not a valid REPL command")*/
        /*};*/

        /*// Ensure the Ideas are saved after every operation (TODO this may be*/
        /*// inefficient. A database may help*/
        /*Idea::write(&ideas, &path);*/
    /*}*/

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
            // TODO getting a series of input lines terminated by "exit" like
            // this, should be a function with a closure for the inner logic
            print!("-> ");
            io::stdout().flush().unwrap();

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
// Answer: The mut is required because get() returns a mutable reference for
// convenient edits. Once all Idea modification happens through an API, this
// will be better.
fn list(ideas: &mut Vec<Idea>, selected_id: usize, all: bool) {
    let child_ids = Idea::get_child_ids(ideas, selected_id);
    let child_names = Idea::get_child_names(ideas, selected_id);
    let ignore_tags = Idea::get_ignore_tags(ideas, selected_id);

    /*println!("Ignore tags: {:?}", ignore_tags);*/

    let mut hidden = 0;
    for (idx, name) in child_names.into_iter().enumerate() {
        let first_char = name.chars().next().unwrap_or(' ');
        if (first_char != '.' && !Idea::has_one_of_tags(ideas, child_ids[idx], &ignore_tags)) || all {
            println!("{}. {}", idx+1, name);
        }
        else {
            hidden += 1;
        }
    }
    if hidden > 0 {
        println!();
        println!("- {} more hidden -", hidden);
    }
}

// TODO rather than call static Idea::get(), define an Ideas type that is
// Vec<Idea>.

fn select(matches: &ArgMatches, ideas: &mut Vec<Idea>, selected_id: &mut usize) {
    let index = usize::from_str_radix(matches.value_of("index").unwrap(), 10);
    // TODO if # precedes an id, it could be an absolute id we wish to select
    // by
    match index {
        Ok(index) => {
            // It could be a child index we wish to select by
            let idea = Idea::get(ideas, *selected_id);

            let new_selected_id = idea.child_ids[index - 1];
            *selected_id = new_selected_id;
        },
        Err(_) => {
            // It could be a child name we wish to select by
            let child_name = matches.value_of("index").unwrap();
            // TODO this only matches one-token names

            let new_selected_id = Idea::get_child_id_by_name(ideas, *selected_id, child_name).unwrap();
            *selected_id = new_selected_id;
        }
    }

}

fn up(ideas: &mut Vec<Idea>, selected_id: &mut usize) {
    let idea = Idea::get(ideas, *selected_id);
    if let Some(parent_id) = idea.parent_id {
        *selected_id = parent_id;
    }
}

fn set_tags(matches: &ArgMatches, ideas: &mut Vec<Idea>, selected_id: usize, tagged: bool) {
    let mut tags: Vec<String> = vec![];

    if matches.is_present("tags") {
        let values: Vec<&str> = matches.values_of("tags").unwrap().collect();
        for value in values {
            tags.push(value.to_string());
        }
    }

    let idea = Idea::get(ideas, selected_id);
    let current_tags = &mut (idea.tags);
    for tag in tags {
        if let Some(position) = current_tags.iter().position(|t| *t==tag) {
            // The tag is present, but should be removed
            if !tagged {
                current_tags.remove(position);
            }
        }
        else {
            // The tag is not present, but should be added
            if tagged {
                current_tags.push(tag);
            }
        }
    }
}

fn describe(ideas: &mut Vec<Idea>, selected_id: usize) {
    let idea = Idea::get(ideas, selected_id);

    let existing_description = idea.description.clone();
    println!("{}", existing_description);

    /*println!("{}", command);*/
    let file = OpenOptions::new().write(true).create(true).open(".IDEA_DESCRIPTION").unwrap();
    file.set_len(0).expect("Failed to overrwrite temp description file!");
    let _exit_status = Exec::cmd("echo").arg(&existing_description).stdout(Redirection::File(file)).join();
    /*println!("{:?}", exit_status);*/

    // TODO allow default text editors other than vim, and launch in
    // a platform-agnostic manner
    let _exit_status = Exec::cmd("vim").arg(".IDEA_DESCRIPTION").join();
    /*println!("{:?}", exit_status);*/
    let mut file = OpenOptions::new().read(true).open(".IDEA_DESCRIPTION").unwrap();
    let mut file_buffer = String::new();
    file.read_to_string(&mut file_buffer).expect("Failed to read from Da Vinci file.");
    file_buffer = file_buffer.trim().to_string();

    if existing_description != file_buffer {
        println!("Updating Idea description.");
        idea.description = file_buffer;
    }
}
// TODO Interrupt ^C signal and treat it as "exit" instead of closing program
