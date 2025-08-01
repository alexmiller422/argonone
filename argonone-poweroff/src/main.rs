use i2c_linux::I2c;

fn set_poweroff() -> Result<(), std::io::Error> {
    let mut i2c = I2c::from_path("/dev/i2c-1")?;

    i2c.smbus_set_slave_address(0x1a, false)?;
    i2c.smbus_write_byte(0xff)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    println!("Sending Power off command to Argon One MCU");

    set_poweroff()?;


    println!("Sent Power off command to Argon One MCU");

    Ok(())
}
