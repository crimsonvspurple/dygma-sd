//! Plugin state machine: action instances, key titles, battery polling.

use crate::battery::{self, BatteryLevels};
use crate::error::PluginError;
use futures::SinkExt;
use futures::stream::SplitSink;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use streamdeck_rs::{Message, MessageOut, StreamDeckSocket, Target, TitlePayload};
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, info, warn};

pub const ACTION_UUID: &str = "com.red.eminence.dygma.battery.levels";
pub const DEFAULT_POLL_SECS: u64 = 60;
pub const MIN_POLL_SECS: u64 = 15;
pub const MAX_POLL_SECS: u64 = 600;
pub const FORCE_WAIT: Duration = Duration::from_secs(2);

/// Per-action settings (property inspector).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ActionSettings {
    /// Poll interval in seconds (clamped to 15..=600).
    #[serde(default = "default_poll_interval_secs")]
    poll_interval_secs: u64,
}

fn default_poll_interval_secs() -> u64 {
    DEFAULT_POLL_SECS
}

impl Default for ActionSettings {
    fn default() -> Self {
        Self {
            poll_interval_secs: DEFAULT_POLL_SECS,
        }
    }
}

impl ActionSettings {
    pub fn poll_interval(&self) -> Duration {
        let secs = self
            .poll_interval_secs
            .clamp(MIN_POLL_SECS, MAX_POLL_SECS);
        Duration::from_secs(secs)
    }

    pub fn poll_interval_secs(&self) -> u64 {
        self.poll_interval_secs.clamp(MIN_POLL_SECS, MAX_POLL_SECS)
    }
}

/// Empty global / property-inspector message types (unused).
pub type GlobalSettings = ();
pub type PiIn = ();
pub type PiOut = ();

pub type SdSocket = StreamDeckSocket<GlobalSettings, ActionSettings, PiIn, PiOut>;
pub type OutMsg = MessageOut<GlobalSettings, ActionSettings, PiOut>;
pub type SdSink = SplitSink<SdSocket, OutMsg>;

/// What the Stream Deck key should display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyTitle {
    Loading,
    Levels(BatteryLevels),
    Error,
}

impl KeyTitle {
    pub fn as_title(&self) -> String {
        match self {
            Self::Loading => "…".to_string(),
            Self::Levels(levels) => levels.title(),
            Self::Error => "ERR\nCOM".to_string(),
        }
    }
}

/// Result of a background battery read.
#[derive(Debug)]
pub struct BatteryOutcome {
    pub result: Result<BatteryLevels, String>,
    pub force: bool,
}

struct ActionInstance {
    settings: ActionSettings,
}

/// Owns action instances and the shared key view model.
pub struct Plugin {
    actions: HashMap<String, ActionInstance>,
    view: KeyTitle,
    last_poll: Option<Instant>,
    battery_in_flight: bool,
}

