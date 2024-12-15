use r2r::builtin_interfaces::msg::Time;
use r2r::geometry_msgs::msg::{Quaternion, Vector3};
use r2r::sensor_msgs::msg::Imu;
use r2r::std_msgs::msg::Header;
use r2r::Clock;

use crate::error::ExtractionError;

pub fn get_imu_msg_and_calibration(
    datapoints: Vec<DataPoint>,
    clk: &mut Clock,
) -> Result<(Imu, [f64; 4]), ExtractionError> {
    let mut accel = None;
    let mut ang = None;
    let mut orient = None;
    let mut calib = None;

    for datapoint in datapoints {
        match datapoint {
            DataPoint::Acceleration(vec) => {
                accel = Some(vec);
            }
            DataPoint::AngularVelocity(vec) => {
                ang = Some(vec);
            }
            DataPoint::Orientation(quat) => {
                orient = Some(quat);
            }
            DataPoint::CalibrationStatus(arr) => {
                calib = Some(arr);
            }
        }
    }

    if let (Some(l), Some(a), Some(o), Some(c)) = (accel, ang, orient, calib) {
        let dur = clk
            .get_now()
            .map_err(|_| ExtractionError::Unknown("Failed to get current time".to_owned()))?;
        Ok((
            Imu {
                header: Header {
                    stamp: Time {
                        nanosec: dur.subsec_nanos(),
                        sec: dur.as_secs() as i32,
                    },
                    frame_id: "0".to_owned(),
                },
                orientation: o,
                // No covariance data currently so set all to 0
                orientation_covariance: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                angular_velocity: a,
                angular_velocity_covariance: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                linear_acceleration: l,
                linear_acceleration_covariance: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            },
            c,
        ))
    } else {
        Err(ExtractionError::MissingDataPoint)
    }
}

pub enum DataPoint {
    Acceleration(Vector3),
    AngularVelocity(Vector3),
    Orientation(Quaternion),
    CalibrationStatus([f64; 4]),
}

impl TryFrom<&str> for DataPoint {
    type Error = ExtractionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Prevent out of bounds indexing
        if value.len() < 3 {
            return Err(ExtractionError::TooSmall);
        }

        let mut vec = vec![];
        // Strip off prefix and parse comma separated floats
        for num in value[2..].split(",") {
            match num.parse::<f64>() {
                Ok(parsed) => vec.push(parsed),
                Err(_) => return Err(ExtractionError::ParseError),
            }
        }

        let prefix = &value[..2];
        if prefix == "o:" && vec.len() == 4 {
            Ok(DataPoint::Orientation(Quaternion {
                w: vec[0],
                x: vec[1],
                y: vec[2],
                z: vec[3],
            }))
        } else if prefix == "l:" && vec.len() == 3 {
            Ok(DataPoint::Acceleration(Vector3 {
                x: vec[0],
                y: vec[1],
                z: vec[2],
            }))
        } else if prefix == "a:" && vec.len() == 3 {
            Ok(DataPoint::AngularVelocity(Vector3 {
                x: vec[0],
                y: vec[1],
                z: vec[2],
            }))
        } else if prefix == "c:" && vec.len() == 4 {
            Ok(DataPoint::CalibrationStatus([
                vec[0], vec[1], vec[2], vec[3],
            ]))
        } else {
            Err(ExtractionError::UnknownType)
        }
    }
}
