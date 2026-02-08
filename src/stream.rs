use std::str;
use std::time::Duration;

use futures::StreamExt;
use r2r::log_info;
use r2r::sensor_msgs::msg::Imu;
use r2r::{Clock, ClockType};
use tokio_serial::SerialStream;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use crate::error::Error;
use crate::parsing::DataPoint;

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find newline char, split off buffer up to that point
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => {
                    if s.len() < 2 {
                        return Err(Error::StreamDecodeFailure(format!("Line too short: {}", s)));
                    }
                    // Remove the newline char
                    Ok(Some(s[..s.len() - 1].to_string()))
                }
                Err(e) => Err(Error::StreamDecodeFailure(format!(
                    "Failed to parse line: {:?}",
                    e
                ))),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn decode_line(line: String, clk: &mut Clock) -> Result<(Imu, [f64; 4]), Error> {
    let data_point = DataPoint::try_from(line)?;
    let calibration = data_point.calibration_status;
    let imu = data_point.into_imu(clk)?;
    Ok((imu, calibration))
}

// Listen on the serial port for raw imu data and publish it as a proper imu sensor msg
pub async fn stream_imu_data(imu_pub: r2r::Publisher<Imu>, port: SerialStream, lg_name: String) {
    let mut clk = Clock::create(ClockType::SystemTime).expect("Unable to create a clock");
    let mut start = clk.get_now().expect("Unable to get current time");
    let lg_id = lg_name.as_str();

    let mut reader = LineCodec.framed(port);
    while let Some(result) = reader.next().await {
        match result {
            Ok(line) => match decode_line(line, &mut clk) {
                Ok((imu, calibration)) => {
                    if let Ok(now) = clk.get_now() {
                        if now > start && (now - start) >= Duration::from_secs(1) {
                            log_info!(lg_id, "{:?}", calibration);
                            start = now;
                        }
                    }
                    imu_pub.publish(&imu).unwrap_or(())
                }
                Err(err) => log_info!(lg_id, "{}", err),
            },
            Err(err) => log_info!(lg_id, "{}", err),
        }
    }
}
