use std::io::stdout;

use schemars::schema_for;
use serde_json::to_writer;

use common_types::{Board, Message};

fn main() {
    to_writer(stdout(), &schema_for!((Board, Message))).expect("Cannot write json schema")
}