impl Plugin {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            view: KeyTitle::Loading,
            last_poll: None,
            battery_in_flight: false,
        }
    }

    pub fn has_visible_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    pub fn poll_interval(&self) -> Duration {
        self.actions
            .values()
            .map(|a| a.settings.poll_interval())
            .min()
            .unwrap_or(Duration::from_secs(DEFAULT_POLL_SECS))
    }

    pub fn should_poll(&self) -> bool {
        if !self.has_visible_actions() || self.battery_in_flight {
            return false;
        }
        self.last_poll
            .is_none_or(|t| t.elapsed() >= self.poll_interval())
    }

    pub fn request_battery(&mut self, bat_tx: &mpsc::Sender<BatteryOutcome>, force: bool) {
        if self.battery_in_flight {
            return;
        }
        self.battery_in_flight = true;
        let tx = bat_tx.clone();
        tokio::task::spawn_blocking(move || {
            let result = battery::read_battery(force, FORCE_WAIT).map_err(|e| e.to_string());
            let _ = tx.blocking_send(BatteryOutcome { result, force });
        });
    }

    pub async fn handle_message(
        &mut self,
        sink: &mut SdSink,
        bat_tx: &mpsc::Sender<BatteryOutcome>,
        msg: Message<GlobalSettings, ActionSettings, PiIn>,
    ) -> Result<(), PluginError> {
        match msg {
            Message::WillAppear {
                action,
                context,
                payload,
                ..
            } => {
                if action != ACTION_UUID {
                    debug!(%action, "ignoring unknown action");
                    return Ok(());
                }
                info!(%context, "willAppear");
                self.actions.insert(
                    context.clone(),
                    ActionInstance {
                        settings: payload.settings,
                    },
                );
                set_title(sink, &context, &self.view.as_title()).await?;
                self.request_battery(bat_tx, true);
            }
            Message::WillDisappear { context, .. } => {
                info!(%context, "willDisappear");
                self.actions.remove(&context);
            }
            Message::KeyDown { context, .. } => {
                info!(%context, "keyDown → force refresh");
                self.view = KeyTitle::Loading;
                set_title(sink, &context, &self.view.as_title()).await?;
                self.request_battery(bat_tx, true);
            }
            Message::DidReceiveSettings {
                context, payload, ..
            } => {
                if let Some(inst) = self.actions.get_mut(&context) {
                    inst.settings = payload.settings;
                    info!(
                        %context,
                        poll = inst.settings.poll_interval_secs(),
                        "settings updated"
                    );
                }
            }
            Message::Unknown => {
                debug!("unknown Stream Deck event (safe to ignore)");
            }
            other => {
                debug!(?other, "unhandled event");
            }
        }
        Ok(())
    }

    pub async fn on_battery_result(
        &mut self,
        sink: &mut SdSink,
        outcome: BatteryOutcome,
    ) -> Result<(), PluginError> {
        let show_alert = self.apply_battery_outcome(outcome);

        if show_alert {
            if let Some(ctx) = self.actions.keys().next() {
                if let Err(e) = sink
                    .send(OutMsg::ShowAlert {
                        context: ctx.clone(),
                    })
                    .await
                {
                    warn!(error = ?e, "showAlert failed");
                }
            }
        }

        self.push_titles(sink).await
    }

    /// Update view / poll bookkeeping from a battery read (no Stream Deck I/O).
    ///
    /// Returns `true` when the caller should flash `showAlert` on a key.
    fn apply_battery_outcome(&mut self, outcome: BatteryOutcome) -> bool {
        self.battery_in_flight = false;
        self.last_poll = Some(Instant::now());

        match outcome.result {
            Ok(levels) => {
                info!(%levels, force = outcome.force, "battery update");
                self.view = KeyTitle::Levels(levels);
                false
            }
            Err(err) => {
                warn!(%err, force = outcome.force, "battery read failed");
                self.view = KeyTitle::Error;
                true
            }
        }
    }

    async fn push_titles(&self, sink: &mut SdSink) -> Result<(), PluginError> {
        let title = self.view.as_title();
        for context in self.actions.keys() {
            set_title(sink, context, &title).await?;
        }
        Ok(())
    }
}

