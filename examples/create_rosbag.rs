use anyhow::Result;
use rosbag2_rs::Writer;
use std::path::Path;

fn main() -> Result<()> {
    // Specify the path for the new ROS2 bag
    let bag_path = Path::new("example_rosbag_by_rust");

    // Initialize the Writer
    let mut writer = Writer::new(bag_path);
    writer.open()?;

    // Define a standard ROS2 message type (adjust this according to actual ROS2 message types)
    let topic = "example_topic";
    let msgtype = "std_msgs/msg/Int32";

    // Add a connection for this message type
    // adjust qos profile according to actual ROS2 message types (https://docs.ros2.org/latest/api/rmw/structrmw__qos__profile__t.html)
    const LATCH: &str = r#"- history: 3
  depth: 0
  reliability: 1
  durability: 1
  deadline:
    sec: 2147483647
    nsec: 4294967295
  lifespan:
    sec: 2147483647
    nsec: 4294967295
  liveliness: 1
  liveliness_lease_duration:
    sec: 2147483647
    nsec: 4294967295
  avoid_ros_namespace_conventions: false
"#;

    let connection = writer.add_connection(topic, msgtype, "cdr", LATCH)?;

    // Write some dummy messages
    for i in 0..50 {
        let dummy_data = [0, 1, 0, 1, 43, 42, 0, 0]; // dummy data 0x2a2b = (int32)10795
        writer.write(&connection, 1_000_000_000 * i, &dummy_data)?;
    }

    writer.close()?;

    println!("ROS2 bag created at {:?}", bag_path);
    Ok(())
}
