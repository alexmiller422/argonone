use std::{fs::File, io::{self, BufRead, Read}, path::Path, time::Duration};

use futures::{never::Never, Stream, StreamExt};
use tokio::time::{Instant, interval};
use tokio_stream::wrappers::IntervalStream;

pub struct Temperatures {
    pub instant: Instant,
    pub cpu_temp: f32,
}


fn get_temperatures(instant: Instant) -> Result<Temperatures, Never> {
    let mut temp_str = String::new();

    let file = File::open(Path::new("/sys/class/thermal/thermal_zone0/temp")).unwrap();
    let mut reader = io::BufReader::new(file); 
    reader.read_line(&mut temp_str).unwrap();

    let cpu_temp: f32 = temp_str.trim().parse::<f32>().unwrap() / 1000.0;

    Ok(Temperatures { instant, cpu_temp })
}

pub fn open() -> impl Stream<Item = Result<Temperatures,Never>> { 
    let interval = interval(Duration::from_secs(30));

    let temp_stream = IntervalStream::new(interval)
        .map(get_temperatures);

    temp_stream
}