pub mod metadata;
pub use metadata::*;

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
