use std::{
    io::{self, Write},
    thread::sleep,
    time::Duration,
    vec,
};

use ringbuf::traits::{Consumer, Split};

mod spectrum;

fn main() {
    let mut socket = spectrum::Client::connect().unwrap();

    let monitors = socket.get_monitors().unwrap();
    let mon = monitors.first().unwrap();

    let bars = 40;
    let cava = spectrum::Cava::new(
        bars,
        mon.sample_spec.sample_rate,
        mon.channel_map.num_channels() as i32,
        1,
        0.77,
        80,
        16000,
    )
    .unwrap();

    let fps = 60;
    let frame_size =
        mon.sample_spec.sample_rate as usize / fps * mon.channel_map.num_channels() as usize;

    let rb = spectrum::HeapRb::<f32>::new(frame_size * 4);
    let (producer, mut consumer) = rb.split();

    let _handle = socket.record_from_source(mon, 256, producer).unwrap();

    let out = io::stdout();
    let mut out = out.lock();
    let levels = [
        ['⠀', '⢀', '⢠', '⢰', '⢸'],
        ['⡀', '⣀', '⣠', '⣰', '⣸'],
        ['⡄', '⣄', '⣤', '⣴', '⣼'],
        ['⡆', '⣆', '⣦', '⣶', '⣾'],
        ['⡇', '⣇', '⣧', '⣷', '⣿'],
    ];

    let mut buffer = vec![0f32; frame_size];
    let mut cava_out = vec![0f64; bars as usize * mon.channel_map.num_channels() as usize];

    loop {
        sleep(Duration::new(0, 1_000_000_000u32 / fps as u32));

        let _ = consumer.pop_slice(&mut buffer);
        cava.execute(&mut buffer, &mut cava_out);

        let levels = spectrum::levels(&mut cava_out, 4, -20.0)
            .chunks(2)
            .map(|x| levels[x[0] as usize][x[1] as usize])
            .collect::<String>();

        writeln!(out, "{}", levels).unwrap();
    }
}
