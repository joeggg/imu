use std::str;
use std::time::Duration;

use r2r::log_info;
use r2r::sensor_msgs::msg::Imu;
use r2r::{Clock, ClockType};
use tokio_serial::SerialStream;

use crate::parsing::{get_imu_msg_and_calibration, DataPoint};

// Listen on the serial port for raw imu data and publish it as a proper imu sensor msg
pub async fn stream_imu_data(
    imu_pub: r2r::Publisher<Imu>,
    mut port: SerialStream,
    lg_name: String,
) {
    let mut clk = Clock::create(ClockType::SystemTime).expect("Unable to create a clock");
    let mut start = clk.get_now().expect("Unable to get current time");
    let lg_id = lg_name.as_str();
    loop {
        if port.readable().await.is_err() {
            continue;
        }
        let mut buf = [0; 200];
        if let Ok(written) = port.try_read(&mut buf) {
            // Convert byte buffer to string
            match str::from_utf8(&buf[..written]) {
                Ok(raw) => {
                    // Convert each line to a DataPoint
                    let mut datapoints: Vec<DataPoint> = Vec::new();
                    raw.lines().for_each(|line| match line.try_into() {
                        Ok(datapoint) => datapoints.push(datapoint),
                        Err(err) => log_info!(lg_id, "{}", err),
                    });

                    match get_imu_msg_and_calibration(datapoints, &mut clk) {
                        Ok((imu, calib)) => {
                            if let Ok(now) = clk.get_now() {
                                if now > start && (now - start) >= Duration::from_secs(1) {
                                    log_info!(lg_id, "{:?}", calib);
                                    start = now;
                                }
                            }
                            imu_pub.publish(&imu).unwrap_or(())
                        }
                        Err(err) => log_info!(lg_id, "{}", err),
                    }
                }
                Err(err) => log_info!(lg_id, "{}", err),
            }
        }
    }
}
