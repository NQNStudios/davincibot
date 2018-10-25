use repl::*;
use error::*;
use idea::IdeaTree;
use std::collections::HashMap;

use edit_rs::get_input;

pub fn core_commands() -> HashMap<String, Command> {
    let mut commands = HashMap::new();

    {
        commands.insert("help".to_string(), Command {
            description: "Display the full command list, or specific usage instructions for a given command",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, print_help),
                CommandHandler::new(CommandArgs::Amount(1), print_command_help),
            ],
        });
        commands.insert("version".to_string(), Command {
            description: "Display the version of Da Vinci Bot which is installed.",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, |repl, tree, _args| { println!("{}", VERSION); Ok(()) }),
            ],
        });
        commands.insert("print".to_string(), Command {
            description: "Print the current Idea's summary",
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, |repl, tree, _args| repl.print(tree, false))],
        });
        commands.insert("listall".to_string(), Command {
            description: "List all children of the current Idea, including hidden ones.",
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, |repl, tree, _args| list(repl, tree, true))],
        });
        // TODO list needs to allow pagination
        commands.insert("list".to_string(), Command {
            description: "List children of the current Idea",
            delimiter: Some(" ".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, |repl, tree, _args| list(repl, tree, false)),
                CommandHandler::new(CommandArgs::VarArgs, |repl, tree, args| list_with_tags(repl, tree, args)),
            ],
        });
        commands.insert("select".to_string(), Command {
            description: "Select an Idea",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(1), select),
            ],
        });
        commands.insert("up".to_string(), Command {
            description: "Select the current Idea's parent Idea",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, |repl, tree, _args| select(repl, tree, vec!["^".to_string()])),
            ],
        });
        commands.insert("root".to_string(), Command {
            description: "Select the root Idea of the current Tree",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, |repl, tree, _args| select(repl, tree, vec!["@".to_string()])),
            ],
        });
        commands.insert("add".to_string(), Command {
            description: "Add a new Idea as a child of the current one.",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, add_multiple),
                CommandHandler::new(CommandArgs::Amount(1), add),
            ],
        });
        commands.insert("tag".to_string(), Command {
            description: "Add tag(s) to the current Idea",
            delimiter: Some(" ".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Zero, tag_multiple),
                CommandHandler::new(CommandArgs::Minimum(1), tag),
            ],
        });
        commands.insert("untag".to_string(), Command {
            description: "Remove tag(s) from the current Idea",
            delimiter: Some(" ".to_string()),
            handlers: vec![CommandHandler::new(CommandArgs::Minimum(1), untag)],
            // TODO untag multiple?
        });
        commands.insert("cleartags".to_string(), Command {
            description: "Clear all tags from the current Idea",
            delimiter: None,
            handlers: vec![CommandHandler::new(CommandArgs::Zero, cleartags)],
            // TODO cleartags implementation that allows selecting multiple
            // Ideas? (although, cleartags already takes zero arguments so it
            // would need a different command name
        });
        commands.insert("move".to_string(), Command {
            description: "Move Idea(s) from one parent to another",
            delimiter: Some("->".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(2), move_both_args),
                CommandHandler::new(CommandArgs::Amount(1), move_one_arg),
                CommandHandler::new(CommandArgs::Zero, move_multiple),
            ],
        });
        commands.insert("describe".to_string(), Command {
            description: "Edit the current Idea's description",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(1), describe),
                CommandHandler::new(CommandArgs::Zero, describe),
            ],
        });
        commands.insert("rename".to_string(), Command {
            description: "Rename an Idea",
            delimiter: Some("->".to_string()),
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(2), rename_any),
                CommandHandler::new(CommandArgs::Amount(1), rename_selected),
                CommandHandler::new(CommandArgs::Amount(0), rename_selected),
            ],
        });
        commands.insert("search".to_string(), Command {
            description: "Search for Ideas containing a given phrase anywhere",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(1), search),
            ],
        });

        // TODO export command with org mode
        commands.insert("export".to_string(), Command {
            description: "Export the current Idea into another file format (such as Emacs org file)",
            delimiter: None,
            handlers: vec![
                CommandHandler::new(CommandArgs::Amount(1), export),
            ],
        });

        // TODO loop through lines of the description and prompt for select expressions to add them as children. Blank select expression = don't turn into an idea. Once this is done, prompt asking whether to archive this idea. Also, while looping, should probably remove lines as they get ideaifyied?
        // TODO reordering children
        // TODO sorting children -- lexicographically?
        // TODO add n ideas
        // TODO pipe accidental git commands back to the shell, lol?
        // TODO noignore command that adds an empty .ignore child, or clears
        // existing .ignore child's tags
        // TODO generic form of ignore command with syntax meta [meta_type]

        // TODO ignore command that creates an .ignore child if necessary and
        // adds the args as tags. (Should it automatically overwrite inherited
        // ignore tags? i.e. equivalent to typing `noignore` first?)
        // [meta_tag] i.e meta ignore done
    }

    commands
}

fn print_help(repl: &mut Repl, _tree: &mut IdeaTree, _args: Vec<String>) -> Result<()> {
    repl.print_help();

    Ok(())
}

fn print_command_help(repl: &mut Repl, _tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let command_name = args.into_iter().next().unwrap();

    repl.print_command_help(command_name);

    Ok(())
}

fn select(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let id_to_select = repl.select_from_expression(tree, &args[0])?;
    repl.select(id_to_select, tree)?;

    Ok(())
}

