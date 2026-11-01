use embedded_hal::digital::{ErrorType, OutputPin};
use pcd8544::PCD8544;
use std::convert::Infallible;
use std::fmt::Write;

pub struct DummyOutputPin;

impl ErrorType for DummyOutputPin {
    type Error = Infallible;
}

impl OutputPin for DummyOutputPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn set_state(&mut self, state: embedded_hal::digital::PinState) -> Result<(), Self::Error> {
        match state {
            embedded_hal::digital::PinState::Low => self.set_low(),
            embedded_hal::digital::PinState::High => self.set_high(),
        }
    }
}

fn main() {
    let pcd_light = DummyOutputPin;
    let pcd_clk = DummyOutputPin;
    let pcd_din = DummyOutputPin;
    let pcd_dc = DummyOutputPin;
    let pcd_ce = DummyOutputPin;
    let pcd_rst = DummyOutputPin;

    let mut display = PCD8544::new(pcd_clk, pcd_din, pcd_dc, pcd_ce, pcd_rst, pcd_light)
        .expect("Infallible cannot fail");

    display.reset().expect("Infallible cannot fail");
    writeln!(display, "Hello World").unwrap();
}
