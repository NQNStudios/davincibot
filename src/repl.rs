use std::io::Write;
use std::io;
use std::collections::HashMap;
use std::rc::Rc;

use idea::{IdeaTree, Idea};
use error::{Result, Error};

use core_commands::core_commands;
use core_printers::core_printers;

pub enum CommandArgs {
    Zero,
    Amount(usize),
    Minimum(usize),
    Maximum(usize),
    Range { min: usize, max: usize },
    VarArgs,
}

impl CommandArgs {
    fn matches(&self, num_args: usize) -> bool {
        match self {
            CommandArgs::Zero => num_args == 0,
            CommandArgs::Amount(n) => num_args == *n,
            CommandArgs::Range { min, max } => *min <= num_args && num_args <= *max,
            CommandArgs::Minimum(min) => *min <= num_args,
            CommandArgs::Maximum(max) => *max >= num_args,

            CommandArgs::VarArgs => true,
        }
    }
}

// TODO this won't be pub after printing is moved out of core_commands.rs into repl.rs
pub type PrinterImplementation = Fn(&Idea, &IdeaTree) -> Result<()>;

pub struct IdeaPrinter {
    pub always_inherited: bool, 
    pub implementation: Box<PrinterImplementation>,
}

impl IdeaPrinter {
    pub fn new<C>(always_inherited: bool, implementation: C) -> Self
        where C: 'static + Fn(&Idea, &IdeaTree) -> Result<()>
    {
        IdeaPrinter {
            always_inherited,
            implementation: Box::new(implementation),
        }
    }
}



type CommandImplementation = Fn(&mut Repl, &mut IdeaTree, Vec<String>) -> Result<()>;

pub struct CommandHandler(CommandArgs, Rc<CommandImplementation>);

impl CommandHandler {
    pub fn new<C>(args: CommandArgs, implementation: C) -> Self
        where C: 'static + Fn(&mut Repl, &mut IdeaTree, Vec<String>) -> Result<()>
    {
        CommandHandler (args, Rc::new(implementation))
    }
}

pub struct HandlerList {
    pub delimiter: Option<String>,
    pub handlers: Vec<CommandHandler>,
}

pub struct Repl {
    pub selected_id: i64,
    commands: HashMap<String, HandlerList>,
    pub printers: HashMap<String, IdeaPrinter>,
}

impl Repl {
    pub fn new() -> Repl {
        let mut repl = Repl { 
            selected_id: 1,
            commands: HashMap::new(),
            printers: HashMap::new(),
        };

        repl.register_commands(core_commands());
        repl.register_printers(core_printers());
        repl
    }

    // add all the commands to this Repl's command map, and throw an
    // error if any of them are a duplicate command name
    pub fn register_commands(&mut self, commands: HashMap<String, HandlerList>) {
        for (command, handler_list) in commands {
            if self.commands.contains_key(&command) {
                println!("Error! Cannot add duplicate command with name '{}'", command);
            } else {
                self.commands.insert(command, handler_list);
            }
        }
    }

    pub fn register_printers(&mut self, printers: HashMap<String, IdeaPrinter>) {
        for (idea_type, printer) in printers {
            if self.printers.contains_key(&idea_type) {
                println!("Error! Cannot add duplicate printer for type '{}'", idea_type);
            } else {
                self.printers.insert(idea_type, printer);
            }
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
            Repl::prompt(&format!(" {}", arg_name), |arg_value| {
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
        self.run_command(tree, "select @".to_string());
        Repl::prompt("$", |input_line| { self.run_command(tree, input_line.to_string()); Ok(true) });
        // TODO confirm quitting Davincibot
    }

    // TODO Idea printing should be a function in this file like run_command,
    // not defined in core_commands.rs as it is

    pub fn run_command(&mut self, tree: &mut IdeaTree, input_line: String) {
        // An empty query is a no-op
        if input_line.len() == 0 {
            return;
        }

        // The first token of every input line should be a valid command name
        let mut parts = input_line.splitn(2, " ");
        let command = parts.next().unwrap();

        if self.commands.contains_key(command) {

            let args;
            let mut handler: Option<Rc<CommandImplementation>> = None;

            {
                let handler_list = &self.commands[command];
                args = match parts.next() {
                    Some(inputs) => match inputs.len() {
                        0 => Vec::new(),
                        _ => match handler_list.delimiter {
                            Some(ref delimiter) => inputs.split(delimiter.as_str()).map(|arg| arg.trim().to_string()).collect(),
                            None => vec![inputs.to_string()]
                        }
                    },
                    None => Vec::new()
                };
                // Check which of this command's handlers matches the number of
                // given inputs
                for possible_handler in &handler_list.handlers {
                    if possible_handler.0.matches((&args).len()) {
                        handler = Some(Rc::clone(&possible_handler.1));
                        break;
                    }
                }
            }

            match handler {
                Some(handler) => {
                    if let Err(e) = (*handler)(self, tree, args) {
                        println!("'{}' command returned an error: {:?}", command, e);
                    }
                },
                None => match &self.commands[command].delimiter {
                    Some(delimiter) => println!("Can't call '{}' command with {} arguments", command, args.len()),
                    None => println!("The '{}' command does not take arguments", command),
                }
            }
        }
        else {
            println!("There is no Da Vinci Bot command named {}", command);
        }
    }

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
