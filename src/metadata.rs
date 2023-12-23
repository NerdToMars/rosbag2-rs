use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct StartingTime {
    pub nanoseconds_since_epoch: i64,
}

#[derive(Serialize, Deserialize)]
pub struct BagDuration {
    pub nanoseconds: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TopicMetadata {
    pub name: String,

    #[serde(rename = "type")]
    pub type_: String, // `type` is a reserved keyword in Rust
    pub serialization_format: String,
    pub offered_qos_profiles: String,
    // pub type_description_hash: String // TODO: humble rosbag2 does not need this field
}

#[derive(Serialize, Deserialize)]
pub struct TopicWithMessageCount {
    pub message_count: i32,
    pub topic_metadata: TopicMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct FileInformation {
    pub path: String,
    pub starting_time: StartingTime,
    pub duration: BagDuration,
    pub message_count: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub version: i32,
    pub storage_identifier: String,
    pub relative_file_paths: Vec<String>,
    pub starting_time: StartingTime,
    pub duration: BagDuration,
    pub message_count: i32,
    pub compression_format: String,
    pub compression_mode: String,
    pub topics_with_message_count: Vec<TopicWithMessageCount>,
    pub files: Vec<FileInformation>,
    pub custom_data: HashMap<String, String>,
    pub ros_distro: String,
}

#[derive(Serialize, Deserialize)]
pub struct BagFileInfo {
    pub rosbag2_bagfile_information: Metadata,
}
