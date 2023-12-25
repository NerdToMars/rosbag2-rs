use anyhow::{Ok, Result};
use rosbag2_rs::{BagFileInfo, Writer};
use rusqlite::Connection;
use std::fs::{self, File};
use tempfile::tempdir;

#[test]
fn test_writer_creation_and_opening() -> Result<()> {
    let test_bag_path = "test_bag";

    let _ = fs::remove_dir_all(test_bag_path);

    let mut writer = Writer::new(test_bag_path);
    assert!(writer.open().is_ok());

    assert!(writer.dbpath.exists());

    // Open the SQLite database and check if the 'topics' table exists
    let conn = Connection::open(&writer.dbpath)?;

    let tables = ["schema", "metadata", "topics", "messages"];

    for &table in &tables {
        let exists = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1;",
            [table],
            |row| row.get::<_, i32>(0),
        )?;
        assert_eq!(exists, 1, "Table {} does not exist", table);
    }

    let _ = fs::remove_dir_all(test_bag_path);
    Ok(())
}

#[test]
fn test_open_existing_db() -> Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join(format!(
        "{}.db3",
        dir.path().file_name().unwrap().to_str().unwrap()
    ));

    // Create a dummy file to simulate existing database
    File::create(db_path)?;

    let mut writer = Writer::new(dir.path());
    let result = writer.open();
    assert!(
        result.is_err(),
        "Expected error when opening existing database, but got Ok"
    );
    Ok(())
}

#[test]
fn test_write_operations() -> Result<()> {
    let dir = tempdir()?;

    let mut writer = Writer::new(dir.path());
    writer.open()?;

    // Add a dummy connection
    let connection = writer.add_connection("topic1", "msgtype1", "cdr", "")?;

    // Write dummy messages
    for i in 0..10 {
        writer.write(&connection, i as i64, &[i as u8])?;
    }

    // Close the writer to flush all data to the database
    writer.close()?;

    // Open the SQLite database to verify the data
    let db_conn = Connection::open(writer.dbpath)?;
    let mut stmt = db_conn.prepare("SELECT data FROM messages WHERE topic_id = ?")?;
    let mut rows = stmt.query([connection.id.to_string().as_str()])?;

    let mut i = 0;
    while let Some(row) = rows.next()? {
        let data: Vec<u8> = row.get(0)?;
        assert_eq!(data, vec![i as u8]);
        i += 1;
    }

    assert_eq!(i, 10); // Ensure we read back 10 messages

    // also check metadata.yaml
    // Read and verify the YAML metadata file
    let metadata_contents = fs::read_to_string(writer.metapath)?;
    let bag_info: BagFileInfo = serde_yaml::from_str(&metadata_contents)?;
    let metadata = bag_info.rosbag2_bagfile_information;
    assert_eq!(metadata.message_count, 10);
    assert_eq!(metadata.topics_with_message_count.len(), 1);
    assert_eq!(metadata.topics_with_message_count[0].message_count, 10);
    Ok(())
}
