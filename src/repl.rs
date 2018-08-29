use std::collections::HashMap;
use std::rc::Rc;
use std::borrow::Borrow;
use std::process::exit;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use yaml_rust::Yaml;

use idea::{IdeaTree, Idea};
use error::{Result, Error};

use core_commands::core_commands;
use core_printers::core_printers;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

pub struct Command {
    pub description: &'static str,
    pub delimiter: Option<String>,
    pub handlers: Vec<CommandHandler>,
}

pub struct Repl {
    pub selected_id: i64,
    rl: Editor<()>,
    commands: HashMap<String, Command>,
    printers: HashMap<String, IdeaPrinter>,
}

impl Repl {
    pub fn new() -> Repl {
        let mut repl = Repl { 
            selected_id: 1,
            commands: HashMap::new(),
            printers: HashMap::new(),
            rl: Editor::<()>::new(),
        };
        // TODO set up rl history file
        // TODO add commands to history log

        repl.register_commands(core_commands());
        repl.register_printers(core_printers());
        repl
    }

    // add all the commands to this Repl's command map, and throw an
    // error if any of them are a duplicate command name
    pub fn register_commands(&mut self, commands: HashMap<String, Command>) {
        for (command, handler_list) in commands {
            if self.commands.contains_key(&command) {
                println!("Error! Cannot add duplicate command with name '{}'", command);
            } else if command.len() == 1 {
                println!("Error! Commands cannot have single-character names.");
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
    pub fn prompt<C>(&mut self, prefix: &str, mut callback: C, add_history: bool) -> Result<()>
        where C: FnMut(&mut Repl, &str) -> Result<bool>
    {
        loop {
            // TODO if add_history is false, don't allow up and down
            // to reuse lines from history, either.
            let input = self.rl.readline(&format!("{} ", prefix));
            match input {
                Ok(input) => {
                    if input == "exit" {
                        break;
                    }
                    else {
                        if add_history {
                            self.rl.add_history_entry(input.trim());
                        }

                        match callback(self, &mut input.trim()) {
                            Ok(true) => { },
                            Ok(false) => break,
                            Err(e) => println!("Error processing console input: {:?}", e),
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => break,
                Err(ReadlineError::Eof) => exit(0),
                Err(e) => {
                    println!("Error getting console input: {:?}", e);
                    continue
                },
            };
        }

        Ok(())
    }

    pub fn prompt_for_args(&mut self, arg_names: Vec<&str>) -> Result<Vec<String>> {
        let mut arg_values = Vec::new();
        for arg_name in &arg_names {
            self.prompt(&format!(" {}", arg_name), |ref _repl, arg_value| {
                arg_values.push(arg_value.to_string());
                Ok(false)
            }, false);
            // Don't save prompt_for_args input in command history
        }

        if arg_values.len() != arg_names.len() {
            return Err(Error::DaVinci(format!("User didn't supply all values for {:?}", arg_names)));
        }

        Ok(arg_values)
    }


    pub fn run(&mut self, tree: &mut IdeaTree) {
        self.run_command(tree, "select @".to_string());

        // Read
        self.prompt("$", |ref mut repl, input_line| {
            // Execute
            repl.run_command(tree, input_line.to_string());

            // Loop
            Ok(true)
        }, true); // Do save commands in the history file
    }

    // TODO this is a janky helper function that doesn't account for terminal width
    fn print_hr(&self) {
        println!("--------------");
    }

    // TODO Ideas should be printed in a prettier form somehow, with line
    // wrapping
    pub fn print(&self, tree: &IdeaTree) -> Result<()> {
        let idea = tree.get_idea(self.selected_id)?;

        let description_limit = match tree.get_meta_idea(self.selected_id, &"settings")? {
            Some(settings) => {
                if let Some(settings_yaml) = settings.get_yaml_data()? {
                    // TODO allow setting a maximum line count for the
                    // description output as well

                    match settings_yaml["max_description"] {
                        Yaml::BadValue => idea.description.len(),
                        Yaml::Integer(max) => max as usize,
                        _ => return Err(Error::DaVinci("max_description setting is not set to an integer!".to_string())),
                    }
                }
                else {
                    idea.description.len()
                }
            },
            None => idea.description.len(),
        };

        self.print_hr();
        // TODO check max_name and shorten name printing
        println!("#{}: {}", idea.id, idea.name);
        if idea.tags.len() > 0 {
            for tag in &idea.tags {
                print!("[{}] ", tag);
            }
            println!();
        }
        self.print_hr();

        if idea.description.len() > 0 {
            let description_to_print = if description_limit < idea.description.len() {
                // TODO this probably doesn't account for multibyte chars!
                format!("{}...", &idea.description[0..description_limit])
            } else {
                // TODO this clone() shouldn't be necessary
                idea.description.clone()
            };

            println!("{}", description_to_print);
            self.print_hr();
        }


        if idea.child_ids.len() > 0 {
            println!("{} children", idea.child_ids.len()); // TODO print how many are hidden
            // TODO and also truncate children
            self.print_hr();
        }

        // do special printing using registered Idea type printers
        for (idea_type, idea_printer) in &self.printers {
            let always_inherited = idea_printer.always_inherited;
            let printer_implementation: &PrinterImplementation = idea_printer.implementation.borrow();

            if tree.get_tags(self.selected_id, always_inherited)?.contains(&idea_type) {
                (*printer_implementation)(&idea, tree)?;
                self.print_hr();
            }
        }

        Ok(())
    }

    pub fn run_command(&mut self, tree: &mut IdeaTree, input_line: String) {
        // An empty query is a no-op
        if input_line.len() == 0 {
            return;
        }

        // The first token of every input line should be a valid command name
        let mut parts = input_line.splitn(2, " ");
        let mut command = parts.next().unwrap().to_string();

        if command.len() == 1 {
            command = match tree.get_meta_idea(self.selected_id, &"shortcuts").unwrap_or(None) {
                Some(shortcuts) => {
                    if let Some(shortcuts_yaml) = shortcuts.get_yaml_data().unwrap_or(None) {
                        match &shortcuts_yaml[command.as_str()] {
                            Yaml::BadValue => {
                                println!("Error! No command for shortcut '{}'", command);
                                return;
                            },
                            Yaml::String(shortcut_command) => {
                                shortcut_command.clone()
                            },
                            _ => {
                                println!("Error! Command for shortcut '{}' is not a string!", command);
                                return;
                            },
                        }
                    }
                    else {
                        println!("Error! The description of the shortcuts meta idea #{} is not properly formatted YAML", shortcuts.id);
                        return;
                    }
                },
                None => {
                    println!("Error! No command shortcuts are defined.");
                    return;
                }, 
            };
        }

        if self.commands.contains_key(&command) {

            let args;
            let mut handler: Option<Rc<CommandImplementation>> = None;

            {
                let handler_list = &self.commands[&command];
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
                None => println!("Can't call '{}' command with {} arguments", command, args.len()),
            }
        }
        else {
            println!("There is no Da Vinci Bot command named {}", command);
        }
    }

    // TODO find out how Git checks for commands with similar names to typos,
    // and maybe use that to provide suggestions when name-based select
    // expressions are mis-typed 
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
                    // A "#{integer}" select expression selects an Idea by its absolute ID.
                    '#' => {
                        let absolute_id: String = text.chars().skip(1).collect();
                        return Ok(absolute_id.parse::<i64>()?);
                    },
                    // A positive index will select a child ID from the start of the list
                    '0'...'9' => {
                        let child_index = text.parse::<usize>()?;
                        let child_ids = tree.get_child_ids(selected_id, false)?;

                        // Ensure that the child index is valid (non-zero, within range)
                        if child_index == 0 {
                            Err(Error::DaVinci(format!("The current Idea has no child at index {}", child_index)))
                        } else {
                            match child_ids.get(child_index-1) {
                                Some(&id) => Ok(id),
                                None => Err(Error::DaVinci(format!("The current Idea has no child at index {}", child_index)))
                            }
                        }
                    },
                    // A negative index will select a child ID from the end of the list backward
                    '-' => {
                        let child_reverse_index = (&text[1..]).parse::<usize>()?;
                        let child_ids = tree.get_child_ids(selected_id, false)?;

                        // Ensure that the child index is valid (non-zero, within range)
                        if child_reverse_index == 0 || child_reverse_index >= child_ids.len() {
                            Err(Error::DaVinci(format!("The current Idea has no child at index {}", text)))
                        } else {
                            match child_ids.get(child_ids.len()-child_reverse_index) {
                                Some(&id) => Ok(id),
                                None => Err(Error::DaVinci(format!("The current Idea has no child at index {}", text)))
                            }
                        }

                    },
                    _ => {
                        let selected_child: Idea = tree.get_child_by_name_hint(selected_id, text.to_string())?;

                        Ok(selected_child.id)
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

    pub fn print_help(&self) {
        // TODO print brackets around the first ocurrance of shortcut character
        for (command_name, command) in &self.commands {
            // TODO this should handle nice wrapping
            println!("{}: {}", command_name, command.description);
        }
    }

    pub fn print_command_help(&self, command: String) {
        // TODO print info on the behavior and different overloads of the given command
    }
}
