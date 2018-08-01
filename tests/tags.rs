extern crate davincibot;
use davincibot::idea::*;
use davincibot::repl::*;

fn test_tree() -> IdeaTree {
    let mut tree = IdeaTree::open_in_memory().unwrap();
    let mut repl = Repl::new();

    // The root Idea (id 1) will have tags "test1" and "test2"
    repl.run_command(&mut tree, "cleartags".to_string());
    repl.run_command(&mut tree, "tag test1 test2".to_string());

    // Create a child Idea (id 3) without tags
    repl.run_command(&mut tree, "add child1".to_string());

    // Create a child Idea (id 4) which overwrites .ignore tags
    repl.run_command(&mut tree, "add child2".to_string());
    repl.run_command(&mut tree, "select child2".to_string());
    repl.run_command(&mut tree, "add .ignore".to_string());

    tree
}

#[test]
fn get_tags_without_inheritance() {
    let test_tree = test_tree();
    // Prove that tagging works
    assert_eq!(test_tree.get_tags(1, false).unwrap(), vec!["test1", "test2"]);

    // Prove that children don't inherit tags by default
    let no_tags: Vec<String> = vec![];
    assert_eq!(test_tree.get_tags(3, false).unwrap(), no_tags);
}

#[test]
fn get_tags_with_inheritance() {
    let test_tree = test_tree();
    assert!(test_tree.get_tags(3, true).unwrap().contains(&"test1".to_string()));
}

#[test]
fn meta_tags() {
    let test_tree = test_tree();
    assert_eq!(test_tree.get_meta_tags(1, "ignore").unwrap(), vec!["done", "hidden", "archived", "paused"]);
    assert_eq!(test_tree.get_meta_tags(3, "ignore").unwrap(), vec!["done", "hidden", "archived", "paused"]);
    assert_eq!(test_tree.get_meta_tags(4, "ignore").unwrap(), Vec::<String>::new());
}
