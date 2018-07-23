use std::io::Write;
use std::io;
use std::collections::HashMap;

use idea::IdeaTree;
use error::{Result, Error};

// Allow the user to keep entering values for a prompt as many times
// as they want until they type "exit"
fn prompt<C>(prefix: &str, mut callback: C)
    where C: FnMut(&str) -> Result<()>
{
    loop {
        print!("{} ", prefix);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input == "exit\n" {
                    break;
                }
                else {
                    if let Err(e) = callback(&mut input.trim()) {
                        println!("Error processing console input: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error getting console input: {:?}", e);
                continue
            },
        };
    }
}

pub struct Repl {
    selected_id: i64,
}

impl Repl {
    pub fn new() -> Repl {
        Repl { 
            selected_id: 1,
        }
    }

    pub fn run(&mut self, tree: &mut IdeaTree) {
        prompt("$", |input_line| {
            let mut parts = input_line.splitn(2, " ");
            // TODO once the try_trait feature and ? for Options becomes
            // stable, this next line
            // should be replaced with:
            /*
            let command = parts.next()?;
            */
            let command = parts.next().unwrap();
            let inputs = parts.next();

            // The / operator chains a command on multiple inputs given on
            // the same line
            if let Some(inputs_present) = inputs {
                for input in inputs_present.split("/") {
                    self.handle_command(tree, command, Some(input))?;
                }
            }
            else {
                self.handle_command(tree, command, None)?;
            }
            Ok(())
        });
    }

    fn handle_command(&mut self, tree: &mut IdeaTree, command: &str, input: Option<&str>) -> Result<()> {
        match (command, input) {
            ("print", None) => print(self, tree),
            ("list", Some("all")) => list(self, tree, true),
            ("list", None) => list(self, tree, false),
            ("add", None) => add_multiple(self, tree),
            ("add", Some(name)) => add(self, tree, name),
            ("tag", Some(tags)) => tag(self, tree, tags),
            ("untag", Some(tags)) => untag(self, tree, tags),
            ("cleartags", None) => cleartags(self, tree),
            (c, i) => Err(Error::DaVinci(format!("Bad Da Vinci command: {} {:?}", c, i))),
        }
    }
}

fn tag(repl: &Repl, tree: &mut IdeaTree, tags: &str) -> Result<()> {
    tree.add_tags(repl.selected_id, tags.split(" ").map(|str_slice| str_slice.to_string()).collect())
}

fn untag(repl: &Repl, tree: &mut IdeaTree, tags: &str) -> Result<()> {
    tree.remove_tags(repl.selected_id, tags.split(" ").map(|str_slice| str_slice.to_string()).collect())
}

fn cleartags(repl: &Repl, tree: &mut IdeaTree) -> Result<()> {
    tree.clear_tags(repl.selected_id)
}

// TODO printing ideas should be prettier
fn print(repl: &Repl, tree: &IdeaTree) -> Result<()> {
    let idea = tree.get_idea(repl.selected_id)?;

    println!("#{}: {}", idea.id, idea.name);
    if idea.tags.len() == 0 {
        println!("[No tags]");
    }
    else {
        for tag in &idea.tags {
            print!("[{}] ", tag);
        }
        println!();
    }
    println!("---");
    println!("{}", idea.description);
    println!("{} children", idea.child_ids.len()); // TODO print how many are hidden

    Ok(())
}

fn list(repl: &Repl, tree: &IdeaTree, show_all: bool) -> Result<()> {
    let child_ids = tree.get_child_ids(repl.selected_id)?;

    for (child_idx, id) in child_ids.into_iter().enumerate() {
        let child_name = tree.get_name(id)?;
        println!("{}. {}", child_idx + 1, child_name);
    }

    Ok(())
}

fn add(repl: &Repl, tree: &mut IdeaTree, name: &str) -> Result<()> {
    tree.create_idea(repl.selected_id, Some([Some(&name), None, None, None]))?;
    Ok(())
}

fn add_multiple(repl: &mut Repl, tree: &mut IdeaTree) -> Result<()> {
    prompt("->", |name: &str| {
        tree.create_idea(repl.selected_id, Some([Some(&name), None, None, None]))?;
        Ok(())
    });

    Ok(())
}
