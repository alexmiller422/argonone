use futures::stream::{StreamExt};
use system_shutdown::{reboot,shutdown, ShutdownResult};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

use crate::power_button_stream::{Error, PowerButtonEvent, PowerButtonStream};

mod edge_stream;
mod power_button_stream;

fn handle_event(event: PowerButtonEvent) -> ShutdownResult{
    match event {
        PowerButtonEvent::Reboot => reboot(),
        PowerButtonEvent::Poweroff => shutdown(),
        PowerButtonEvent::Unknown(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown power button event"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut events_stream = PowerButtonStream::open()?;
    let mut signals = signal(SignalKind::interrupt())?;

    loop {
        select! {
            Some(_) = signals.recv() => { break },
            Some(event) = events_stream.next() => {handle_event(event).unwrap();}
        }
    }

    Ok(())
}
