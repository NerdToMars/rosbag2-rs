use crate::*;
use anyhow::Result;
use rusqlite::{Connection, Statement};
use std::path::Path;

pub struct Sqlite3Reader {
    paths: Vec<String>, // Assuming paths are stored as strings
    dbconns: Vec<Connection>,
    schema: i32,
    // msgtypes: Vec<String>,
    // connections: Vec<TopicConnection>,
}

impl Sqlite3Reader {
    pub fn new(paths: Vec<String>) -> Self {
        Sqlite3Reader {
            paths,
            dbconns: Vec::new(),
            schema: 0,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        for path_str in &self.paths {
            let path = Path::new(path_str);
            println!("opening db using path {path:?}");
            let conn = Connection::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            )?;

            {
                let mut stmt = conn.prepare(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('messages', 'topics')",
                )?;

                let table_count: i32 = stmt.query_row([], |row| row.get(0))?;

                if table_count != 2 {
                    return Err(anyhow::anyhow!(
                        "Cannot open database {} or database missing tables.",
                        path_str
                    ));
                }
            }

            self.dbconns.push(conn);
        }

        // Check the schema version and initialize `self.schema` and `self.msgtypes`
        if let Some(conn) = self.dbconns.last() {
            let mut stmt = conn.prepare("PRAGMA table_info(schema)")?;

            self.schema = if stmt.exists([])? {
                let schema_version: i32 =
                    conn.query_row("SELECT schema_version FROM schema", [], |row| row.get(0))?;
                schema_version
            } else {
                let mut stmt = conn.prepare("PRAGMA table_info(topics)")?;
                let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

                if rows
                    .filter(
                        |r| matches!(r, Ok(column_name) if column_name == "offered_qos_profiles"),
                    )
                    .count()
                    > 0
                {
                    2
                } else {
                    1
                }
            };

            // TODO: Initialize `self.msgtypes` based on the schema version
        }

        Ok(())
    }

    /// clear all SQLite connections
    pub fn close(&mut self) {
        self.dbconns.clear();
    }

    // pub fn get_statuement(&self) -> Statement {}

    pub fn messages_statement(
        &self,
        connections: &[TopicConnection],
        start: Option<i64>,
        stop: Option<i64>,
    ) -> Result<Statement> {
        if self.dbconns.is_empty() {
            return Err(anyhow::anyhow!("Rosbag has not been opened."));
        }

        let mut query = String::from(
            "SELECT topics.id, messages.timestamp, messages.data FROM messages JOIN topics ON messages.topic_id=topics.id",
        );
        let mut args: Vec<String> = vec![];
        let mut clause = "WHERE";

        if !connections.is_empty() {
            let topics: Vec<String> = connections.iter().map(|c| c.topic.clone()).collect();
            let formated_topics = topics
                .iter()
                .map(|t| format!("'{}'", t))
                .collect::<Vec<_>>()
                .join(", ");
            query.push_str(&format!(" {clause} topics.name IN ({formated_topics})"));
            clause = "AND";
        }

        if let Some(start_ns) = start {
            // query.push_str(&format!(" {clause} messages.timestamp >= ?", clause));
            query.push_str(&format!(" {clause} messages.timestamp >= {start_ns}"));
            args.push(start_ns.to_string());
            clause = "AND";
        }

        if let Some(stop_ns) = stop {
            // query.push_str(&format!(" {} messages.timestamp < ?", clause));
            query.push_str(&format!(" {clause} messages.timestamp < {stop_ns}"));
            args.push(stop_ns.to_string());
        }

        query.push_str(" ORDER BY messages.timestamp");

        if self.dbconns.len() > 1 {
            panic!("not support multiple db3 files");
        }

        return if let Some(conn) = self.dbconns.last() {
            println!("query string is {query}");
            let stmt = conn.prepare(&query)?;
            // let rows = stmt.query([])?; parse_row
            // let messages = stmt.query_map([], parse_row)?;
            // let messages = stmt.query_map([], |row| {
            //     let id: i32 = row.get(0)?;
            //     let timestamp: i64 = row.get(1)?;
            //     let data: Vec<u8> = row.get(2)?;
            //     Ok((id, timestamp, data))
            // })?;
            // let
            Ok(stmt)
            // let t = messages.into_iter();
            // Ok(messages)
            // Err(anyhow::anyhow!("Cannot open database."))
        } else {
            Err(anyhow::anyhow!("Cannot open database."))
        };
    }
}

pub fn handle_messages<F: Fn((i64, i64, Vec<u8>)) -> Result<()>>(
    mut stmt: Statement,
    handle_func: F,
) -> Result<()> {
    let handler = stmt.query_map([], |row| {
        let id = row.get::<_, i64>(0)?;
        let timestamp = row.get(1)?;
        let data = row.get(2)?;
        let result = handle_func((id, timestamp, data));
        if let Err(e) = result {
            return Err(rusqlite::Error::InvalidParameterName(format!("error: {e}")));
        }
        Ok(())
    })?;

    for res in handler {
        if let Err(e) = res {
            println!("error: {e:?} when handle messages");
        }
    }
    Ok(())
}