fn move_multiple(repl: &mut Repl, tree: &mut IdeaTree, _args: Vec<String>) -> Result<()> {
    let select_expression = &repl.prompt_for_args(vec!["destination?"])?[0];
    let parent_id = repl.select_from_expression(tree, select_expression)?;
    repl.prompt(" idea to move:", |ref repl, select_expression| {
        let id_to_move = repl.select_from_expression(tree, &select_expression)?;
        tree.set_parent(id_to_move, parent_id)?;
        Ok(true)
    }, false); // Don't save idea movement args in history

    Ok(())
}

fn move_one_arg(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let id_to_move = repl.select_from_expression(tree, args[0].as_str())?;
    let further_arg = repl.prompt_for_args(vec!["desination?"])?;
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
    tree.add_tags(repl.selected_id(), tags)
}

fn tag_multiple(repl: &mut Repl, tree: &mut IdeaTree, _args: Vec<String>) -> Result<()> {
    let tags: Vec<String> = repl.prompt_for_args(vec!["tags?"])?[0].split(" ").map(|tag| tag.to_string()).collect();

    // Collect all the ids to tag without applying any,
    // because applying a hide tag to one will change the child indices of the
    // others.
    let mut ids_to_tag = Vec::new();
    repl.prompt(" tag idea ->", |ref repl, select_expression| {
        ids_to_tag.push(repl.select_from_expression(tree, select_expression)?);
        Ok(true)
    }, false); // Don't store this input in history
    for id_to_tag in ids_to_tag {
        tree.add_tags(id_to_tag, tags.clone())?;
    }

    Ok(())
}

fn untag(repl: &mut Repl, tree: &mut IdeaTree, tags: Vec<String>) -> Result<()> {
    tree.remove_tags(repl.selected_id(), tags)
}

fn cleartags(repl: &mut Repl, tree: &mut IdeaTree, _args: Vec<String>) -> Result<()> {
    tree.clear_tags(repl.selected_id())
}

fn list(repl: &Repl, tree: &IdeaTree, show_all: bool) -> Result<()> {
    let shown_child_ids = tree.get_child_ids(repl.selected_id(), false)?;

    for (child_idx, id) in shown_child_ids.iter().enumerate() {
        let child= tree.get_name_with_tags(*id)?;

        println!("{}. {}", child_idx+1, child);
    }

    if show_all {
        let mut hidden_child_ids = tree.get_child_ids(repl.selected_id(), true)?;
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


    Ok(())
}

// Parse the desired parent ID and name of an add expression
// TODO maybe this could go in repl
fn evaluate_add_expression(repl: &mut Repl, tree: &mut IdeaTree, expression: String) -> Result<(i64, String)> {
    // If the idea name contains slashes, then it is selecting a different
    // parent for the new idea
    let parts: Vec<&str> = expression.rsplitn(2, "/").collect();

    let (parent_exp, name) = match parts.len() {
        1 => ("", parts[0]),
        2 => (parts[1], parts[0]), // These indices are reverse from expected because of rsplit
        _ => panic!("rsplitn returned > n components"),
    };

    let mut parent_id = repl.selected_id();
    if parent_exp.len() > 0 {
        parent_id = repl.select_from_expression(tree, parent_exp)?;
    }

    Ok((parent_id, name.to_string()))
}

fn add(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let (parent_id, name) = evaluate_add_expression(repl, tree, args[0].clone())?;

    let id = tree.create_idea(parent_id, name.to_string(), None)?;
    repl.run_command(tree, format!("select #{}", id));

    Ok(())
}

fn add_multiple(repl: &mut Repl, tree: &mut IdeaTree, _args: Vec<String>) -> Result<()> {
    repl.prompt(" new idea ->", |ref mut repl, name: &str| {
        let (parent_id, name) = evaluate_add_expression(*repl, tree, name.to_string())?;
        tree.create_idea(parent_id, name.to_string(), None)?;
        Ok(true)
    }, false); // Don't save Idea names in the command history

    Ok(())
}

fn describe(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let target_id = match args.into_iter().next() {
        Some(expression) => repl.select_from_expression(tree, &expression)?,
        None => repl.selected_id(),
    };
    let existing_description = tree.get_description(target_id)?;

    // TODO edit_rs get_input() seems not to work with rustyline
    // let new_description = get_input(&existing_description)?;

    println!("Description was: {}", existing_description);
    let new_description = repl.prompt_for_args(vec!["description"])?[0].clone();

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
    let old_name = tree.get_name(repl.selected_id())?;
    let new_name = match args.into_iter().next() {
        Some(new_name) => new_name,
        None => get_input(&old_name)?
    };

    if old_name != new_name{
        // TODO this should have to run all other Idea name validations
        if new_name.contains('\n') || new_name.contains('\r') {
            println!("Idea name cannot include newline");
        }
        else {
            println!("Updating name.");
            tree.set_name(repl.selected_id(), &new_name)?;
        }
    }
    else {
        println!("Name unchanged.");
    }

    Ok(())
}

fn search(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let query = args.into_iter().next().unwrap();

    let matches = tree.search_ideas(&query)?;

    if matches.len() == 0 {
        println!("No matches for query '{}'", query);
    } else {
        repl.prompt_to_select_from(&matches, tree);
    }

    Ok(())
}

fn export(repl: &mut Repl, tree: &mut IdeaTree, args: Vec<String>) -> Result<()> {
    let filename = args.into_iter().next().unwrap();
    tree.export_idea(repl.selected_id(), &filename)
}
