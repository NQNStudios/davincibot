#[macro_use]
extern crate serde_derive;
extern crate serde;
// TODO use EDN instead?
extern crate serde_json;

extern crate mentat;
extern crate mentat_cli;
use mentat::store;
/*extern crate mentat_query;*/
use mentat::Queryable;
use mentat_cli::repl;

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
    let mut repl = repl::Repl::new(true).unwrap();
    repl.run(None);

    /*let store = store::Store::open("test.db").unwrap(); */

    /*let query_output = store.q_once("[:person/name                  :db.type/string :db.cardinality/one]", None).unwrap();*/
    /*let query_output = store.q_once("[:find ?e :where [ ?e :person/name \"Ridley Scott\"] ]", None).unwrap();*/



    /*println!("Hello, Mx. Da Vinci!");*/
    /*// TODO load the root Idea or create it*/

    /*let root_idea = Idea { id: 0, name: "Do All the Vastly Impractical Nonsense Conceivable In (short) Bursts Of Time".to_string(), description: "Here's the root of all your brilliant Ideas.".to_string(), tags: vec!(), child_ids: vec!() };*/
}
