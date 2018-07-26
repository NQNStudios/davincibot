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
        commands.insert("listall".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, |repl, tree, args| list(repl, tree, true))],
        });
        commands.insert("list".to_string(), HandlerList {
            delimiter: Some(" ".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, |repl, tree, args| list(repl, tree, false)),
                CommandHandler::new(CommandArgs::VarArgs, |repl, tree, args| list_with_tags(repl, tree, args)),
            ],
        });
        commands.insert("select".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(1), select),
            ],
        });
        commands.insert("up".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, |repl, tree, args| select(repl, tree, vec!["^".to_string()])),
            ],
        });
        commands.insert("root".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, |repl, tree, args| select(repl, tree, vec!["@".to_string()])),
            ],
        });
        commands.insert("add".to_string(), HandlerList {
            delimiter: Some(",".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, add_multiple),
                CommandHandler::new(CommandArgs::Minimum(1), add),
            ],
        });
        commands.insert("tag".to_string(), HandlerList {
            delimiter: Some(" ".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, tag_multiple),
                CommandHandler::new(CommandArgs::Minimum(1), tag),
            ],
        });
        commands.insert("untag".to_string(), HandlerList {
            delimiter: Some(" ".to_string()),
            handlers: vec![CommandHandler::new(CommandArgs::Minimum(1), untag)],
            // TODO untag multiple?
        });
        commands.insert("cleartags".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, cleartags)],
            // TODO cleartags multiple? (although, cleartags already takes zero
            // arguments so the disambiguation would be weird)
        });
            /*("tag", Some(tags)) => tag(self, tree, tags),*/
            /*("untag", Some(tags)) => untag(self, tree, tags),*/
            /*("cleartags", None) => cleartags(self, tree),*/
            /*("move", None) => move_multiple(self, tree),*/
            /*("move", Some(inputs)) => move_one(self, tree, inputs),*/
        // TODO describe!
        // TODO rename!
    }

    commands
}

fn select(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
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

    tree.set_parent(id_to_move, parent_id)
}

fn tag(repl: &mut Repl, tree: &mut IdeaTree, tags: Vec<String>) -> Result<()> {
    tree.add_tags(repl.selected_id, tags)
}

fn tag_multiple(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let tags: Vec<String> = Repl::prompt_for_args(vec![" tags?"])?[0].split(" ").map(|tag| tag.to_string()).collect();

    // Collect all the ids to tag without applying any,
    // because applying a hide tag to one will change the child indices of the
    // others.
    let mut ids_to_tag = Vec::new();
    Repl::prompt(" tag idea ->", |select_expression| {
        ids_to_tag.push(repl.select_from_expression(tree, select_expression)?);
        Ok(true)
    });
    for id_to_tag in ids_to_tag {
        tree.add_tags(id_to_tag, tags.clone())?;
    }

    Ok(())
}

fn untag(repl: &mut Repl, tree: &mut IdeaTree, tags: Vec<String>) -> Result<()> {
    tree.remove_tags(repl.selected_id, tags)
}

fn cleartags(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
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
    let shown_child_ids = tree.get_child_ids(repl.selected_id, false)?;
    let all_child_ids = tree.get_child_ids(repl.selected_id, true)?;

    for (child_idx, id) in shown_child_ids.iter().enumerate() {
        let child_name = tree.get_name(*id)?;

        println!("{}. {}", child_idx+1, child_name);
    }

    if show_all {
        let mut hidden_child_ids = tree.get_child_ids(repl.selected_id, true)?;
        hidden_child_ids.retain(|id| !shown_child_ids.contains(id));

        for id in hidden_child_ids {
            let child_name = tree.get_name(id)?;

            println!("Hidden: {}", child_name);
        }
    }

    Ok(())
}

fn list_with_tags(repl: &Repl, tree: &IdeaTree, tags: Vec<String>) -> Result<()> {
    // TODO only list children that have the right tags (including hidden ones)

    Ok(())
}

fn add(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    for arg in args {
        if arg != "" {
            tree.create_idea(repl.selected_id, arg, None)?;
        }
    }
    Ok(())
}

fn add_multiple(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    Repl::prompt(" new idea ->", |name: &str| {
        tree.create_idea(repl.selected_id, name.to_string(), None)?;
        Ok(true)
    });

    Ok(())
}
