#include <Adafruit_BNO055.h>

#define LED_PIN 14

Adafruit_BNO055 bno = Adafruit_BNO055(55);
uint32_t start_time = 0;
uint8_t cycle_time = 10;  // = 1/f in Hz

void setup() {
  pinMode(LED_PIN, OUTPUT_OPENDRAIN);

  if (!bno.begin()) {
    Serial.println("Something went wrong initialising the bno!");
    while (1);
  }

  delay(1000);
  bno.setExtCrystalUse(true);
}

void loop() {
  uint32_t current_time = millis();

  if (current_time - start_time >= cycle_time) {
    start_time = current_time;

    uint8_t system_calib = 0;
    uint8_t gyro_calib = 0;
    uint8_t accel_calib = 0;
    uint8_t mag_calib = 0;

    bno.getCalibration(&system_calib, &gyro_calib, &accel_calib, &mag_calib);

    imu::Quaternion quat = bno.getQuat();
    imu::Vector accel = bno.getVector(Adafruit_BNO055::VECTOR_LINEARACCEL);
    imu::Vector ang = bno.getVector(Adafruit_BNO055::VECTOR_GYROSCOPE);

    // Print in one buffer for performance reasons
    // Format is <4d orientation>;<3d linear accel>;<3d angular vel>;<calibration info>
    char buf[128];
    snprintf(buf, sizeof(buf), "%f,%f,%f,%f;%f,%f,%f;%f,%f,%f;%d,%d,%d,%d\n",
             quat.w(), quat.x(), quat.y(), quat.z(),
             accel.x(), accel.y(), accel.z(),
             ang.x(), ang.y(), ang.z(),
             system_calib, gyro_calib, accel_calib, mag_calib);

    Serial.printf(buf);

    digitalToggle(LED_PIN);
  }
}
