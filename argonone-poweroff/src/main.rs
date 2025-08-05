use i2c_linux::I2c;
use log::{error, info};
use systemd_journal_logger::{connected_to_journal, JournalLog};

fn setup_logging() {
    if connected_to_journal() {
        JournalLog::new()
            .unwrap()
            .install()
            .unwrap();
        log::set_max_level(log::LevelFilter::Info);
    }
    else {
       env_logger::init(); 
    }
}

fn send_poweroff_cmd() -> Result<(), std::io::Error> {
    let mut i2c = I2c::from_path("/dev/i2c-1")?;

    i2c.smbus_set_slave_address(0x1a, false)?;
    i2c.smbus_write_byte(0xff)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    setup_logging();

    info!("Sending Power off command to Argon One MCU");

    send_poweroff_cmd()
        .inspect(|_| info!("Poweroff command sent."))
        .inspect_err(|e| error!("Error sending poweroff command: {}", e))?;

    Ok(())
}
