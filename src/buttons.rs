use sysfs_gpio::{Direction, Edge, Pin};

// this is the same as Magnet now but maybe I will want
// eg debounce beheviour or other mode behaviour in here?

pub struct PushButton {
    poller: sysfs_gpio::PinPoller,
}

impl PushButton {
    pub fn new() -> std::result::Result<PushButton, sysfs_gpio::Error> {
        let poller = setup_buttons()?;
        Ok(PushButton { poller: poller })
    }

    pub fn pulsed(&mut self) -> bool {
        match self.poller.poll(0) {
            Ok(Some(value)) => {
                println!("Poll got a value {}", value);
                true
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
        Some(value) => println!("Poll got first value {} - ignoring", value),
        None => (),
    }
    println!("Done configuring button(s)");

    Ok(poller)
}
