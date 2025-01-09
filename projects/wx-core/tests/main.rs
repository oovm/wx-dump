#[test]
fn ready() {
    println!("it works!")
}

use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};

/// `<revokemsg>"某人" 撤回了一条消息</revokemsg>`
#[derive(Debug, Deserialize)]
pub struct RevokeMessage {
    #[serde(rename = "$value")]
    field: String,
}
#[test]
fn main() {
    let xml_data = r#"<revokemsg>"<XXX>" 撤回了一条消息</revokemsg>"#;

    let message: RevokeMessage = from_str(xml_data).unwrap();
    println!("{:?}", message);
}
