//! Alarms and the countdown timer. They run in the app process (not in the
//! web view) so they still ring while the window is hidden in the tray.

use chrono::{Datelike, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    #[default]
    Default,
    Information,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alarm {
    pub id: String,
    pub label: String,
    pub hour: u8,
    pub minute: u8,
    /// Days it repeats on, 0 = Sunday ... 6 = Saturday. Empty = rings once.
    #[serde(default)]
    pub days: Vec<u8>,
    pub enabled: bool,
    #[serde(default)]
    pub tone: Tone,
}

impl Alarm {
    pub fn validate(&self) -> Result<()> {
        if self.hour > 23 || self.minute > 59 {
            return Err(Error::Invalid("Alarm time must be between 00:00 and 23:59.".into()));
        }
        if self.days.iter().any(|d| *d > 6) {
            return Err(Error::Invalid("Alarm days must be 0 (Sunday) to 6 (Saturday).".into()));
        }
        if self.label.chars().count() > 60 {
            return Err(Error::Invalid("Alarm label can be at most 60 characters.".into()));
        }
        if self.id.trim().is_empty() {
            return Err(Error::Invalid("Alarm id is missing.".into()));
        }
        Ok(())
    }

    pub fn is_due(&self, now: NaiveDateTime) -> bool {
        let weekday = now.weekday().num_days_from_sunday() as u8;
        self.enabled
            && u32::from(self.hour) == now.hour()
            && u32::from(self.minute) == now.minute()
            && (self.days.is_empty() || self.days.contains(&weekday))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Countdown {
    pub label: String,
    /// Unix time in milliseconds.
    pub ends_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Fired {
    Alarm { id: String, label: String, tone: Tone },
    Countdown { label: String },
}

/// Decides what rings on each tick. Call [`Clock::tick`] about once a second.
#[derive(Debug, Default)]
pub struct Clock {
    last_minute: Option<(i32, u32, u32)>,
    pub countdown: Option<Countdown>,
}

impl Clock {
    /// Returns what fired. One-time alarms are switched off after ringing;
    /// the returned flag says whether `alarms` changed and should be saved.
    pub fn tick(&mut self, alarms: &mut [Alarm], local_now: NaiveDateTime, unix_ms: u64) -> (Vec<Fired>, bool) {
        let mut fired = Vec::new();
        let mut changed = false;
        let minute_key = (local_now.num_days_from_ce(), local_now.hour(), local_now.minute());
        if self.last_minute != Some(minute_key) {
            // Skip the first tick so starting the app at 07:00:30 doesn't re-ring a 07:00 alarm.
            if self.last_minute.is_some() {
                for alarm in alarms.iter_mut().filter(|a| a.is_due(local_now)) {
                    fired.push(Fired::Alarm { id: alarm.id.clone(), label: alarm.label.clone(), tone: alarm.tone });
                    if alarm.days.is_empty() {
                        alarm.enabled = false;
                        changed = true;
                    }
                }
            }
            self.last_minute = Some(minute_key);
        }
        if let Some(countdown) = &self.countdown {
            if unix_ms >= countdown.ends_at_ms {
                fired.push(Fired::Countdown { label: countdown.label.clone() });
                self.countdown = None;
            }
        }
        (fired, changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn at(day: u32, h: u32, m: u32, s: u32) -> NaiveDateTime {
        // 2026-10-11 is a Sunday.
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap().and_hms_opt(h, m, s).unwrap()
    }

    fn alarm(days: Vec<u8>) -> Alarm {
        Alarm { id: "a".into(), label: "Wake".into(), hour: 7, minute: 0, days, enabled: true, tone: Tone::Default }
    }

    #[test]
    fn rings_once_per_minute_and_disables_one_shots() {
        let mut clock = Clock::default();
        let mut alarms = vec![alarm(vec![])];
        assert!(clock.tick(&mut alarms, at(12, 6, 59, 58), 0).0.is_empty());
        let (fired, changed) = clock.tick(&mut alarms, at(12, 7, 0, 0), 0);
        assert_eq!(fired.len(), 1);
        assert!(changed);
        assert!(!alarms[0].enabled);
        assert!(clock.tick(&mut alarms, at(12, 7, 0, 1), 0).0.is_empty(), "no double ring");
    }

    #[test]
    fn does_not_ring_on_first_tick() {
        let mut clock = Clock::default();
        let mut alarms = vec![alarm(vec![])];
        assert!(clock.tick(&mut alarms, at(12, 7, 0, 30), 0).0.is_empty());
        assert!(alarms[0].enabled);
    }

    #[test]
    fn repeating_alarms_respect_days() {
        let mut clock = Clock::default();
        let mut alarms = vec![alarm(vec![1, 2, 3, 4, 5])]; // weekdays
        clock.tick(&mut alarms, at(11, 6, 59, 0), 0); // Sunday
        assert!(clock.tick(&mut alarms, at(11, 7, 0, 0), 0).0.is_empty(), "Sunday is skipped");
        clock.tick(&mut alarms, at(12, 6, 59, 0), 0); // Monday
        let (fired, changed) = clock.tick(&mut alarms, at(12, 7, 0, 0), 0);
        assert_eq!(fired.len(), 1);
        assert!(!changed && alarms[0].enabled, "repeating alarms stay on");
    }

    #[test]
    fn countdown_fires_once() {
        let mut clock =
            Clock { countdown: Some(Countdown { label: "Tea".into(), ends_at_ms: 1_000 }), ..Default::default() };
        let mut alarms = vec![];
        assert!(clock.tick(&mut alarms, at(12, 8, 0, 0), 999).0.is_empty());
        assert_eq!(clock.tick(&mut alarms, at(12, 8, 0, 1), 1_000).0, vec![Fired::Countdown { label: "Tea".into() }]);
        assert!(clock.countdown.is_none());
    }

    #[test]
    fn validation() {
        assert!(alarm(vec![]).validate().is_ok());
        assert!(Alarm { hour: 24, ..alarm(vec![]) }.validate().is_err());
        assert!(alarm(vec![7]).validate().is_err());
    }
}
