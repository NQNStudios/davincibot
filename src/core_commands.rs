use repl::*;
use error::*;
use idea::IdeaTree;
use std::collections::HashMap;

use edit_rs::get_input;

pub fn core_commands() -> HashMap<String, HandlerList> {
    let mut commands = HashMap::new();

    {
        commands.insert("print".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, |repl, tree, args| repl.print(tree))],
        });
        commands.insert("listall".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, |repl, tree, args| list(repl, tree, true))],
        });
        // TODO list needs to allow pagination
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
        commands.insert("move".to_string(), HandlerList {
            delimiter: Some("->".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(2), move_both_args),
                CommandHandler::new(CommandArgs::Amount(1), move_one_arg),
                CommandHandler::new(CommandArgs::Zero, move_multiple),
            ],
        });
        commands.insert("describe".to_string(), HandlerList {
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(1), describe),
                CommandHandler::new(CommandArgs::Zero, describe),
            ],
        });
        commands.insert("rename".to_string(), HandlerList {
            delimiter: Some("->".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(2), rename_any),
                CommandHandler::new(CommandArgs::Amount(1), rename_selected),
                CommandHandler::new(CommandArgs::Amount(0), rename_selected),
            ],
        });
        // TODO reordering children
        // TODO add n ideas
        // TODO pipe accidental git commands back to the shell, lol?
        // TODO ignore command that creates an .ignore child if necessary and
        // adds the args as tags. (Should it automatically overwrite inherited
        // ignore tags? i.e. equivalent to typing `noignore` first?)
        // TODO noignore command that adds an empty .ignore child, or clears
        // existing .ignore child's tags
        // TODO generic form of ignore command with syntax meta [meta_type]
        // [meta_tag] i.e meta ignore done
    }

    commands
}

fn select(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    repl.selected_id = repl.select_from_expression(tree, &args[0])?;
    repl.print(tree);

    Ok(())
}

fn move_multiple(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let parent_id = repl.select_from_expression(tree, &Repl::prompt_for_args(vec!["destination?"])?[0])?;
    Repl::prompt(" idea to move:", |select_expression| {
        let id_to_move = repl.select_from_expression(tree, &select_expression)?;
        tree.set_parent(id_to_move, parent_id)?;
        Ok(true)
    });

    Ok(())
}

fn move_one_arg(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let id_to_move = repl.select_from_expression(tree, args[0].as_str())?;
    let further_arg = Repl::prompt_for_args(vec!["desination?"])?;
    let parent_selector = further_arg[0].as_str();
    let parent_id = repl.select_from_expression(tree, parent_selector)?;
    tree.set_parent(id_to_move, parent_id)
}

fn move_both_args(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let id_to_move = repl.select_from_expression(tree, args[0].as_str())?;
    let parent_id = repl.select_from_expression(tree, args[1].as_str())?;

    tree.set_parent(id_to_move, parent_id)
}

fn tag(repl: &mut Repl, tree: &mut IdeaTree, tags: Vec<String>) -> Result<()> {
    tree.add_tags(repl.selected_id, tags)
}

fn tag_multiple(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let tags: Vec<String> = Repl::prompt_for_args(vec!["tags?"])?[0].split(" ").map(|tag| tag.to_string()).collect();

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

// TODO don't list hidden ones with a numeric index even when show_all is given
fn list(repl: &Repl, tree: &IdeaTree, show_all: bool) -> Result<()> {
    let shown_child_ids = tree.get_child_ids(repl.selected_id, false)?;
    let all_child_ids = tree.get_child_ids(repl.selected_id, true)?;

    for (child_idx, id) in shown_child_ids.iter().enumerate() {
        let child= tree.get_name_with_tags(*id)?;

        println!("{}. {}", child_idx+1, child);
    }

    if show_all {
        let mut hidden_child_ids = tree.get_child_ids(repl.selected_id, true)?;
        hidden_child_ids.retain(|id| !shown_child_ids.contains(id));

        for id in hidden_child_ids {
            let child = tree.get_name_with_tags(id)?;

            println!("Hidden: {}", child);
        }
    }

    Ok(())
}

// TODO this is now ambiguous because get_name_with_tags() is already what's 
// printed from `list`
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

fn add_multiple(repl: &mut Repl, tree: &mut IdeaTree, _args: Vec<String>) -> Result<()> {
    Repl::prompt(" new idea ->", |name: &str| {
        tree.create_idea(repl.selected_id, name.to_string(), None)?;
        Ok(true)
    });

    Ok(())
}

fn describe(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let target_id = match args.into_iter().next() {
        Some(expression) => repl.select_from_expression(tree, &expression)?,
        None => repl.selected_id,
    };
    let existing_description = tree.get_description(target_id)?;

    let new_description = get_input(&existing_description)?;

    if new_description != existing_description {
        println!("Updating description.");
        tree.set_description(target_id, &new_description)?
    }
    Ok(())
}

fn rename_any(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let select_expression = &args[0];
    let new_name = &args[1];
    let id_to_rename = repl.select_from_expression(tree, &select_expression)?;

    tree.set_name(id_to_rename, new_name)
}

fn rename_selected(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let new_name = match args.into_iter().next() {
        Some(new_name) => new_name,
        None => match Repl::prompt_for_args(vec!["new name?"])?.into_iter().next() {
            Some(new_name) => new_name,
            None => return Err(Error::DaVinci("Couldnn't get a new name for the idea".to_string())),
        }
    };

    tree.set_name(repl.selected_id, &new_name)
}
