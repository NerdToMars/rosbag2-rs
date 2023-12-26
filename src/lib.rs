///
/// # Usage
///
/// ```
/// use std::rc::Rc;
/// use std::cell::RefCell;
/// use rosbag2_rs::{Reader, Writer};
/// use anyhow::Result;
/// use tempfile::tempdir;

/// fn main() -> Result<()> {
///     let dir = tempdir()?;
///     let mut writer = Writer::new(dir.path());
///     writer.open()?;
///
///     let connection = writer.add_connection("topic1", "msgtype1", "cdr", "")?;
///     for i in 0..10 {
///         writer.write(&connection, i as i64, &[i * 2 + 1 as u8])?;
///     }
///     let connection = writer.add_connection("topic2", "msgtype2", "cdr", "")?;
///     for i in 0..10 {
///         writer.write(&connection, i as i64, &[i * 2 as u8])?;
///     }
///     writer.close()?;
///
///     let mut reader = Reader::new(dir.path())?;
///     let msg_data: Vec<(i64, i64, Vec<u8>)> = vec![];
///     let msg_data = Rc::new(RefCell::new(msg_data)); // Wrap the vector in Rc and RefCell
///     reader.handle_messages(
///         |(id, timestamp, data)| {
///             // Use `borrow_mut` to get a mutable reference to the vector
///             println!("processed message: {:?} {:?} {:?}", id, timestamp, data);
///
///             msg_data.borrow_mut().push((id, timestamp, data));
///             Ok(())
///         },
///         None,
///         None,
///     )?;
///
///     Ok(())
/// }
/// ```
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
