use std::io::Write;
use std::io;
use std::collections::HashMap;

use idea::IdeaTree;
use error::{Result, Error};

pub enum CommandArgs {
    Zero,
    Amount(usize),
    Range { min: usize, max: usize },
}

impl CommandArgs {
    fn matches(&self, num_args: usize) -> bool {
        match self {
            CommandArgs::Zero => num_args == 0,
            CommandArgs::Amount(n) => num_args == *n,
            CommandArgs::Range { min, max } => *min <= num_args && num_args <= *max,
        }
    }
}

type CommandImplementation = Fn(&mut Repl, &mut IdeaTree, Vec<String>) -> Result<()>;

pub struct CommandHandler(CommandArgs, Box<CommandImplementation>);

impl CommandHandler {
    pub fn new<C>(args: CommandArgs, implementation: C) -> Self
        where C: 'static + Fn(&mut Repl, &mut IdeaTree, Vec<String>) -> Result<()>
    {
        CommandHandler (args, Box::new(implementation))
    }
}

pub struct HandlerList {
    pub delimiter: Option<String>,
    pub handlers: Vec<CommandHandler>,
    // TODO this might be enough information to print usage info on any
    // malformed command
}

pub struct Repl {
    pub selected_id: i64,
}

impl Repl {
    pub fn new() -> Repl {
        Repl { 
            selected_id: 1,
        }
    }

    // Allow the user to keep entering values for a prompt as many times
    // as they want until they type "exit"
    pub fn prompt<C>(prefix: &str, mut callback: C)
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

    pub fn prompt_for_args(arg_names: Vec<&str>) -> Result<Vec<String>> {
        let mut arg_values = Vec::new();
        for arg_name in &arg_names {
            Repl::prompt(&format!("{}", arg_name), |arg_value| {
                arg_values.push(arg_value.to_string());
                Ok(false)
            });
        }

        if arg_values.len() != arg_names.len() {
            return Err(Error::DaVinci(format!("User didn't supply all values for {:?}", arg_names)));
        }

        Ok(arg_values)
    }


    pub fn run(&mut self, tree: &mut IdeaTree, commands: HashMap<String, HandlerList>) {
    
        Repl::prompt("$", |input_line| {
            // An empty query is a no-op
            if input_line.len() == 0 {
                return Ok(true)
            }

            // The first token of every input line should be a valid command name
            let mut parts = input_line.splitn(2, " ");
            let command = parts.next().unwrap();

            if commands.contains_key(command) {

                let handler_list = &commands[command];
                let args: Vec<String> = {
                    match parts.next() {
                        Some(inputs) => match inputs.len() {
                            0 => Vec::new(),
                            _ => match handler_list.delimiter {
                                Some(ref delimiter) => inputs.split(delimiter.as_str()).map(|arg| arg.trim().to_string()).collect(),
                                None => vec![inputs.to_string()]
                            }
                        },
                        None => Vec::new()
                    }
                };

                // Check which of this command's handlers matches the number of
                // given inputs
                let mut handler = None;
                {
                    let handler_list = &commands[command];
                    for possible_handler in &handler_list.handlers {
                        if possible_handler.0.matches((&args).len()) {
                            handler = Some(&possible_handler.1);
                            break;
                        }
                    }
                }

                match handler {
                    Some(handler) => {
                        if let Err(e) = handler(self, tree, args) {
                            println!("'{}' command returned an error: {:?}", command, e);
                        }
                    },
                    None => println!("Can't call '{}' command with {} arguments", command, args.len()),
                }
            }
            else {
                println!("There is no Da Vinci Bot command named {}", command);

                return Ok(true);
            }

            Ok(true)
        });
    }

    /*fn handle_command(&mut self, tree: &mut IdeaTree, command: &str, input: Option<&str>) -> Result<()> {*/
        /*match (command, input) {*/
            /*("print", None) => print(self, tree),*/
            /*("list", Some("all")) => list(self, tree, true),*/
            /*("list", None) => list(self, tree, false),*/
            /*("select", Some(expression)) => select(self, tree, expression),*/
            /*("up", None) => self.handle_command(tree, "select", Some("^")),*/
            /*("root", None) => self.handle_command(tree, "select", Some("@")),*/
            /*("add", None) => add_multiple(self, tree),*/
            /*("add", Some(name)) => add(self, tree, name),*/
            /*("tag", Some(tags)) => tag(self, tree, tags),*/
            /*("untag", Some(tags)) => untag(self, tree, tags),*/
            /*("cleartags", None) => cleartags(self, tree),*/
            /*("move", None) => move_multiple(self, tree),*/
            /*("move", Some(inputs)) => move_one(self, tree, inputs),*/
            /*(c, i) => Err(Error::DaVinci(format!("Bad Da Vinci command: {} {:?}", c, i))),*/
        /*}*/
    /*}*/

    fn select_from_expression_internal(selected_id: i64, tree: &IdeaTree, expression: &str) -> Result<i64> {
        match expression {
            // @ is the operator for selecting the root Idea
            "@" => Ok(1),
            // ^ is the operator for selecting the parent Idea
            "^" => Ok(tree.get_idea(selected_id)?.parent_id.unwrap_or(1)),
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
                        let child_ids = tree.get_child_ids(selected_id, false)?;
                        if child_ids.len() < child_index {
                            return Err(Error::DaVinci(format!("Tried to select child {} from an Idea that only has {} children", child_index, child_ids.len())));
                        }

                        Ok(child_ids[child_index-1])
                    },
                    _ => {
                        let child_ids = tree.get_child_ids(selected_id, true)?;
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

    pub fn select_from_expression(&self, tree: &IdeaTree, expression: &str) -> Result<i64> {
        let mut temp_selected = self.selected_id;
        for part in expression.split_terminator('/') {
            temp_selected = Repl::select_from_expression_internal(temp_selected, tree, part)?;
        }
        Ok(temp_selected)
    }
}
