use crate::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use std::{fs, vec};

// Define other structs like Metadata, FileInformation, Connection, etc.

pub struct Reader {
    pub metadata: Metadata,
    pub connections: Vec<TopicConnection>,
    storage: Sqlite3Reader,
}

/// The `Reader` struct provides an interface for reading message data from a ROS bag file.
///
/// The `Reader` initializes with the path to a ROS bag directory and reads metadata
/// and message data from the storage. It supports filtering messages by time and handling
/// each message through a user-defined function.
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
/// let dir = tempdir()?;
/// # let mut writer = Writer::new(dir.path());
/// # writer.open()?;
/// # let connection = writer.add_connection("topic1", "msgtype1", "cdr", "")?;

/// # for i in 0..10 {
/// #     writer.write(&connection, i as i64, &[i * 2 + 1 as u8])?;
/// # }
/// # let connection = writer.add_connection("topic2", "msgtype2", "cdr", "")?;

/// # for i in 0..10 {
/// #     writer.write(&connection, i as i64, &[i * 2 as u8])?;
/// # }
/// # writer.close()?;

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
///
/// # Errors
///
/// - Returns an error if the ROS bag version is not supported.
/// - Returns an error if the compression mode is not supported.
/// - Returns an error if a non-CDR serialization format is found in any topic.
/// - Returns an error if the storage identifier is not 'sqlite3'.
///
/// # Note
///
/// - This struct assumes that the ROS bag files are in `sqlite3` format.
/// - The `handle_messages` method allows for processing of individual messages.

impl Reader {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let metapath = path.join("metadata.yaml");

        let metadata_contents = fs::read_to_string(metapath).unwrap();
        let bag_info: BagFileInfo = serde_yaml::from_str(&metadata_contents).unwrap();

        let metadata = bag_info.rosbag2_bagfile_information;

        // Check version and storage identifier
        if metadata.version > 5 {
            return Err(anyhow!("Not supported version: {}", metadata.version));
        }

        if !metadata.compression_mode.is_empty() {
            return Err(anyhow!(
                "Not supported compression mode: {}",
                metadata.compression_mode
            ));
        }

        // metadata.topics_with_message_count.iter().map(|topi)
        if let Some(topic_info) = metadata
            .topics_with_message_count
            .iter()
            .find(|t| t.topic_metadata.serialization_format != "cdr")
        {
            return Err(anyhow!(
                "Only CDR serialization format is supported: {:?}",
                topic_info.topic_metadata.serialization_format
            ));
        }

        if metadata.storage_identifier != "sqlite3" {
            return Err(anyhow!(
                "Not supported storage identifier: {}",
                metadata.storage_identifier
            ));
        }

        // Initialize connections
        let connections = metadata
            .topics_with_message_count
            .iter()
            .enumerate()
            .map(|(idx, topic_info)| TopicConnection {
                id: idx as i32 + 1,
                topic: topic_info.topic_metadata.name.clone(),
                msgtype: topic_info.topic_metadata.type_.clone(),
                msgcount: topic_info.message_count,
                ext: ConnectionExt {
                    serialization_format: topic_info.topic_metadata.serialization_format.clone(),
                    offered_qos_profiles: topic_info.topic_metadata.offered_qos_profiles.clone(),
                },
            })
            .collect::<Vec<_>>();

        let paths: Vec<String> = metadata
            .relative_file_paths
            .iter()
            .map(|relative_path| path.join(relative_path).to_string_lossy().into_owned())
            .collect();
        let mut storage = Sqlite3Reader::new(paths);

        println!("Opening storage");
        storage.open()?;
        println!("Opening storage Done");

        Ok(Self {
            metadata,
            connections,
            storage,
        })
    }

    pub fn open(&mut self) -> Result<()> {
        self.storage.open()?;

        Ok(())
    }

    pub fn handle_messages(
        &mut self,
        handle_func: impl Fn((i64, i64, Vec<u8>)) -> Result<()>,
        start: Option<i64>,
        stop: Option<i64>,
    ) -> Result<()> {
        let statement = self
            .storage
            .messages_statement(&self.connections, start, stop)?;
        handle_messages(statement, handle_func)?;
        Ok(())
    }

    pub fn duration(&self) -> i64 {
        let nsecs = self.metadata.duration.nanoseconds;
        if self.message_count() > 0 {
            nsecs + 1
        } else {
            0
        }
    }

    pub fn start_time(&self) -> i64 {
        let nsecs = self.metadata.starting_time.nanoseconds_since_epoch;
        if self.message_count() > 0 {
            nsecs
        } else {
            i64::MAX
        }
    }

    pub fn end_time(&self) -> i64 {
        self.start_time() + self.duration()
    }

    pub fn message_count(&self) -> i32 {
        self.metadata.message_count
    }

    pub fn compression_format(&self) -> String {
        self.metadata.compression_format.clone()
    }

    pub fn compression_mode(&self) -> Option<String> {
        let mode = self.metadata.compression_mode.to_lowercase();
        if mode != "none" {
            Some(mode)
        } else {
            None
        }
    }

    pub fn topics(&self) -> HashMap<String, TopicInfo> {
        // Assuming TopicInfo is already defined
        self.connections
            .iter()
            .map(|conn| {
                (
                    conn.topic.clone(),
                    TopicInfo::new(conn.msgtype.clone(), conn.msgcount, vec![conn.clone()]),
                )
            })
            .collect()
    }

    pub fn ros_distro(&self) -> String {
        self.metadata.ros_distro.clone()
    }
}
