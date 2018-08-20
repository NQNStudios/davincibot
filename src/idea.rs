extern crate rusqlite;
extern crate yaml_rust;

use std::path::Path;

use yaml_rust::{YamlLoader, YamlEmitter, Yaml};

use rusqlite::{Connection, Row};
use rusqlite::types::{Value, Null, ToSql};

use error::*;


// NOTE unwrap is used below because Da Vinci Bot promises only to put
// string values in the tags field of the database:
fn tag_vec_from_yaml(yaml: &str) -> Vec<String> {
    YamlLoader::load_from_str(yaml).unwrap().remove(0).into_iter().map(|tag| tag.as_str().unwrap().to_string()).collect()
}

// NOTE unwrap is used below because Da Vinci Bot promises only to put
// i64 values in the child_ids field of the database:
fn id_vec_from_yaml(yaml: &str) -> Vec<i64> {
    YamlLoader::load_from_str(yaml).unwrap().remove(0).into_iter().map(|child_id| child_id.as_i64().unwrap()).collect()
}

fn tag_vec_to_yaml(vec: Vec<String>) -> String {
    let mut yaml = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut yaml);
        emitter.dump(&Yaml::Array(vec.into_iter().map(|tag| Yaml::String(tag)).collect())).expect("Serialization failed");
    }
    yaml
}

fn id_vec_to_yaml(vec: Vec<i64>) -> String {
    let mut yaml = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut yaml);
        emitter.dump(&Yaml::Array(vec.into_iter().map(|id| Yaml::Integer(id)).collect())).expect("Failed");
    }
    yaml
}

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

impl Idea {
    pub fn get_yaml_data(&self) -> Result<Option<Yaml>> {
        Ok(YamlLoader::load_from_str(&self.description)?.into_iter().next())
    }

    pub fn format_name_with_tags(&self) -> String {
        let mut buffer = String::new();
        buffer += &self.name;

        for tag in &self.tags {
            buffer += &format!(" [{}]", tag);
        }

        buffer
    }
}

pub struct IdeaTree {
    conn: Connection,
}

