use std::{convert::Infallible, future::ready};

use futures::{never::Never, sink::{unfold, Sink}};
use i2c_linux::I2c;
use log::debug;
use crate::temperature_stream::Temperatures;

fn get_fan_speed(temp: f32) -> u8 {
    if temp >= 65.0 {
        0x64
    }
    else if temp >= 60.0 {

        0x37
    }
    else if temp >= 55.0 { 
        0x1e
    }
    else {
        0x00
    }
}

fn set_fan_speed(speed: u8) {
    let mut i2c = I2c::from_path("/dev/i2c-1").unwrap();

    i2c.smbus_set_slave_address(0x1a, false).unwrap();
    i2c.smbus_write_byte(speed).unwrap();

    debug!("Set fan speed to {} %", speed);
}

pub fn temp_sink() -> impl Sink<Temperatures, Error = Infallible> {
    let mut current: u8 = 0;
    let mut next: Option<u8> = None;

    let sink = unfold((), move |_, temps: Temperatures| {
        let suggested = get_fan_speed(temps.cpu_temp);
        debug!("Current temperature = {}℃. Current fan speed = {}%. Suggested fan speed = {}%", temps.cpu_temp, current, suggested);

        if suggested > current {
            set_fan_speed(suggested);
            current = suggested;
            next = None;
        }
        else {
            match next {
                None => {next = Some(suggested)},
                Some(prev_suggestion) if prev_suggestion != current && prev_suggestion >= suggested => {
                    set_fan_speed(prev_suggestion);
                    current = prev_suggestion;
                    next = Some(suggested);
                },
                Some(_) => {
                    next = Some(suggested);
                }                
            }
        }
        ready(Ok::<_, Never>(()))
    });

    sink
}