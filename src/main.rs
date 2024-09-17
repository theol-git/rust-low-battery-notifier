extern crate starship_battery as battery;

use std::io;
use std::thread;
use std::time::Duration;

use notify_rust::Notification;

#[cfg(all(unix, not(target_os = "macos")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    const NOTIFICATION_THRESHOLD: f32 = 0.33;
    //const NOTIFICATION_THRESHOLD: f32 = 0.05;

    let manager = battery::Manager::new()?;
    let mut battery = match manager.batteries()?.next() {
        Some(Ok(battery)) => battery,
        Some(Err(e)) => {
            eprintln!("Unable to access battery information");
            return Err(Box::new(e));
        }
        None => {
            eprintln!("Unable to find any batteries");
            return Err(io::Error::from(io::ErrorKind::NotFound).into());
        }
    };

    loop {
        manager.refresh(&mut battery)?;

        if battery.state_of_charge().value < NOTIFICATION_THRESHOLD {
            Notification::new()
                .body("Battery low! Please charge.")
                .icon("dialog-warning")
                .urgency(notify_rust::Urgency::Critical)
                .show()
                .unwrap()
                .on_close(|| thread::sleep(Duration::from_secs(30)));
            continue;
        }

        thread::sleep(Duration::from_secs(1));
    }
}
