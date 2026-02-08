use r2r::builtin_interfaces::msg::Time;
use r2r::geometry_msgs::msg::{Quaternion, Vector3};
use r2r::sensor_msgs::msg::Imu;
use r2r::std_msgs::msg::Header;
use r2r::Clock;

use crate::error::Error;

pub struct DataPoint {
    acceleration: Vector3,
    angular_velocity: Vector3,
    orientation: Quaternion,
    pub calibration_status: [f64; 4],
}

impl DataPoint {
    pub fn into_imu(self, clk: &mut Clock) -> Result<Imu, Error> {
        let dur = clk
            .get_now()
            .map_err(|_| Error::Unknown("Failed to get current time".to_owned()))?;
        Ok(Imu {
            header: Header {
                stamp: Time {
                    nanosec: dur.subsec_nanos(),
                    sec: dur.as_secs() as i32,
                },
                frame_id: "0".to_owned(),
            },
            orientation: self.orientation,
            // No covariance data currently so set all to 0
            orientation_covariance: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            angular_velocity: self.angular_velocity,
            angular_velocity_covariance: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            linear_acceleration: self.acceleration,
            linear_acceleration_covariance: vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        })
    }
}

impl TryFrom<String> for DataPoint {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut values: Vec<Vec<f64>> = vec![];

        // Split into each variable
        for part in value.split(";") {
            let mut v = vec![];
            // Parse comma separated floats
            for num in part.split(",") {
                match num.parse::<f64>() {
                    Ok(parsed) => v.push(parsed),
                    Err(_) => {
                        return Err(Error::InvalidData(format!(
                            "Failed to parse float: {}",
                            num
                        )))
                    }
                }
            }
            values.push(v);
        }
        // Prevent out of bounds indexing
        if values.len() != 4 {
            return Err(Error::InvalidData(format!(
                "Wrong number of variables in line, got {}",
                values.len()
            )));
        }
        for (i, expected_len) in [4, 3, 3, 4].into_iter().enumerate() {
            if values[i].len() != expected_len {
                return Err(Error::InvalidData(format!(
                    "Expected {} values, got: {:?}",
                    expected_len, values[0]
                )));
            }
        }
        let orientation = Quaternion {
            w: values[0][0],
            x: values[0][1],
            y: values[0][2],
            z: values[0][3],
        };
        let acceleration = Vector3 {
            x: values[1][0],
            y: values[1][1],
            z: values[1][2],
        };
        let angular_velocity = Vector3 {
            x: values[2][0],
            y: values[2][1],
            z: values[2][2],
        };
        let calibration_status = [values[3][0], values[3][1], values[3][2], values[3][3]];

        Ok(Self {
            acceleration,
            angular_velocity,
            orientation,
            calibration_status,
        })
    }
}
