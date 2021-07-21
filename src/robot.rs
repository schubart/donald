use std::cmp::{max, min};
use std::ops::Range;
use std::time::Duration;
use std::time::Instant;

use linux_embedded_hal::i2cdev::core::I2CDevice;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::Address;
use pwm_pca9685::Channel;
use pwm_pca9685::Channel::{C0, C1, C2, C3};
use pwm_pca9685::Pca9685;

type Color = usize;
const COLOR_COUNT: usize = 4;
const COLORS: Range<Color> = 0..COLOR_COUNT;

type ServoPosition = u16;
struct Servo {
    channel: Channel,
    off: ServoPosition,
    on: ServoPosition,
}

type SensorAddress = u8;
type SensorReading = u8;
struct Sensor {
    address: SensorAddress,
}

#[rustfmt::skip]
const SERVOS: [Servo; COLOR_COUNT] = [
    Servo { channel: C0, off: 260, on: 470 }, // red
    Servo { channel: C1, off: 260, on: 470 }, // green
    Servo { channel: C2, off: 260, on: 470 }, // yellow
    Servo { channel: C3, off: 470, on: 260 }, // blue
];

#[rustfmt::skip]
const SENSORS: [Sensor; COLOR_COUNT] = [
    Sensor { address: 0x84 }, // red
    Sensor { address: 0xc4 }, // green
    Sensor { address: 0x94 }, // yellow
    Sensor { address: 0xd4 }, // blue
];

pub struct Robot {
    sensors: I2cdev,
    servos: Pca9685<I2cdev>,
    thresholds: [SensorReading; COLOR_COUNT],
}

impl Robot {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let path = "/dev/i2c-1";

	// Interface with sensors.
        let mut sensors = I2cdev::new(path).unwrap();
        sensors.set_slave_address(0x4b).unwrap();

	// Interface with servos.
        let device = I2cdev::new(path).unwrap();
        let address = Address::default();
        let mut servos = Pca9685::new(device, address).unwrap();
        servos.enable().unwrap();
        servos.set_prescale(100).unwrap();

	// Provisional thresholds until call to `callibrate()`: 50% of max.
        let thresholds = [SensorReading::MAX / 2; COLOR_COUNT];

        Self {
            sensors,
            servos,
            thresholds,
        }
    }

    pub fn calibrate(&mut self, duration: Duration) {
	// At the start of a game the toy blinks all lights for a
	// while. Use this period to read the sensors and keep track
	// of min and max observed values. This gives the readings
	// for lights that are off and on. Calculate thresholds as
	// mid-point between min and max.
        let mut mins = [SensorReading::MAX; COLOR_COUNT];
        let mut maxs = [SensorReading::MIN; COLOR_COUNT];
        let start = Instant::now();
        while start.elapsed() < duration {
            for color in COLORS {
                let value = self.read_sensor(color);
                mins[color] = min(value, mins[color]);
                maxs[color] = max(value, maxs[color]);
                self.thresholds[color] = mins[color] / 2 + maxs[color] / 2;
            }
        }
    }

    pub fn lower_hand(&mut self, color: Color) {
        let servo = &SERVOS[color];
        self.set_servo(servo.channel, servo.on);
    }

    pub fn lift_hand(&mut self, color: Color) {
        let servo = &SERVOS[color];
        self.set_servo(servo.channel, servo.off);
    }

    pub fn lower_all_hands(&mut self) {
        COLORS.for_each(|color| self.lower_hand(color));
    }

    pub fn lift_all_hands(&mut self) {
        COLORS.for_each(|color| self.lift_hand(color));
    }

    pub fn wait_for_any_light_on(&mut self) -> Color {
	COLORS.cycle().find(|&color| self.is_light_on(color)).unwrap()
    }

    pub fn wait_for_light_on(&mut self, color: Color) {
        while !self.is_light_on(color) {}
    }

    pub fn wait_for_light_off(&mut self, color: Color) {
        while self.is_light_on(color) {}
    }

    fn is_light_on(&mut self, color: Color) -> bool {
        self.read_sensor(color) < self.thresholds[color]
    }

    fn read_sensor(&mut self, color: Color) -> SensorReading {
        self.sensors
            .smbus_read_byte_data(SENSORS[color].address)
            .unwrap()
    }

    fn set_servo(&mut self, channel: Channel, position: ServoPosition) {
        self.servos
            .set_channel_on_off(channel, 0, position)
            .unwrap();
    }
}
