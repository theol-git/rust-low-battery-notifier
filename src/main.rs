extern crate starship_battery as battery;

use std::io;
use std::thread;
use std::time::Duration;

use clap::Parser;
use notify_rust::Notification;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Battery Percentage Threshold to trigger notification
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u8).range(1..=100))]
    notification_threshold: u8,

    /// Delay to wait before rerunning the battery check
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
    delay_after_notification_close: u64,
}

#[cfg(all(unix, not(target_os = "macos")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let notification_threshold = (cli.notification_threshold / 100) as f32;

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

        if battery.state_of_charge().value < notification_threshold {
            Notification::new()
                .body("Battery low! Please charge.")
                .icon("dialog-warning")
                .urgency(notify_rust::Urgency::Critical)
                .show()
                .unwrap()
                .on_close(|reason: notify_rust::CloseReason| {
                    println!("Low battery notification closed, reason: {reason:?}");
                    thread::sleep(Duration::from_secs(cli.delay_after_notification_close));
                });
            continue;
        }

        thread::sleep(Duration::from_secs(1));
    }
}
