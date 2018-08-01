use std::collections::HashMap;
use std::str::from_utf8;

use repl::IdeaPrinter;
use idea::{Idea, IdeaTree};
use error::{Result, Error};
use conv::prelude::*;

pub fn core_printers() -> HashMap<String, IdeaPrinter> {
    let mut printers = HashMap::new();

    // The progress bar printer is inherited by default
    printers.insert("todo".to_string(), IdeaPrinter::new(true, print_progress_bar));

    printers
}

fn progress(todo_idea: &Idea, tree: &IdeaTree) -> Result<f64> {
    // If no children are present, return has_tag(done)
    if todo_idea.child_ids.len() == 0 {
        if todo_idea.tags.contains(&"done".to_string()) {
            Ok(1f64)
        } else {
            Ok(0f64)
        }
    }
    // Otherwise return the average of children's progress (recursively)
    else {
        let mut sum = 0f64;

        for child_id in &todo_idea.child_ids {
            sum += progress(&tree.get_idea(*child_id)?, tree)?;
        }

        // TODO exclude archived, paused, etc. Ideas from count

        match todo_idea.child_ids.len().value_as::<f64>() {
            Ok(count) => Ok(sum / count),
            Err(_) => Err(Error::DaVinci(format!("Too many children of Idea #{} for floating point calculation.", todo_idea.id))),
        }
    }
}

fn print_progress_bar(todo_idea: &Idea, tree: &IdeaTree) -> Result<()> {
    // [======>       ]
    let progress = progress(todo_idea, tree)?;

    print!("[");

    let mut bar = String::new();

    let current_tenth = (progress * 10f64).ceil() as i8;
    for tenth in 1..10 {
        if tenth < current_tenth {
            bar.push('=');
        }
        else if tenth == current_tenth {
            bar.push('>');
        }
        else {
            bar.push(' ');
        }
    }

    // If the Idea is fully complete, replace the last > with = and add DONE!
    if progress == 1f64 {
        bar = {
            let mut bytes = bar.into_bytes();
            bytes[9] = '=' as u8;

            bytes[3] = 'D' as u8;
            bytes[4] = 'O' as u8;
            bytes[5] = 'N' as u8;
            bytes[6] = 'E' as u8;
            bytes[7] = '!' as u8;

            from_utf8(&bytes)?.to_string()
        };
    }

    print!("{}", bar);

    println!("]");

    Ok(())
}
