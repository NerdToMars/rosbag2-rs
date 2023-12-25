use anyhow::Result;
use rosbag2_rs::{Reader, Writer};
use std::{cell::RefCell, rc::Rc};
use tempfile::tempdir;

#[test]
fn test_messages_iterator_initialization() -> Result<()> {
    // Initialize with mock data and verify properties

    let dir = tempdir().unwrap();

    let mut writer = Writer::new(dir.path());
    writer.open()?;

    // Add a dummy connection
    let connection = writer.add_connection("topic1", "msgtype1", "cdr", "")?;

    // Write dummy messages
    for i in 0..10 {
        writer.write(&connection, i as i64, &[(i * 2 + 1) as u8])?;
    }

    // Add a dummy connection
    let connection = writer.add_connection("topic2", "msgtype2", "cdr", "")?;

    // Write dummy messages
    for i in 0..10 {
        writer.write(&connection, i as i64, &[(i * 2) as u8])?;
    }

    writer.close()?;

    let mut reader = Reader::new(dir.path())?;

    assert_eq!(reader.connections.len(), 2);
    assert_eq!(reader.connections[0].topic, "topic1");
    assert_eq!(reader.connections[0].msgtype, "msgtype1");
    assert_eq!(reader.connections[0].msgcount, 10);

    assert_eq!(reader.duration(), 10);

    let msg_data: Vec<(i64, i64, Vec<u8>)> = vec![];
    let msg_data = Rc::new(RefCell::new(msg_data)); // Wrap the vector in Rc and RefCell

    reader.handle_messages(
        |(id, timestamp, data)| {
            // Use `borrow_mut` to get a mutable reference to the vector
            println!("processed message: {:?} {:?} {:?}", id, timestamp, data);

            msg_data.borrow_mut().push((id, timestamp, data));
            Ok(())
        },
        None,
        None,
    )?;

    assert_eq!(msg_data.borrow().len(), 20);

    Ok(())
}
