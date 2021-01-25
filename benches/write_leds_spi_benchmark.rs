use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

use rusty_wheels::leds::WheelLEDs;

pub fn criterion_benchmark(c: &mut Criterion) {

    let mut wheel_leds = WheelLEDs::new();

    c.bench_function("wheel_leds.show()", |b| b.iter(|| wheel_leds.show() ));
}

criterion_group!{
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(15));
    targets = criterion_benchmark
}

criterion_main!(benches);
