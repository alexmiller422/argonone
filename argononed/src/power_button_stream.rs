use std::fmt::Debug;
use std::future::{ready,Ready};
use std::time::Duration;

use futures::stream::{Stream, StreamExt};
use libgpiod::line::EdgeKind;
use libgpiod::request::Event;

use crate::edge_stream;

pub type Error = crate::edge_stream::Error;

#[derive(Debug)]
pub enum PowerButtonEvent {
    Reboot,
    Poweroff,
    Unknown(Duration)    
}

impl PowerButtonEvent {
    fn new(pulse_length: Duration) -> PowerButtonEvent {
        match pulse_length.as_millis() {
            20..30 => PowerButtonEvent::Reboot,
            40..50 => PowerButtonEvent::Poweroff,
            _ => PowerButtonEvent::Unknown(pulse_length)
        }
    }
}


fn create_converter() ->  impl FnMut(Event) -> Ready<Option<PowerButtonEvent>> {
    let mut pulse_start: Option<Duration> = None;

    let mut convert = move |edge_event: Event| match (edge_event.event_type().unwrap(), pulse_start) {
        (EdgeKind::Rising, _) => {
            pulse_start = Some(edge_event.timestamp());
            None
        }
        (EdgeKind::Falling, Some(start_time)) => Some(PowerButtonEvent::new(edge_event.timestamp() - start_time)),
        (EdgeKind::Falling, None) => unreachable!()
    };
 
    move |edge_event: Event| ready(convert(edge_event))
}

pub fn open() -> Result<impl Stream<Item = PowerButtonEvent>, Error>{
    let edge_stream = edge_stream::open()?;


    Ok(edge_stream.filter_map(create_converter()))
}
