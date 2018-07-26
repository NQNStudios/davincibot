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
            tree.create_root_idea()?;
            /*println!("Root idea: {:?}", tree.get_idea(1));*/
            tree.create_idea(1, ".ignore".to_string(), Some([
                                     Some(&"The tags applied to this Idea will be hidden when listing children of this Idea's parent or any of its children that don't have their own .ignore child."),
                                     Some(&serde_json::to_string(&vec![&"done", &"hidden", &"archived", &"paused"])?.as_str()),
                                     None,
            ]))?;
            /*println!("Root idea after adding child: {:?}", tree.get_idea(1));*/
            /*println!("Child idea: {:?}", tree.get_idea(2));*/
        }

        Ok(tree)
    }

    fn create_root_idea(&mut self) -> Result<()> {
        let mut statement = self.conn.prepare_cached("INSERT INTO ideas (name, description, tags, parent_id, child_ids) VALUES (?, ?, ?, ?, ?)")?;


        let args: &[&ToSql] = &[
            &"Do All the Vastly Impractical Nonsense Conceivable In (short) Bursts Of Time!", // Name
            &"This is the root Idea for your Da Vinci Bot project.", // Description
            &"[]", // Tags
            &Null, // Parent ID
            &"[]", // Child IDS
        ];

        statement.execute(args)?;
        Ok(())
    }

    pub fn error_on_duplicate_child(&self, parent_id: i64, name: String) -> Result<()> {
        // Check for a duplicate-named children.
        let child_ids = self.get_child_ids(parent_id, true)?;
        let child_names: Vec<String> = child_ids.into_iter().map(|id| self.get_name(id).expect("A child id was listed for an Idea that doesn't exist!")).collect();
        if child_names.contains(&name) {
            return Err(Error::DaVinci(format!("Idea #{} already has a child named '{}'", parent_id, name)));
        }

        Ok(())
    }

    // TODO need to validate the names of ideas being created, or renamed
    // Names should not be able to
    // contain:  "->",
    // start with: "exit", "/", "^", "@", or a digit
    // or have leading/trailing whitespace
    pub fn create_idea(&mut self, parent_id: i64, name: String, args: Option<[Option<&ToSql>; 3]>) -> Result<i64> {
        self.error_on_duplicate_child(parent_id, name.clone())?;
        if name.trim().len() == 0 {
            return Err(Error::DaVinci("Can't make an Idea without a name".to_string()));
        }

        let new_id = {

            let mut statement = self.conn.prepare_cached("INSERT INTO ideas (name, description, tags, child_ids, parent_id) VALUES (?, ?, ?, ?, ?)")?; 
            let default_args: [&ToSql; 5] = [
                &name, // Name
                &"", // Description
                &"[]", // Tags
                &"[]", // Child IDS
                &Null, // Parent ID
            ];

            // TODO document how to create an Idea with preset field values
            // by passing a ToSql array.
            let mut creation_args: Vec<&ToSql> = Vec::new();
            
            if let Some(user_args) = args {
                creation_args.push(&name);
                creation_args.extend(
                    user_args.into_iter().enumerate().map(|arg| match arg {
                        (_, Some(arg)) => arg.clone(),
                        (idx, None) => default_args[idx].clone(),
                    }));
                creation_args.push(&Null);
            }
            else {
                creation_args.extend(default_args.iter());
            }


            statement.execute(&creation_args)?;
            self.conn.last_insert_rowid()
        };

        self.set_parent(new_id, parent_id)?;
        Ok(new_id)
    }


    fn add_child(&mut self, parent_id: i64, child_id: i64) -> Result<()> {
        self.error_on_duplicate_child(parent_id, self.get_name(child_id)?)?;

        let mut child_ids = self.get_child_ids(parent_id, true)?;
        child_ids.push(child_id);

        let mut statement = self.conn.prepare_cached("UPDATE ideas SET child_ids=? where id=?")?;
        let args: &[&ToSql] = &[
            &serde_json::to_string(&child_ids)?,
            &parent_id
        ];

        statement.execute(args)?;

        // Set the child's parent ID to the new parent ID
        let mut statement = self.conn.prepare_cached("UPDATE ideas SET parent_id=? where id=?")?;
        let args: &[&ToSql] = &[
            &parent_id,
            &child_id
        ];
        statement.execute(args)?;
        Ok(())
    }

    fn remove_child(&mut self, parent_id: i64, child_id: i64) -> Result<()> {
        // Remove the child from the child list of its parent
        let mut child_ids = self.get_child_ids(parent_id, true)?;
        child_ids.retain(|&id| id != child_id);
        let mut statement = self.conn.prepare_cached("UPDATE ideas SET child_ids=? where id=?")?;
        let args: &[&ToSql] = &[
            &serde_json::to_string(&child_ids)?,
            &parent_id
        ];

        statement.execute(args)?;

        // Set the child's parent ID to null
        let mut statement = self.conn.prepare_cached("UPDATE ideas SET parent_id=? where id=?")?;
        let args: &[&ToSql] = &[
            &Null,
            &child_id
        ];
        statement.execute(args)?;
        Ok(())
    }

    pub fn set_parent(&mut self, child_id: i64, parent_id: i64) -> Result<()> {
        if child_id == 1 {
            return Err(Error::DaVinci("Cannot move the Root idea.".to_string()));
        }

        // Get the child's old parent so we can sever that bond later
        let old_parent_id = self.get_parent_id(child_id)?;

        // Attempt to add the child to the new parent FIRST, because this may
        // fail if its name is a duplicate
        self.add_child(parent_id, child_id)?;

        if let Some(old_parent_id) = old_parent_id {
            // Finally, sever the old parent-child relationship
            self.remove_child(old_parent_id, child_id)?;
        }
        Ok(())
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
        let tags_json: String = self.conn.query_row("SELECT tags FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        let tags: Vec<String> = serde_json::from_str(tags_json.as_str())?;
        Ok(tags)
    }

    pub fn set_tags(&mut self, id: i64, tags: Vec<String>) -> Result<()> {
        let mut statement = self.conn.prepare_cached("UPDATE ideas SET tags=? WHERE id=?")?;

        statement.execute(&[&serde_json::to_string(&tags)?, &id])?;
        Ok(())
    }

    pub fn clear_tags(&mut self, id: i64) -> Result<()> {
        self.set_tags(id, Vec::new())
    }

    pub fn remove_tags(&mut self, id: i64, tags: Vec<String>) -> Result<()> {
        let mut new_tags = self.get_tags(id)?;
        new_tags.retain(|tag| !tags.contains(tag));
        self.set_tags(id, new_tags)
    }

    pub fn add_tags(&mut self, id: i64, tags: Vec<String>) -> Result<()> {
        let mut new_tags = self.get_tags(id)?;
        new_tags.extend(tags);

        self.set_tags(id, new_tags)
    }

    pub fn get_parent_id(&self, id: i64) -> Result<Option<i64>> {
        let parent_id: Value = self.conn.query_row("SELECT parent_id FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        match parent_id {
            Value::Null => Ok(None),
            Value::Integer(id) => Ok(Some(id)),
            _ => panic!("The parent_id row of an Idea in the database is neither Null nor an ID!")
        }
    }

    fn get_ignore_tags(&self, id: i64) -> Result<Vec<String>> {
        let idea = self.get_idea(id)?;
        // First check if this idea has an .ignore child to pull tags from
        for child_id in idea.child_ids{
            let child_idea = self.get_idea(child_id)?;
            if child_idea.name == ".ignore" {
                return Ok(child_idea.tags);
            }
        }

        // If not, check if the parent has an .ignore child and return those
        if let Some(parent_id) = idea.parent_id {
            return self.get_ignore_tags(parent_id);
        }

        Ok(Vec::new())
    }

    pub fn get_child_ids(&self, id: i64, include_hidden: bool) -> Result<Vec<i64>> {
        let child_ids_json: String = self.conn.query_row("SELECT child_ids FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        let mut child_ids: Vec<i64> = serde_json::from_str(child_ids_json.as_str())?;

        // TODO get the tags of this Idea's ignore child, or parent's ignore
        // child
        let ignore_tags = self.get_ignore_tags(id)?;

        if !include_hidden {
            child_ids.retain(|child_id| {
                let idea = self.get_idea(*child_id).unwrap();
                if idea.name.chars().into_iter().next().unwrap_or(' ') == '.' {
                    return false;
                }
                for ignore_tag in &ignore_tags {
                    if idea.tags.contains(ignore_tag) {
                        return false;
                    }
                }
                return true;
            });
        }

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
