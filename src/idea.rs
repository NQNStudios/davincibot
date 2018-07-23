extern crate rusqlite;
extern crate serde_json;

use std::path::Path;

use rusqlite::{Connection};
use rusqlite::types::{Value, Null, ToSql};

use error::*;

// An Idea is the basic building block of Da Vinci Bot.
// TODO explain exactly how Ideas work and why
#[derive(Debug)]
pub struct Idea {
    pub id: i64,

    pub name: String,
    pub description: String,
    pub tags: Vec<String>,

    pub parent_id: Option<i64>,
    pub child_ids: Vec<i64>,
}

pub struct IdeaTree {
    conn: Connection,
}

impl IdeaTree {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<IdeaTree> {
        let conn = Connection::open(path)?;
        let mut tree = IdeaTree { conn };

        // Create the Idea table in the database if one doesn't exist. 
        tree.conn.execute("CREATE TABLE IF NOT EXISTS ideas (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            tags TEXT NOT NULL,

            parent_id INTEGER,
            child_ids TEXT NOT NULL)", &[])?;

        // Create the root Idea in the database if one doesn't exist.
        // Also create a .ignore Idea to ignore the default ignore tags
        if let Err(_) = tree.get_idea(1) {
            tree.create_idea(None)?;
            println!("Root idea: {:?}", tree.get_idea(1));
            tree.create_idea(Some(1))?;
            println!("Root idea after adding child: {:?}", tree.get_idea(1));
            println!("Child idea: {:?}", tree.get_idea(2));
        }

        Ok(tree)
    }

    pub fn create_idea(&mut self, parent_id: Option<i64>) -> Result<i64> {
        // Get the new id
        let parent_id: i64 = parent_id.unwrap_or(-1);

        let mut id_arg: &ToSql = &Null;
        if parent_id > 0 {
            id_arg = &parent_id;
        }

        {
            let mut statement = self.conn.prepare_cached("INSERT INTO ideas (name, description, tags, parent_id, child_ids) VALUES (?, ?, ?, ?, ?)")?;


            let args: &[&ToSql] = &[
                &"", // Name
                &"", // Description
                &"[]", // Tags
                id_arg,
                &"[]", // Child IDS
            ];

            let rows_changed = statement.execute(args)?;

            if rows_changed != 1 {
                panic!("Sqlite Idea insertion claims to be successful, but modified {} rows instead of 1", rows_changed); 
            }
        }

        let new_id = self.conn.last_insert_rowid();

        if parent_id > 0 {
            let mut parent_child_ids = self.get_child_ids(parent_id)?;
            parent_child_ids.push(new_id);
            let new_child_ids = serde_json::to_string(&parent_child_ids)?;

            {
                let mut statement = self.conn.prepare_cached("UPDATE ideas SET child_ids=? where id=?")?;
                let args: &[&ToSql] = &[&new_child_ids, &parent_id.clone()];
                statement.execute(args)?;
            }
        }

        Ok(new_id)
    }

    pub fn get_name(&self, id: i64) -> Result<String> {
        let name: String = self.conn.query_row("SELECT name FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        Ok(name)
    }

    pub fn get_description(&self, id: i64) -> Result<String> {
        let name: String = self.conn.query_row("SELECT description FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        Ok(name)
    }

    pub fn get_tags(&self, id: i64) -> Result<Vec<String>> {
        let tags_json: String = self.conn.query_row("SELECT description FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        let tags: Vec<String> = serde_json::from_str(tags_json.as_str())?;
        Ok(tags)
    }

    pub fn get_parent_id(&self, id: i64) -> Result<Option<i64>> {
        let parent_id: Value = self.conn.query_row("SELECT parent_id FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        match parent_id {
            Value::Null => Ok(None),
            Value::Integer(id) => Ok(Some(id)),
            _ => panic!("The parent_id row of an Idea in the database is neither Null nor an ID!")
        }
    }

    pub fn get_child_ids(&self, id: i64) -> Result<Vec<i64>> {
        let child_ids_json: String = self.conn.query_row("SELECT child_ids FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        let child_ids: Vec<i64> = serde_json::from_str(child_ids_json.as_str())?;
        Ok(child_ids)
    }

    pub fn get_idea(&self, id: i64) -> Result<Idea> { 
        let idea: Result<Idea> = self.conn.query_row("SELECT * FROM ideas WHERE id=?", &[&id], |row| -> Result<Idea> {
            let tags = serde_json::from_str(row.get::<i32, String>(3).as_str())?;
            let child_ids = serde_json::from_str(row.get::<i32, String>(5).as_str())?;

            Ok(Idea {
                id,
                name: row.get(1),
                description: row.get(2),
                tags,
                parent_id: match row.get(4) {
                    Value::Null => None,
                    Value::Integer(id) => Some(id),
                    _ => panic!("The parent_id row of an Idea in the database is neither Null nor an ID!")
                },
                child_ids,
            })
        })?;
        idea
    }
}
