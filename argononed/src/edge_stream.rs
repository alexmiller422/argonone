use std::path::Path;
use std::time::Duration;

use futures::{Stream, StreamExt};
use futures::stream::{iter, poll_fn};
use libgpiod::line::{Direction, Edge};
use libgpiod::request::{Buffer, Event, Request};
use tokio::io::Ready;
use tokio::io::unix::AsyncFd;

#[derive(Debug)]
pub enum Error {
    StdIoError(std::io::Error),
    LgpiodError(libgpiod::Error)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::StdIoError(value)
    }
}

impl From<libgpiod::Error> for Error {
    fn from(value: libgpiod::Error) -> Self {
        Error::LgpiodError(value)
    }
}

fn read_events(request: &Request) -> Vec<Event> {
    let mut events = Vec::new();

    while request.wait_edge_events(Some(Duration::new(0,0))).unwrap_or(false) == true {
        let mut buffer = Buffer::new(0).unwrap();

        let new_events = request.read_edge_events(&mut buffer).unwrap();
        for event in new_events {
            match event {
                Ok(event) => events.push(Event::try_clone(event).unwrap()),
                Err(error) => panic!("Error reading edge event: {}", error)
            }
        }
    }

    events
}

pub fn open() -> Result<impl Stream<Item = Event>, Error> {
    let chip = libgpiod::chip::Chip::open(&Path::new("/dev/gpiochip0"))?;

    let mut line_settings = libgpiod::line::Settings::new()?;

    line_settings
        .set_direction(Direction::Input)?
        .set_edge_detection(Option::Some(Edge::Both))?;

    let mut line_config = libgpiod::line::Config::new()?;
    line_config.add_line_settings(&[4], line_settings)?;

    let request = chip.request_lines(None, &line_config)?;

    let async_fd = AsyncFd::new(request)?;

    let base = poll_fn(move |cx| {
        async_fd.poll_read_ready(cx).map(|result| {
            let mut guard = result.expect("Unexpected error polling GPIO line");
            let events = read_events(guard.get_inner());
            guard.clear_ready_matching(Ready::READABLE);
            Some(iter(events))
        })
    });

    let three = base.flatten();

    Ok(three)
}
