use std::time::{Duration, Instant};
use sysfs_gpio::{Direction, Edge, Pin};

// debounce duration is between main calling "pulsed", not when
// the system detects actual rising edges.
const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

// this is the same as Magnet now but maybe I will want
// eg debounce beheviour or other mode behaviour in here?

pub struct PushButton {
    poller: sysfs_gpio::PinPoller,
    debounce_start: Instant,
}

impl PushButton {
    pub fn new() -> std::result::Result<PushButton, sysfs_gpio::Error> {
        let poller = setup_buttons()?;
        Ok(PushButton {
            poller,
            debounce_start: Instant::now(),
        })
    }

    pub fn pulsed(&mut self) -> bool {
        match self.poller.poll(0) {
            Ok(Some(value)) => {
                if Instant::now() - self.debounce_start > DEBOUNCE_DURATION {
                    println!("Poll got a value {value} which will be ignored");
                    self.debounce_start = Instant::now();
                    true
                } else {
                    println!("Debounce");
                    false
                }
            }
            _ => false,
        }
    }
}

fn setup_buttons() -> std::result::Result<sysfs_gpio::PinPoller, sysfs_gpio::Error> {
    println!("Configuring push button(s)");
    let pin = Pin::new(12); // other button is 13
    pin.export()?;
    pin.set_direction(Direction::In)?;
    pin.set_edge(Edge::RisingEdge)?;
    let mut poller: sysfs_gpio::PinPoller = pin.get_poller()?;
    println!("Making first pin poll");
    match poller.poll(0)? {
        Some(value) => println!("Poll got first value {value} - ignoring"),
        None => (),
    }
    println!("Done configuring button(s)");

    Ok(poller)
}
