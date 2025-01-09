#[test]
fn ready() {
    println!("it works!")
}

use anyhow::{Context, Result};
use quick_xml::de::{Deserializer, from_str};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use xmltree::{Element, ParseError};

