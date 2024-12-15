mod error;
mod parsing;
mod stream;

use std::hash::RandomState;
use std::str;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use r2r::indexmap::IndexMap;
use r2r::sensor_msgs::msg::Imu;
use r2r::{log_info, Context, Node, Parameter, ParameterValue, QosProfile};
use tokio_serial::SerialPortBuilderExt;

use crate::stream::stream_imu_data;

#[tokio::main]
async fn main() {
    let ctx = Context::create().expect("Failed to create context");
    let mut node = Node::create(ctx, "imu", "robot").expect("Failed to create node");

    let lg_name = node.logger().to_string();
    let id = get_imu_id(node.params.clone());
    let serial_port_path = get_serial_port_path(node.params.clone());

    let imu_pub = node
        .create_publisher::<Imu>(&format!("/imu_{}", id), QosProfile::default())
        .expect("Failed to create imu msg publisher");

    let port = tokio_serial::new(serial_port_path, 115200)
        .open_native_async()
        .expect("Failed to open stream to serial port");

    log_info!(lg_name.as_str(), "Starting imu data transmitter");
    tokio::spawn(stream_imu_data(imu_pub, port, lg_name));
    loop {
        node.spin_once(Duration::from_millis(100));
    }
}

type ParamMap = Arc<Mutex<IndexMap<String, Parameter, RandomState>>>;

// Get the value of a parameter. Can panic as is required for startup
fn get_parameter(params_mut: ParamMap, key: &str) -> ParameterValue {
    let params = params_mut.lock().unwrap();
    let param = params
        .get(key)
        .unwrap_or_else(|| panic!("Missing ros parameter: {}", key));
    param.value.clone()
}

fn get_imu_id(params_mut: ParamMap) -> i64 {
    match get_parameter(params_mut, "id") {
        ParameterValue::Integer(val) => val,
        val => panic!("Wrong id format, value: {:?}", val),
    }
}

fn get_serial_port_path(params_mut: ParamMap) -> String {
    match get_parameter(params_mut, "serial_port_path") {
        ParameterValue::String(val) => val,
        val => panic!("Wrong serial port path format, value: {:?}", val),
    }
}
