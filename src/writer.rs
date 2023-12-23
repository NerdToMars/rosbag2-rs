use crate::*;
use anyhow::Result;
use rusqlite::params;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

/// This class implements writing of rosbag2 files in version 8. It should be
/// used as a contextmanager.
pub struct Writer {
    pub path: PathBuf,
    pub metapath: PathBuf,
    pub dbpath: PathBuf,
    pub connections: Vec<TopicConnection>,
    pub counts: HashMap<i32, i32>,
    pub conn: Option<Connection>,
    pub custom_data: HashMap<String, String>,
    pub added_types: Vec<String>,
    pub compression_mode: String,
    pub compression_format: String,
}

impl Writer {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        let metapath = path.join("metadata.yaml");
        let dbpath = path.join(format!(
            "{}.db3",
            path.file_name().unwrap().to_str().unwrap()
        ));

        Writer {
            path,
            metapath,
            dbpath,
            connections: Vec::new(),
            counts: HashMap::new(),
            conn: None,
            custom_data: HashMap::new(),
            added_types: Vec::new(),
            compression_mode: "".to_string(),
            compression_format: "".to_string(),
        }
    }

    /// humble schema can be found here:
    /// Rosbag2 <https://github.com/ros2/rosbag2/blob/humble/rosbag2_storage_default_plugins/src/rosbag2_storage_default_plugins/sqlite/sqlite_storage.cpp#L360C13-L360C13>
    pub fn open(&mut self) -> Result<()> {
        if self.dbpath.exists() {
            return Err(anyhow::anyhow!(
                "Database file {:?} already exists.",
                self.dbpath
            ));
        }

        std::fs::create_dir_all(&self.path)?;
        let conn = Connection::open(&self.dbpath)?;

        // TODO: add support for beyond humble ros2bag
        // related discussion: https://github.com/ros2/ros2/issues/1159
        conn.execute_batch(
            r#"
            CREATE TABLE schema(
              schema_version INTEGER PRIMARY KEY,
              ros_distro TEXT NOT NULL
            );
            CREATE TABLE metadata(
              id INTEGER PRIMARY KEY,
              metadata_version INTEGER NOT NULL,
              metadata TEXT NOT NULL
            );
            CREATE TABLE topics(
              id INTEGER PRIMARY KEY,
              name TEXT NOT NULL,
              type TEXT NOT NULL,
              serialization_format TEXT NOT NULL,
              offered_qos_profiles TEXT NOT NULL
            );
            CREATE TABLE messages(
              id INTEGER PRIMARY KEY,
              topic_id INTEGER NOT NULL,
              timestamp INTEGER NOT NULL,
              data BLOB NOT NULL
            );
            CREATE INDEX timestamp_idx ON messages (timestamp ASC);
            INSERT INTO schema(schema_version, ros_distro) VALUES (4, 'rosbags');
            "#,
        )?;
        self.conn = Some(conn);
        Ok(())
    }

    pub fn add_connection(
        &mut self,
        topic: &str,
        msgtype: &str,
        serialization_format: &str,
        offered_qos_profiles: &str,
    ) -> Result<TopicConnection> {
        if self.conn.is_none() {
            return Err(anyhow::anyhow!("Bag was not opened."));
        }

        let conn = self.conn.as_ref().unwrap();

        let new_id = self.connections.len() as i32 + 1;
        let new_connection = TopicConnection {
            id: new_id,
            topic: topic.to_string(),
            msgtype: msgtype.to_string(),
            msgcount: 0,
            ext: ConnectionExt {
                serialization_format: serialization_format.to_string(),
                offered_qos_profiles: offered_qos_profiles.to_string(),
            },
            // owner field is omitted in Rust
        };

        if self
            .connections
            .iter()
            .any(|conn| conn.topic == topic && conn.msgtype == msgtype)
        {
            return Err(anyhow::anyhow!(
                "Connection can only be added once: {:?}",
                new_connection
            ));
        }

        self.connections.push(new_connection.clone());
        self.counts.insert(new_id, 0);
        conn.execute(
            "INSERT INTO topics VALUES(?, ?, ?, ?, ?)",
            [
                new_id.to_string().as_str(),
                topic,
                msgtype,
                serialization_format,
                offered_qos_profiles,
            ],
        )?;

        Ok(new_connection)
    }

    pub fn write(
        &mut self,
        connection: &TopicConnection,
        timestamp: i64,
        data: &[u8],
    ) -> Result<()> {
        if self.conn.is_none() {
            return Err(anyhow::anyhow!("Bag was not opened."));
        }

        let conn = self.conn.as_ref().unwrap();

        if !self.connections.contains(connection) {
            return Err(anyhow::anyhow!(
                "Tried to write to unknown connection {:?}",
                connection
            ));
        }

        // TODO: Handle compression if needed
        // if self.compression_mode == "message" {
        //     // Compress data
        //     // data_to_write = self.compressor.compress(data)?;
        // }

        conn.execute(
            "INSERT INTO messages (topic_id, timestamp, data) VALUES(?1, ?2, ?3)",
            params![
                connection.id.to_string().as_str(),
                &timestamp.to_string(),
                data
            ],
        )?;

        if let Some(count) = self.counts.get_mut(&connection.id) {
            *count += 1;
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if let Some(conn) = self.conn.take() {
            // Calculate duration, start time, and message count
            let (duration, start, count) = conn.query_row(
                "SELECT max(timestamp) - min(timestamp), min(timestamp), count(*) FROM messages",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;

            // Commit and optimize the database
            conn.execute("PRAGMA optimize", [])?;

            // Handle compression (skipped for now as compression is not implemented)

            // Generate metadata
            let metadata = BagFileInfo {
                rosbag2_bagfile_information: self.generate_metadata(duration, start, count)?,
            };
            let file = File::create(&self.metapath)?;
            serde_yaml::to_writer(file, &metadata)?;
        }

        Ok(())
    }

    fn generate_metadata(&self, duration: i64, start: i64, count: i32) -> Result<Metadata> {
        // Placeholder for topics_with_message_count
        let topics_with_message_count: Vec<TopicWithMessageCount> = self
            .connections
            .iter()
            .map(|conn| TopicWithMessageCount {
                message_count: *self.counts.get(&conn.id).unwrap_or(&0),
                topic_metadata: TopicMetadata {
                    name: conn.topic.clone(),
                    type_: conn.msgtype.clone(),
                    serialization_format: conn.ext.serialization_format.clone(),
                    offered_qos_profiles: conn.ext.offered_qos_profiles.clone(),
                    // type_description_hash: conn.digest.clone(), // TODO: implement type_description_hash
                },
            })
            .collect();

        // Generate files information
        let files = vec![FileInformation {
            path: self.dbpath.to_str().unwrap().to_string(),
            starting_time: StartingTime {
                nanoseconds_since_epoch: start,
            },
            duration: BagDuration {
                nanoseconds: duration,
            },
            message_count: count,
        }];

        Ok(Metadata {
            version: 5,
            storage_identifier: "sqlite3".to_string(),
            relative_file_paths: vec![self
                .dbpath
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()],
            starting_time: StartingTime {
                nanoseconds_since_epoch: start,
            },
            duration: BagDuration {
                nanoseconds: duration,
            },
            message_count: count,
            compression_format: self.compression_format.clone(),
            compression_mode: self.compression_mode.clone(),
            topics_with_message_count,
            files,
            custom_data: self.custom_data.clone(),
            ros_distro: "rosbags".to_string(),
        })
    }
}
