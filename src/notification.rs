use color_eyre::Result;
use notify_rust::{Notification, Urgency};

pub fn notify(summary: &str, body: &str, urgent: bool) -> Result<()> {
    let urgency = if urgent {
        Urgency::Critical
    } else {
        Urgency::Low
    };
    Notification::new()
        .summary(summary)
        .body(body)
        .urgency(urgency)
        .show()?;
    Ok(())
}
