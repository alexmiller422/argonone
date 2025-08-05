use futures::stream::{StreamExt};
use log::{error, info, LevelFilter};
use systemd_journal_logger::{connected_to_journal, JournalLog};
use system_shutdown::{self, ShutdownResult};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

use crate::power_button_stream::{Error, PowerButtonEvent, PowerButtonStream};

mod edge_stream;
mod power_button_stream;

fn reboot() -> ShutdownResult {
    info!("Reboot event received");
    info!("Triggering system reboot");

    system_shutdown::reboot()
        .inspect(|_| info!("System reboot triggered"))
        .inspect_err(|e| error!("System reboot error: {}", e))
}

fn shutdown() -> ShutdownResult {
    info!("Shutdown event received");
    info!("Triggering system shutdown");

    system_shutdown::shutdown()
        .inspect(|_| info!("System shutdown triggered"))
        .inspect_err(|e| error!("System shutdown error: {}", e))
}


fn handle_event(event: PowerButtonEvent) -> ShutdownResult{
    match event {
        PowerButtonEvent::Reboot => reboot(),
        PowerButtonEvent::Poweroff => shutdown(),
        PowerButtonEvent::Unknown(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown power button event"))
    }
}

fn setup_logging() {
    if connected_to_journal() {
        JournalLog::new()
            .unwrap()
            .install()
            .unwrap();

        log::set_max_level(LevelFilter::Info)
    }
    else {
       env_logger::init(); 
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_logging();

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
