#[macro_use]
extern crate serde_derive;
extern crate serde;
// TODO use END instead?
extern crate serde_json;

extern crate mentat;
use mentat::store;

// An Idea is the basic building block of Da Vinci Bot.
// TODO explain exactly how Ideas work and why

#[derive(Serialize, Deserialize)]
struct Idea {
    // TODO is 64 bits enough for Da Vinci ID's?
    id: i64,
    name: String,
    description: String,
    tags: Vec<String>,
    child_ids: Vec<i64>,
    // TODO add attachments to Ideas
    // TODO add extra serde-serializable data to Ideas
}

fn main() {
    println!("Hello, Mx. Da Vinci!");
    // TODO load the root Idea or create it

    let root_idea = Idea { id: 0, name: "Do All the Vastly Impractical Nonsense Conceivable In (short) Bursts Of Time", description: "Here's the root of all your brilliant Ideas.", tags: vec!(), child_ids: vec!() };
}
