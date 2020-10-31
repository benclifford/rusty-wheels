use sysfs_gpio::{Direction, Edge, Pin};

pub struct Magnet {
    poller: sysfs_gpio::PinPoller,
}

impl Magnet {
    pub fn new() -> std::result::Result<Magnet, sysfs_gpio::Error> {
        let poller = setup_magnet()?;
        Ok(Magnet { poller: poller })
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

fn setup_magnet() -> std::result::Result<sysfs_gpio::PinPoller, sysfs_gpio::Error> {
    println!("Configuring magnet");
    let pin = Pin::new(27);
    pin.export()?;
    pin.set_direction(Direction::In)?;
    pin.set_edge(Edge::RisingEdge)?;
    let mut poller: sysfs_gpio::PinPoller = pin.get_poller()?;
    println!("Making first pin poll");
    match poller.poll(0)? {
        Some(value) => println!("Poll got first value {} - ignoring", value),
        None => (),
    }
    println!("Done configuring magnet");

    Ok(poller)
}
