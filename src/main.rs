mod error;
mod parsing;
mod stream;

use std::time::Duration;

use r2r::sensor_msgs::msg::Imu;
use r2r::{log_info, Context, Error, Node, QosProfile};
use tokio_serial::SerialPortBuilderExt;

use crate::stream::stream_imu_data;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let ctx = Context::create()?;
    let mut node = Node::create(ctx, "imu", "robot")?;

    let lg_name = node.logger().to_string();
    let id: i64 = node.get_parameter("id")?;
    let serial_port_path: String = node.get_parameter("serial_port_path")?;

    let imu_pub = node.create_publisher::<Imu>(&format!("/imu_{}", id), QosProfile::default())?;

    let port = tokio_serial::new(serial_port_path, 115200)
        .open_native_async()
        .expect("Failed to open stream to serial port");

    log_info!(lg_name.as_str(), "Starting imu data transmitter");
    tokio::spawn(stream_imu_data(imu_pub, port, lg_name));

    loop {
        node.spin_once(Duration::from_millis(100));
    }
}