impl IdeaTree {
    pub fn open_in_memory() -> Result<IdeaTree> { 
        IdeaTree::create(Connection::open_in_memory()?)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<IdeaTree> {
        IdeaTree::create(Connection::open(path)?)
    }

    fn create(conn: Connection) -> Result<IdeaTree> {
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
        if let Err(_) = tree.get_idea(1) {
            tree.create_root_idea()?;
            // Also create a .ignore Idea to ignore the default ignore tags
            tree.create_idea(1,
                             ".ignore".to_string(),
                             Some([
                                 Some(&"The tags applied to this Idea will be hidden when listing children of this Idea's parent or any of its children that don't have their own .ignore child."),
                                 Some(&r#"["done", "hidden", "archived", "paused"]"#),
                                 None,
                             ]))?;

            // Also create a .settings Idea with sensible defaults for
            // truncating Idea output
            tree.create_idea(1,
                             ".settings".to_string(),
                             Some([
                                  Some(&"tutorial: The settings defined in Yaml format in this Idea's description will be applied to operations on all of the root idea's children by default, but can be overridden by adding another .settings child.\nmax_description: 300"),
                                  None,
                                  None,
                             ]))?;

            // Also create a .shortcuts Idea with single-character shortcuts
            // for the most common core commands
            tree.create_idea(1,
                             ".shortcuts".to_string(),
                             Some([
                                  Some(&r#"
                                      h: help
                                      p: print
                                      l: list
                                      s: select
                                      a: add
                                      t: tag
                                      u: untag
                                      m: move
                                      d: describe
                                      r: rename
                                       "#
                                   ),
                                   None,
                                   None,
                             ]))?;
        }

        Ok(tree)
    }

    fn create_root_idea(&mut self) -> Result<()> {
        let mut statement = self.conn.prepare_cached("INSERT INTO ideas (name, description, tags, parent_id, child_ids) VALUES (?, ?, ?, ?, ?)")?;


        let args: &[&ToSql] = &[
            &"Do All the Vastly Impractical Nonsense Conceivable In (short) Bursts Of Time!", // Name
            &"This is the root Idea for your Da Vinci Bot project.\nType 'help' for a command list.\n\nSupport this free, open-source project by contributing on GitHub (https://github.com/NQNStudios/davincibot) or Patreon (https://patreon.com/natquaylenelson).", // Description
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
    // contain:  "->", "/", "[", "]"
    // start with: "exit", "^", "@", or a digit
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
                        (idx, None) => default_args[idx+1].clone(),
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
            &id_vec_to_yaml(child_ids),
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
            &id_vec_to_yaml(child_ids),
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

    // TODO delete this function in favor of Idea.format_name_with_tags()
    pub fn get_name_with_tags(&self, id: i64) -> Result<String> {
        let mut buffer = self.get_name(id)?;
        let tags = self.get_tags(id, false)?;

        for tag in tags {
            buffer += &format!(" [{}]", tag);
        }

        Ok(buffer)
    }

    pub fn set_name(&self, id: i64, name: &String) -> Result<()> {
        let mut statement = self.conn.prepare_cached("UPDATE ideas SET name=? WHERE id=?")?;

        statement.execute(&[name, &id])?;
        Ok(())
    }

    pub fn get_description(&self, id: i64) -> Result<String> {
        let name: String = self.conn.query_row("SELECT description FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        Ok(name)
    }

    pub fn set_description(&mut self, id: i64, description: &String) -> Result<()> {
        let mut statement = self.conn.prepare_cached("UPDATE ideas SET description=? WHERE id=?")?;

        statement.execute(&[description, &id])?;
        Ok(())
    }

    pub fn get_tags(&self, id: i64, inherit_tags: bool) -> Result<Vec<String>> {
        let tags_yaml: String = self.conn.query_row("SELECT tags FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        let mut tags: Vec<String> = tag_vec_from_yaml(&tags_yaml);

        if inherit_tags {
            if let Some(parent_id) = self.get_parent_id(id)? {
                let parent_tags = self.get_tags(parent_id, true)?;

                tags.extend(parent_tags);
                tags.sort();
                tags.dedup();
            }
        }

        Ok(tags)
    }

    pub fn set_tags(&mut self, id: i64, tags: Vec<String>) -> Result<()> {
        let mut statement = self.conn.prepare_cached("UPDATE ideas SET tags=? WHERE id=?")?;

        statement.execute(&[&tag_vec_to_yaml(tags), &id])?;
        Ok(())
    }

    pub fn clear_tags(&mut self, id: i64) -> Result<()> {
        self.set_tags(id, Vec::new())
    }

    pub fn remove_tags(&mut self, id: i64, tags: Vec<String>) -> Result<()> {
        let mut new_tags = self.get_tags(id, false)?;
        new_tags.retain(|tag| !tags.contains(tag));
        self.set_tags(id, new_tags)
    }

    pub fn add_tags(&mut self, id: i64, tags: Vec<String>) -> Result<()> {
        let mut new_tags = self.get_tags(id, false)?;
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

    /*pub fn get_meta_yaml(&self, id: i64, */

    pub fn get_meta_idea(&self, id: i64, meta_type: &str) -> Result<Option<Idea>> {
        let idea = self.get_idea(id)?;

        // First check if this idea has a .{meta_type} child
        for child_id in idea.child_ids {
            let child_idea = self.get_idea(child_id)?;
            if child_idea.name == format!(".{}", meta_type) {
                return Ok(Some(child_idea));
            }
        }

        // If it doesn't, check if its parent does (all the way back up the tree)
        if let Some(parent_id) = idea.parent_id {
            self.get_meta_idea(parent_id, meta_type)
        } else {
            Ok(None)
        }
    }

    pub fn get_meta_tags(&self, id: i64, meta_type: &str) -> Result<Vec<String>> {
        let idea = self.get_meta_idea(id, meta_type)?;

        if let Some(idea) = idea {
            Ok(idea.tags)
        }
        // It's not an error if there is no meta idea with that name,
        // just return an empty tag list
        else {
            Ok(Vec::new())
        }
    }

    pub fn get_child_ids(&self, id: i64, include_hidden: bool) -> Result<Vec<i64>> {
        let child_ids_yaml: String = self.conn.query_row("SELECT child_ids FROM ideas WHERE id=?", &[&id], |row| { row.get(0) })?;
        let mut child_ids: Vec<i64> = id_vec_from_yaml(&child_ids_yaml);

        let ignore_tags = self.get_meta_tags(id, "ignore")?;

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
        let idea = self.conn.query_row("SELECT * FROM ideas WHERE id=?", &[&id], |row| {
            idea_from_row(row)
        })?;
        Ok(idea)
    }

    pub fn search_ideas(&self, query_pattern: &String) -> Result<Vec<Idea>> {
        let full_pattern = format!("%{}%", query_pattern);
        let mut stmt = self.conn.prepare("SELECT * FROM ideas WHERE name||tags||description LIKE ?")?;
        let matches = stmt.query_map(&[&full_pattern], |row| {
            idea_from_row(row)
        })?;
        
        let mut results = Vec::new();
        for m in matches {
            results.push(m?);
        }
        Ok(results)
    }
}

fn idea_from_row(row: &Row) -> Idea {
    let tags = tag_vec_from_yaml(row.get::<i32, String>(3).as_str()) ;
    let child_ids = id_vec_from_yaml(row.get::<i32, String>(5).as_str());

    Idea {
        id: row.get(0),
        name: row.get(1),
        description: row.get(2),
        tags,
        parent_id: match row.get(4) {
            Value::Null => None,
            Value::Integer(id) => Some(id),
            _ => panic!("The parent_id row of an Idea in the database is neither Null nor an ID!")
        },
        child_ids,
    }
}
