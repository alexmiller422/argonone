use std::path::Path;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::Stream;
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

pub struct EdgeStream {
    async_fd: AsyncFd<Request>,
    events: Vec<Event>
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

impl EdgeStream {
    pub fn open() -> Result<Self, Error> {
        let chip = libgpiod::chip::Chip::open(&Path::new("/dev/gpiochip0"))?;

        let mut line_settings = libgpiod::line::Settings::new()?;

        line_settings
            .set_direction(Direction::Input)?
            .set_edge_detection(Option::Some(Edge::Both))?;

        let mut line_config = libgpiod::line::Config::new()?;
        line_config.add_line_settings(&[4], line_settings)?;

        let request = chip.request_lines(None, &line_config)?;

        let async_fd = AsyncFd::new(request)?;

        Ok(EdgeStream {
            async_fd,
            events: Vec::new()
        })
    }

    fn pop_event(self: &mut Self) -> Event {
        self.events.pop().unwrap()
    }

    fn has_event(self: &mut Self) -> bool {
        !self.events.is_empty()
    }

    fn extend<T: IntoIterator<Item = Event>>(self: &mut Self, events: T) {
        self.events.extend(events);
    }

    fn try_read(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Event>> {        
        self.async_fd.poll_read_ready(cx)
            .map_ok(|mut guard| {
                let events = read_events(guard.get_inner());
                guard.clear_ready_matching(Ready::READABLE);
                events
            }).map(|result| {
                match result {
                    Ok(events) => {
                        self.extend(events);
                        Some(self.pop_event())
                    },
                    Err(_) => None
                }
            })
    }

}

impl Stream for EdgeStream {
    type Item = Event;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.has_event() {
            Poll::Ready(Some(self.pop_event()))
        }
        else {
            self.try_read(cx)
        }

    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}