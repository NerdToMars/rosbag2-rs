pub mod metadata;
pub use metadata::*;

pub mod sqlite3_storage;
pub use sqlite3_storage::*;

pub mod reader;
pub use reader::*;

pub mod writer;
pub use writer::*;

#[derive(Clone, Debug, PartialEq)]
pub struct TopicConnection {
    pub id: i32,
    pub topic: String,
    pub msgtype: String,
    // pub msgdef: String,
    // pub digest: String,
    pub msgcount: i32,
    pub ext: ConnectionExt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionExt {
    pub serialization_format: String,
    pub offered_qos_profiles: String,
    // Add other fields specific to ROS bag version 2
}

#[derive(Clone, Debug)]
pub struct TopicInfo {
    pub msgtype: String,
    pub msgcount: i32,
    pub connections: Vec<TopicConnection>,
}

impl TopicInfo {
    pub fn new(msgtype: String, msgcount: i32, connections: Vec<TopicConnection>) -> Self {
        TopicInfo {
            msgtype,
            msgcount,
            connections,
        }
    }
}
