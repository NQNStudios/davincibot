use std::io::Write;
use std::io;
use std::collections::HashMap;

use idea::IdeaTree;
use error::{Result, Error};

// Allow the user to keep entering values for a prompt as many times
// as they want until they type "exit"
fn prompt<C>(prefix: &str, mut callback: C)
    where C: FnMut(&str) -> Result<bool>
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
                    match callback(&mut input.trim()) {
                        Ok(true) => { },
                        Ok(false) => break,
                        Err(e) => println!("Error processing console input: {:?}", e),
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

fn prompt_for_args(arg_names: Vec<&str>) -> Vec<String> {
    let mut arg_values = Vec::new();
    for arg_name in arg_names {
        prompt(&format!("{}:", arg_name), |arg_value| {
            arg_values.push(arg_value.to_string());
            Ok(false)
        });
    }

    arg_values
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

            self.handle_command(tree, command, inputs)?;

            Ok(true)
        });
    }

    fn handle_command(&mut self, tree: &mut IdeaTree, command: &str, input: Option<&str>) -> Result<()> {
        match (command, input) {
            ("print", None) => print(self, tree),
            ("list", Some("all")) => list(self, tree, true),
            ("list", None) => list(self, tree, false),
            ("select", Some(expression)) => select(self, tree, expression),
            ("up", None) => self.handle_command(tree, "select", Some("^")),
            ("root", None) => self.handle_command(tree, "select", Some("@")),
            ("add", None) => add_multiple(self, tree),
            ("add", Some(name)) => add(self, tree, name),
            ("tag", Some(tags)) => tag(self, tree, tags),
            ("untag", Some(tags)) => untag(self, tree, tags),
            ("cleartags", None) => cleartags(self, tree),
            ("move", None) => move_idea(self, tree),
            ("move", Some(_)) => Err(Error::DaVinci("Hint: Try calling 'move' without any arguments.".to_string())),
            (c, i) => Err(Error::DaVinci(format!("Bad Da Vinci command: {} {:?}", c, i))),
        }
    }
}

fn select_from_expression(repl: &Repl, tree: &IdeaTree, expression: &str) -> Result<i64> {
    if expression.contains('/') {
        let mut temp_selected = repl.selected_id;
        for part in expression.split_terminator('/') {
            temp_selected = select_from_expression(&Repl { selected_id: temp_selected }, tree, part)?;
        }
        return Ok(temp_selected);
    }

    match expression {
        // @ is the operator for selecting the root Idea
        "@" => Ok(1),
        // ^ is the operator for selecting the parent Idea
        "^" => Ok(tree.get_idea(repl.selected_id)?.parent_id.unwrap_or(1)),
        text => {
            let first_char = {
                text.chars().next().unwrap()
            };
            match first_char {
                '#' => {
                    let absolute_id: String = text.chars().skip(1).collect();
                    return Ok(absolute_id.parse::<i64>()?);
                },
                '0'...'9' => {
                    let child_index = text.parse::<usize>()?;
                    let child_ids = tree.get_child_ids(repl.selected_id, false)?;

                    Ok(child_ids[child_index-1])
                },// TODO parse index,
                other => {
                    let child_ids = tree.get_child_ids(repl.selected_id, true)?;
                    let mut selected_id: Option<i64> = None;

                    for child_id in child_ids {
                        if tree.get_name(child_id)? == text {
                            selected_id = Some(child_id);
                        }
                    }

                    if selected_id == None {
                        return Err(Error::DaVinci(format!("Selected Idea has no child named \"{}\"", text)));
                    }

                    Ok(selected_id.unwrap())
                }
            }
        }
    }
}

fn select(repl: &mut Repl, tree: &IdeaTree, expression: &str) -> Result<()> {
    repl.selected_id = select_from_expression(repl, tree, expression)?;

    Ok(())
}

fn move_idea(repl: &Repl, tree: &mut IdeaTree) -> Result<()> {
    let args = prompt_for_args(vec!["idea to move", "destination"]);
    let id_to_move = select_from_expression(repl, tree, args[0].as_str())?;
    let new_parent_id = select_from_expression(repl, tree, args[1].as_str())?;
    tree.set_parent(id_to_move, new_parent_id)
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

// TODO don't list hidden ones with a numeric index even when show_all is given
fn list(repl: &Repl, tree: &IdeaTree, show_all: bool) -> Result<()> {
    let child_ids = tree.get_child_ids(repl.selected_id, show_all)?;

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
        Ok(true)
    });

    Ok(())
}
