//! Dygma Focus wireless battery helpers.

use dygma_focus::errors::FocusError;
use dygma_focus::Focus;
use std::fmt;
use std::thread;
use std::time::Duration;

/// Snapshot of wireless battery state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryLevels {
    pub left: u8,
    pub right: u8,
    pub left_status: Option<u8>,
    pub right_status: Option<u8>,
}

impl BatteryLevels {
    /// Title suitable for a Stream Deck key (two lines).
    pub fn title(&self) -> String {
        format!("L{}%\nR{}%", self.left, self.right)
    }
}

impl fmt::Display for BatteryLevels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "L{}% (s{}) R{}% (s{})",
            self.left,
            status_label(self.left_status),
            self.right,
            status_label(self.right_status),
        )
    }
}

fn status_label(status: Option<u8>) -> String {
    status.map_or_else(|| "?".to_string(), |s| s.to_string())
}

/// Open Focus, optionally force-read, query L/R levels (+ status), close.
///
/// Bazecor (or any other Focus client) must not hold the COM port.
pub fn read_battery(force: bool, force_wait: Duration) -> Result<BatteryLevels, FocusError> {
    let mut focus = Focus::new_first_available()?;

    if force {
        // force_read writes the command but does not consume the serial
        // end-of-response (`.`). Drain it before further commands or the
        // next parse sees an empty payload.
        focus.wireless_battery_force_read()?;
        focus.read_string()?;
        // Neuron needs a moment to query both sides over RF after the ack.
        thread::sleep(force_wait);
    }

    let left = focus.wireless_battery_level_left_get()?;
    let right = focus.wireless_battery_level_right_get()?;

    let left_status = match focus.wireless_battery_status_left_get() {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::debug!(error = %e, "left battery status unavailable");
            None
        }
    };
    let right_status = match focus.wireless_battery_status_right_get() {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::debug!(error = %e, "right battery status unavailable");
            None
        }
    };

    Ok(BatteryLevels {
        left,
        right,
        left_status,
        right_status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(left: u8, right: u8) -> BatteryLevels {
        BatteryLevels {
            left,
            right,
            left_status: Some(0),
            right_status: Some(0),
        }
    }

    #[test]
    fn title_format() {
        assert_eq!(sample(100, 40).title(), "L100%\nR40%");
        assert_eq!(sample(0, 0).title(), "L0%\nR0%");
        assert_eq!(sample(9, 99).title(), "L9%\nR99%");
    }

    #[test]
    fn display_includes_status() {
        let levels = BatteryLevels {
            left: 80,
            right: 20,
            left_status: Some(1),
            right_status: None,
        };
        let s = levels.to_string();
        assert!(s.contains("L80% (s1)"), "{s}");
        assert!(s.contains("R20% (s?)"), "{s}");
    }

    #[test]
    fn display_both_status_unknown() {
        let levels = BatteryLevels {
            left: 50,
            right: 50,
            left_status: None,
            right_status: None,
        };
        assert_eq!(levels.to_string(), "L50% (s?) R50% (s?)");
    }

    #[test]
    fn equality_and_clone() {
        let a = sample(10, 20);
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, sample(11, 20));
    }
}
