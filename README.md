# Argon One Tools

This repository contains Rust source code and associated Linux configuration files for use with the Argon One cases,
provided by Argon 40.


## Binaries

### [argonone-poweroff](./argonone-poweroff)

A simple Rust binary that sends the byte `0xff` to address `0x1a` of the device `/dev/i2c-1`. After the Argon One MCU
receives this if will cut power to Raspberry Pi when GPIO pin 14 goes low, or after ~17 seconds.

If `enable_uart=1` and `dtparam=krnbt=off` are added to the kernel's `/boot/config.txt`, GPIO pin 14 is managed as a
serial device, and will go low as the Linux kernel shuts down.

Users invoking this binary need write access to I2C device `/dev/i2c-1`.


### [argononed](./argononed/)

A daemon that monitors GPIO pin 4 for power button events, and reboots or shuts down the system based on the events received.

**TODO:** Add fan control functionality

The user invoking this needs read access to the the GPIO device `/dev/gpiochip0` and permissions to shutdown and reboot the system.


## Additional Files

### [argonone-poweroff.service](./argonone-poweroff.service)

Systemd service file for running the [argonone-poweroff](#argonone-poweroff) binary.

Executes the binary as the argonone user and expects the binary to be copied to `/usr/local/bin/argonone-poweroff`.


### [argononed.service](./argononed.service)

Systemd service file for running the [argononed](#argononed) binary.

Executes the binary as the argonone user and expects it be located at `/usr/local/bin/argononed`.


###  [polkit.rules](./pollkit.rules)

Polkit rules allowing the [argononed.service](#argononedservice) to reboot or shutdown the system.


### [sysusers.conf](./sysusers.conf)

systemd-sysusers file to create the `argonone` user and add it to the `i2c` and `gpio` groups.

