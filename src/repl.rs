use std::io::Write;
use std::io;
use std::collections::HashMap;

use idea::IdeaTree;
use error::{Result, Error};

enum CommandArgs {
    Zero,
    Amount(n: usize),
    Range { min: usize, max: usize }
}

impl CommandArgs {
    fn matches(&self, num_args: usize): bool {
        match self {
            Zero => num_args == 0,
            Amount(n) => num_args == n,
            Range { min, max } => min <= num_args && num_args <= max,
        }
    }
}

type CommandImplementation = FnMut(Vec<String>) -> Result<()>;

struct CommandHandler = (CommandArgs, Box<CommandImplementation>);

struct HandlerList {
    delimiter: Option<String>,
    handlers: Vec<CommandHandler>,
    // TODO this might be enough information to print usage info on any
    // malformed command
}

pub struct Repl {
    selected_id: i64,
    commands: HashMap<String, HandlerList>,
    delimiters: Vec<String>,
}

impl Repl {
    pub fn new() -> Repl {
        let mut repl = Repl { 
            selected_id: 1,
            commands: HashMap::new(),
            delimiters: vec!["/", "->", " "],
        };

        // TODO register the core commands
    }

    fn register_core_command(&mut self, key: &str, delimiter: &str) {

    }

    pub fn register_command(&mut self, tree: &IdeaTree, key: &str, delimiter: Option<String>) -> Result<()> {
        if let Some(d) = delimiter {

        }

        self.commands[key.to_string()] = HandlerList {
            delimiter: delimiter,
            handlers: vec![],
        };
    }

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

    fn prompt_for_args(arg_names: Vec<&str>) -> Result<Vec<String>> {
        let mut arg_values = Vec::new();
        for arg_name in &arg_names {
            prompt(&format!("{}", arg_name), |arg_value| {
                arg_values.push(arg_value.to_string());
                Ok(false)
            });
        }

        if arg_values.len() != arg_names.len() {
            return Err(Error::DaVinci(format!("User didn't supply all values for {:?}", arg_names)));
        }

        Ok(arg_values)
    }


    pub fn run(&mut self, tree: &mut IdeaTree) {
        prompt("$", |input_line| {
            // An empty query is a no-op
            if input_line.len() == 0 {
                return Ok(true)
            }

            // The first token of every input line should be a valid command name
            let mut parts = input_line.splitn(2, " ");
            let command = parts.next().unwrap();

            if self.commands.contains(command) {
                let handler_list = commands[command];

                let args = match parts.next() {
                    Some(inputs) => match inputs.len() {
                        0 => Vec::new(),
                        _ => match handler_list.delimiter {
                            Some(delimiter) => inputs.split(delimiter).map(|arg| arg.trim()).collect(),
                            None => vec![inputs]
                        }
                    },
                    None => Vec::new()
                };

                // Check which of this command's handlers matches the number of
                // given inputs
                for (handler, implementation) in handler_list {
                    if handler.args.matches(args.len()) {
                        if let Err(e) = implementation(args) {
                            println!("'{}' command returned an error: {:?}", command, e);
                        }
                    }
                }

            }
            else {
                println!("There is no Da Vinci Bot command named {}", command);

                Ok(true)
            }

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
            ("move", None) => move_multiple(self, tree),
            ("move", Some(inputs)) => move_one(self, tree, inputs),
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
                    if child_ids.len() < child_index {
                        return Err(Error::DaVinci(format!("Tried to select child {} from an Idea that only has {} children", child_index, child_ids.len())));
                    }

                    Ok(child_ids[child_index-1])
                },
                _ => {
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

fn move_multiple(repl: &Repl, tree: &mut IdeaTree) -> Result<()> {
    let parent_id = select_from_expression(repl, tree, &prompt_for_args(vec!["destination?"])?[0])?;
    prompt("idea to move: ", |select_expression| {
        let id_to_move = select_from_expression(repl, tree, select_expression)?;
        tree.set_parent(id_to_move, parent_id)?;
        Ok(true)
    });

    Ok(())
}

fn move_one(repl: &Repl, tree: &mut IdeaTree, inputs: &str) -> Result<()> {
    let parts: Vec<&str> = inputs.split("->").map(|part| part.trim()).collect();
    if parts.len() != 2 {
        return Err(Error::DaVinci("'move' can either be called with no arguments, or with 2 separated by '->'".to_string()));
    }

    let id_to_move = select_from_expression(repl, tree, parts[0])?;
    let parent_id = select_from_expression(repl, tree, parts[1])?;

    tree.set_parent(id_to_move, parent_id);

    Ok(())
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
