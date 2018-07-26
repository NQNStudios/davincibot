use repl::*;
use error::*;
use idea::IdeaTree;
use std::collections::HashMap;

pub fn core_commands() -> HashMap<String, HandlerList> {
    let mut commands = HashMap::new();

    {
        commands.insert("print".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, print)],
        });
    }

    commands
}

fn select(repl: &mut Repl, tree: &IdeaTree, args: Vec<String>) -> Result<()> {
    repl.selected_id = repl.select_from_expression(tree, &args[0])?;

    Ok(())
}

fn move_multiple(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let parent_id = repl.select_from_expression(tree, &Repl::prompt_for_args(vec![" destination?"])?[0])?;
    Repl::prompt(" idea to move: ", |select_expression| {
        let id_to_move = repl.select_from_expression(tree, select_expression)?;
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

    let id_to_move = repl.select_from_expression(tree, parts[0])?;
    let parent_id = repl.select_from_expression(tree, parts[1])?;

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
fn print(repl: &mut Repl, tree: &mut IdeaTree, _args: Vec<String>) -> Result<()> {
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
    Repl::prompt(" ->", |name: &str| {
        tree.create_idea(repl.selected_id, Some([Some(&name), None, None, None]))?;
        Ok(true)
    });

    Ok(())
}
