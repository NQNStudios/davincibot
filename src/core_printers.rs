use std::collections::HashMap;
use std::str::from_utf8;

use repl::IdeaPrinter;
use idea::{Idea, IdeaTree};
use error::{Result, Error};
use conv::prelude::*;

const PARTS: usize = 30;

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


        let mut count = todo_idea.child_ids.len();
        // exclude archived, paused, etc. Ideas from count
        let ignore_tags = tree.get_meta_tags(todo_idea.id, "ignore")?;

        for child_id in &todo_idea.child_ids {
            let child_tags = tree.get_tags(*child_id, false)?;

            for tag in &ignore_tags {
                if tag != "done" && child_tags.contains(&tag) {
                    count -= 1;
                }
            }
        }


        match count.value_as::<f64>() {
            Ok(count) => Ok(sum / count),
            Err(_) => Err(Error::DaVinci(format!("Too many children of Idea #{} for floating point calculation.", todo_idea.id))),
        }
    }
}

fn print_progress_bar(todo_idea: &Idea, tree: &IdeaTree) -> Result<()> {
    // [======>                       ]
    let progress = progress(todo_idea, tree)?;

    print!("[");

    let mut bar = String::new();

    let current_part = (progress * PARTS as f64).floor() as usize;
    for part in 0..PARTS {
        if part < current_part {
            bar.push('=');
        }
        else if part == current_part {
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
            bytes[PARTS-1] = '=' as u8;

            let start = bytes.len() / 2 - 2;

            bytes[start] = 'D' as u8;
            bytes[start+1] = 'O' as u8;
            bytes[start+2] = 'N' as u8;
            bytes[start+3] = 'E' as u8;
            bytes[start+4] = '!' as u8;

            from_utf8(&bytes)?.to_string()
        };
    }

    print!("{}", bar);

    println!("]");

    // TODO print a ___/___ number

    Ok(())
}