async fn set_title(sink: &mut SdSink, context: &str, title: &str) -> Result<(), PluginError> {
    sink.send(OutMsg::SetTitle {
        context: context.to_string(),
        payload: TitlePayload {
            title: Some(title.to_string()),
            target: Target::Both,
            state: None,
        },
    })
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels(left: u8, right: u8) -> BatteryLevels {
        BatteryLevels {
            left,
            right,
            left_status: Some(0),
            right_status: Some(0),
        }
    }

    fn settings(poll_interval_secs: u64) -> ActionSettings {
        ActionSettings { poll_interval_secs }
    }

    impl Plugin {
        fn with_action(context: &str, settings: ActionSettings) -> Self {
            let mut plugin = Self::new();
            plugin.actions.insert(
                context.to_string(),
                ActionInstance { settings },
            );
            plugin
        }
    }

    #[test]
    fn key_title_variants() {
        assert_eq!(KeyTitle::Loading.as_title(), "…");
        assert_eq!(KeyTitle::Error.as_title(), "ERR\nCOM");
        assert_eq!(
            KeyTitle::Levels(levels(100, 40)).as_title(),
            "L100%\nR40%"
        );
    }

    #[test]
    fn action_settings_clamp_poll_interval() {
        assert_eq!(settings(5).poll_interval_secs(), MIN_POLL_SECS);
        assert_eq!(settings(60).poll_interval_secs(), 60);
        assert_eq!(settings(9999).poll_interval_secs(), MAX_POLL_SECS);
        assert_eq!(
            settings(30).poll_interval(),
            Duration::from_secs(30)
        );
    }

    #[test]
    fn action_settings_serde_camel_case_and_default() {
        let parsed: ActionSettings =
            serde_json::from_str(r#"{"pollIntervalSecs": 45}"#).unwrap();
        assert_eq!(parsed.poll_interval_secs(), 45);

        let defaults: ActionSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(defaults.poll_interval_secs(), DEFAULT_POLL_SECS);

        let json = serde_json::to_string(&ActionSettings::default()).unwrap();
        assert!(json.contains("pollIntervalSecs"));
    }

    #[test]
    fn should_poll_requires_visible_action() {
        let plugin = Plugin::new();
        assert!(!plugin.has_visible_actions());
        assert!(!plugin.should_poll());
    }

    #[test]
    fn should_poll_when_visible_and_never_polled() {
        let plugin = Plugin::with_action("ctx-1", settings(60));
        assert!(plugin.has_visible_actions());
        assert!(plugin.should_poll());
    }

    #[test]
    fn should_not_poll_while_in_flight() {
        let mut plugin = Plugin::with_action("ctx-1", settings(60));
        plugin.battery_in_flight = true;
        assert!(!plugin.should_poll());
    }

    #[test]
    fn should_not_poll_immediately_after_successful_read() {
        let mut plugin = Plugin::with_action("ctx-1", settings(600));
        plugin.apply_battery_outcome(BatteryOutcome {
            result: Ok(levels(90, 50)),
            force: true,
        });
        assert!(!plugin.should_poll());
        assert_eq!(plugin.view, KeyTitle::Levels(levels(90, 50)));
    }

    #[test]
    fn poll_interval_uses_minimum_across_actions() {
        let mut plugin = Plugin::with_action("a", settings(120));
        plugin.actions.insert(
            "b".to_string(),
            ActionInstance {
                settings: settings(30),
            },
        );
        assert_eq!(plugin.poll_interval(), Duration::from_secs(30));
    }

    #[test]
    fn poll_interval_default_without_actions() {
        let plugin = Plugin::new();
        assert_eq!(
            plugin.poll_interval(),
            Duration::from_secs(DEFAULT_POLL_SECS)
        );
    }

    #[test]
    fn apply_battery_success_updates_view_and_clears_in_flight() {
        let mut plugin = Plugin::with_action("ctx", settings(60));
        plugin.battery_in_flight = true;
        let alert = plugin.apply_battery_outcome(BatteryOutcome {
            result: Ok(levels(100, 40)),
            force: true,
        });
        assert!(!alert);
        assert!(!plugin.battery_in_flight);
        assert!(plugin.last_poll.is_some());
        assert_eq!(plugin.view.as_title(), "L100%\nR40%");
    }

    #[test]
    fn apply_battery_error_sets_error_view_and_requests_alert() {
        let mut plugin = Plugin::with_action("ctx", settings(60));
        plugin.battery_in_flight = true;
        let alert = plugin.apply_battery_outcome(BatteryOutcome {
            result: Err("port busy".into()),
            force: true,
        });
        assert!(alert);
        assert!(!plugin.battery_in_flight);
        assert_eq!(plugin.view, KeyTitle::Error);
        assert_eq!(plugin.view.as_title(), "ERR\nCOM");
    }

    #[tokio::test]
    async fn request_battery_is_single_flight() {
        let mut plugin = Plugin::with_action("ctx", settings(60));
        let (tx, mut rx) = mpsc::channel::<BatteryOutcome>(4);
        plugin.request_battery(&tx, true);
        assert!(plugin.battery_in_flight);
        // Second request must be ignored while in-flight (no second task).
        plugin.request_battery(&tx, true);
        assert!(plugin.battery_in_flight);

        // First task still produces one outcome (may fail if no hardware).
        let _ = rx.recv().await;
        // Drain: ensure we did not get a second spurious send immediately.
        let second = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(second.is_err() || second.unwrap().is_none());
    }
}
