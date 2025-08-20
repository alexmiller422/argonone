use std::convert::Infallible;

use futures::sink::unfold;
use futures::stream::{StreamExt};
use log::{error, info, LevelFilter};
use systemd_journal_logger::{connected_to_journal, JournalLog};
use system_shutdown::{self, ShutdownResult};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

use crate::power_button_stream::{Error, PowerButtonEvent};

mod edge_stream;
mod fan_control;
mod power_button_stream;
mod temperature_stream;

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

fn power_button_pipeline() -> impl Future<Output = Result<(), Infallible>> {
    let events_stream = power_button_stream::open().unwrap();
    let events_sink = unfold((), |_, event| {
        async { 
            handle_event(event).unwrap();
            Ok::<_, Infallible>(())
        }
    });
    let events_pipeline = events_stream
        .map(|event| Ok::<PowerButtonEvent, Infallible>(event))
        .forward(events_sink);
    
    events_pipeline
}

fn temp_pipeline() -> impl Future<Output = Result<(), Infallible>> {
    let temp_stream = temperature_stream::open();
    let temp_pipeline = temp_stream.forward(fan_control::temp_sink());

    temp_pipeline
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_logging();
    let mut signals = signal(SignalKind::interrupt())?;

    select! {
        Some(_) = signals.recv() => { info!("Received signal. Terminating") },
        _ = power_button_pipeline() => { error!("Power button events pipeline unexpectedly completed.") },
        _ = temp_pipeline() => { error!("Temperature pipeline unexpected completed") }
    }


    Ok(())
}
