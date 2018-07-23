use std::io::prelude::*;
use std::io::Write;
use std::io;
use idea::IdeaTree;
use error::Result;

pub struct Repl {
    selected_id: i64,
    current_command: String,
}

impl Repl {
    pub fn new() -> Repl {
        Repl { 
            selected_id: 1,
            current_command: "".to_string()
        }
    }
    // Allow the user to keep entering values for a prompt as many times
    // as they want until they type "exit"
    fn prompt<C>(prefix: String, callback: C)
        where C: Fn(String) -> Result<()>
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
                        callback(input.trim().to_string());
                    }
                }
                Err(e) => continue,
            };
        }
    }

    pub fn run(&mut self, tree: &mut IdeaTree) {
        Self::prompt("$".to_string(), |command| {
            println!("{}", command);
            Ok(())
        });
    }
}
