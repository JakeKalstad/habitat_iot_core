use esp_idf_hal::gpio::{Gpio21, Gpio22, Gpio23, Gpio25, Output, Pin, PinDriver};
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::prelude::*;

/// Represents the L298N motor driver with two motors.
pub struct L298N<MOTOR_A, MOTOR_B> {
    pub a: MOTOR_A,
    pub b: MOTOR_B,
}
pub struct Motor<INF, INS>
where
    INF: Pin,
    INS: Pin,
{
    in1: PinDriver<'static, INF, Output>,
    in2: PinDriver<'static, INS, Output>,
    pwm: LedcDriver<'static>,
}

impl<INF, INS> Motor<INF, INS>
where
    INF: Pin,
    INS: Pin,
{
    /// Creates a new motor.
    pub fn new(
        mut in1: PinDriver<'static, INF, Output>,
        mut in2: PinDriver<'static, INS, Output>,
        mut pwm: LedcDriver<'static>,
    ) -> Motor<INF, INS> {
        // Ensure the motor is initially stopped
        in1.set_low().unwrap();
        in2.set_low().unwrap();
        pwm.set_duty(0).unwrap();

        Motor { in1, in2, pwm }
    }

    /// Moves the motor forward.
    pub fn forward(&mut self) {
        self.in1.set_high().unwrap();
        self.in2.set_low().unwrap();
    }

    /// Moves the motor backward.
    pub fn reverse(&mut self) {
        self.in1.set_low().unwrap();
        self.in2.set_high().unwrap();
    }

    /// Stops the motor.
    pub fn stop(&mut self) {
        self.in1.set_low().unwrap();
        self.in2.set_low().unwrap();
        self.pwm.set_duty(0).unwrap();
    }

    /// Sets the motor speed.
    pub fn set_speed(&mut self, duty: u32) {
        self.pwm.set_duty(duty).unwrap();
    }
}