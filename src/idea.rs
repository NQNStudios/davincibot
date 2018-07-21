extern crate rusqlite;
extern crate serde_json;

use std::path::Path;

use rusqlite::{Connection};

use error::*;

/*static IDEA_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;*/

// An Idea is the basic building block of Da Vinci Bot.
// TODO explain exactly how Ideas work and why
#[derive(Debug, Default)]
pub struct Idea {
    id: usize,
    name: String,
    description: String,
    tags: Vec<String>,

    parent_id: Option<usize>,
    child_ids: Vec<usize>,
    // TODO add attachments to Ideas
    // TODO add extra serde-serializable data to Ideas
}

pub struct IdeaTree {
    conn: Connection,
}

impl IdeaTree {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<IdeaTree> {
        let conn = Connection::open(path)?;
        let tree = IdeaTree { conn };

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
        if let Err(_) = tree.get_name(0) {
            println!("No root idea");
        }

        Ok(tree)
    }

    pub fn get_name(&self, id: i64) -> Result<String> {
        let name: String = self.conn.query_row("SELECT name FROM ideas WHERE id=?", &[&id], |row| { row.get(1) })?;
        Ok(name)
    }

    pub fn get_description(&self, id: i64) -> Result<String> {
        let name: String = self.conn.query_row("SELECT description FROM ideas WHERE id=?", &[&id], |row| { row.get(2) })?;
        Ok(name)
    }

    pub fn get_tags(&self, id: i64) -> Result<Vec<String>> {
        let tags_json: String = self.conn.query_row("SELECT description FROM ideas where id=?", &[&id], |row| { row.get(3) })?;
        let tags: Vec<String> = serde_json::from_str(tags_json.as_str())?;
        Ok(tags)
    }

    pub fn get_idea(&self, id: u64) -> Result<Idea> { 
        Ok(Idea { .. Default::default() })
    }
}
