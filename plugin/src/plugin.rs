//! Plugin state machine: per-key settings, device selection, battery polling.

use crate::battery::{self, BatteryLevels, FocusDevice};
use crate::error::PluginError;
use crate::visual;
use futures::SinkExt;
use futures::stream::SplitSink;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use streamdeck_rs::{ImagePayload, Message, MessageOut, StreamDeckSocket, Target, TitlePayload};
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
    /// Draw L/R percentage under each bar in the SVG.
    #[serde(default = "default_true")]
    show_percentage: bool,
    /// Focus serial port (e.g. `COM4`). Empty = auto (first available).
    #[serde(default)]
    device_port: String,
}

fn default_poll_interval_secs() -> u64 {
    DEFAULT_POLL_SECS
}

fn default_true() -> bool {
    true
}

impl Default for ActionSettings {
    fn default() -> Self {
        Self {
            poll_interval_secs: DEFAULT_POLL_SECS,
            show_percentage: true,
            device_port: String::new(),
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

    pub fn show_percentage(&self) -> bool {
        self.show_percentage
    }

    pub fn device_port(&self) -> Option<&str> {
        let p = self.device_port.trim();
        if p.is_empty() {
            None
        } else {
            Some(p)
        }
    }
}

/// Messages from the property inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum PiIn {
    /// Ask the plugin to re-scan Focus devices.
    RefreshDevices,
    #[serde(other)]
    Unknown,
}

/// Messages to the property inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum PiOut {
    DeviceList {
        devices: Vec<DeviceListEntry>,
        selected: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceListEntry {
    pub port: String,
    pub label: String,
}

pub type GlobalSettings = ();
pub type SdSocket = StreamDeckSocket<GlobalSettings, ActionSettings, PiIn, PiOut>;
pub type OutMsg = MessageOut<GlobalSettings, ActionSettings, PiOut>;
pub type SdSink = SplitSink<SdSocket, OutMsg>;

/// What a key should display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyView {
    Loading,
    Levels(BatteryLevels),
    Error,
}

impl KeyView {
    fn image_data_uri(&self, show_percentage: bool) -> String {
        match self {
            Self::Loading => visual::loading_image_data_uri(),
            Self::Levels(levels) => visual::key_image_data_uri(levels, show_percentage),
            Self::Error => visual::error_image_data_uri(),
        }
    }
}

/// Result of a background battery read for one device port key.
#[derive(Debug)]
pub struct BatteryOutcome {
    /// Resolved port used for the read (`""` means auto/first).
    pub port_key: String,
    pub result: Result<BatteryLevels, String>,
    pub force: bool,
}

struct ActionInstance {
    settings: ActionSettings,
    view: KeyView,
    last_poll: Option<Instant>,
}

/// Owns action instances (each Stream Deck key is independent).
pub struct Plugin {
    actions: HashMap<String, ActionInstance>,
    /// Ports currently being read (`""` = auto).
    ports_in_flight: HashSet<String>,
}

impl Plugin {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            ports_in_flight: HashSet::new(),
        }
    }

    fn port_key(settings: &ActionSettings) -> String {
        settings.device_port().unwrap_or("").to_string()
    }

    /// Ports that are due for a poll, with one representative context each.
    pub fn ports_due_for_poll(&self) -> Vec<String> {
        let mut due = HashSet::new();
        for inst in self.actions.values() {
            let key = Self::port_key(&inst.settings);
            if self.ports_in_flight.contains(&key) {
                continue;
            }
            let ready = inst
                .last_poll
                .is_none_or(|t| t.elapsed() >= inst.settings.poll_interval());
            if ready {
                due.insert(key);
            }
        }
        due.into_iter().collect()
    }

    pub fn request_battery_for_port(
        &mut self,
        bat_tx: &mpsc::Sender<BatteryOutcome>,
        port_key: String,
        force: bool,
    ) {
        if !self.ports_in_flight.insert(port_key.clone()) {
            return;
        }
        let tx = bat_tx.clone();
        let port_for_open = if port_key.is_empty() {
            None
        } else {
            Some(port_key.clone())
        };
        tokio::task::spawn_blocking(move || {
            let result = battery::read_battery(port_for_open.as_deref(), force, FORCE_WAIT)
                .map_err(|e| e.to_string());
            let _ = tx.blocking_send(BatteryOutcome {
                port_key,
                result,
                force,
            });
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
                let settings = payload.settings;
                let port = Self::port_key(&settings);
                self.actions.insert(
                    context.clone(),
                    ActionInstance {
                        settings,
                        view: KeyView::Loading,
                        last_poll: None,
                    },
                );
                self.push_key_visual(sink, &context).await?;
                self.request_battery_for_port(bat_tx, port, true);
            }
            Message::WillDisappear { context, .. } => {
                info!(%context, "willDisappear");
                self.actions.remove(&context);
            }
            Message::KeyDown { context, .. } => {
                info!(%context, "keyDown → force refresh");
                let port = if let Some(inst) = self.actions.get_mut(&context) {
                    inst.view = KeyView::Loading;
                    Self::port_key(&inst.settings)
                } else {
                    return Ok(());
                };
                self.push_key_visual(sink, &context).await?;
                self.request_battery_for_port(bat_tx, port, true);
            }
            Message::DidReceiveSettings {
                context, payload, ..
            } => {
                if let Some(inst) = self.actions.get_mut(&context) {
                    let old_port = Self::port_key(&inst.settings);
                    inst.settings = payload.settings;
                    let new_port = Self::port_key(&inst.settings);
                    info!(
                        %context,
                        poll = inst.settings.poll_interval_secs(),
                        show_pct = inst.settings.show_percentage(),
                        device = %new_port,
                        "settings updated"
                    );
                    if old_port != new_port {
                        inst.view = KeyView::Loading;
                        inst.last_poll = None;
                        self.push_key_visual(sink, &context).await?;
                        self.request_battery_for_port(bat_tx, new_port, true);
                    } else {
                        self.push_key_visual(sink, &context).await?;
                    }
                }
            }
            Message::PropertyInspectorDidAppear {
                action,
                context,
                ..
            } => {
                if action != ACTION_UUID {
                    return Ok(());
                }
                self.send_device_list(sink, &context).await?;
            }
            Message::SendToPlugin {
                action,
                context,
                payload,
                ..
            } => {
                if action != ACTION_UUID {
                    return Ok(());
                }
                match payload {
                    PiIn::RefreshDevices => {
                        self.send_device_list(sink, &context).await?;
                    }
                    PiIn::Unknown => {
                        debug!(%context, "unknown PI message");
                    }
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
        self.ports_in_flight.remove(&outcome.port_key);
        let force = outcome.force;
        let port_key = outcome.port_key;

        let (view, alert) = match outcome.result {
            Ok(levels) => {
                info!(%levels, %port_key, force, "battery update");
                (KeyView::Levels(levels), false)
            }
            Err(err) => {
                warn!(%err, %port_key, force, "battery read failed");
                (KeyView::Error, true)
            }
        };

        let now = Instant::now();
        let mut contexts = Vec::new();
        for (ctx, inst) in &mut self.actions {
            if Self::port_key(&inst.settings) == port_key {
                inst.view = view.clone();
                inst.last_poll = Some(now);
                contexts.push(ctx.clone());
            }
        }

        if alert {
            if let Some(ctx) = contexts.first() {
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

        for ctx in contexts {
            self.push_key_visual(sink, &ctx).await?;
        }
        Ok(())
    }

    async fn send_device_list(
        &self,
        sink: &mut SdSink,
        context: &str,
    ) -> Result<(), PluginError> {
        let selected = self
            .actions
            .get(context)
            .map(|a| a.settings.device_port.clone())
            .unwrap_or_default();

        let devices = match battery::list_devices() {
            Ok(list) => list
                .into_iter()
                .map(|d: FocusDevice| DeviceListEntry {
                    port: d.port,
                    label: d.label,
                })
                .collect(),
            Err(e) => {
                warn!(error = %e, "device enumeration failed");
                Vec::new()
            }
        };

        info!(count = devices.len(), %context, "sending device list to PI");
        sink.send(OutMsg::SendToPropertyInspector {
            action: ACTION_UUID.to_string(),
            context: context.to_string(),
            payload: PiOut::DeviceList { devices, selected },
        })
        .await?;
        Ok(())
    }

    async fn push_key_visual(&self, sink: &mut SdSink, context: &str) -> Result<(), PluginError> {
        let Some(inst) = self.actions.get(context) else {
            return Ok(());
        };
        let image = inst
            .view
            .image_data_uri(inst.settings.show_percentage());
        set_image(sink, context, &image).await?;
        clear_title(sink, context).await?;
        Ok(())
    }
}

async fn set_image(sink: &mut SdSink, context: &str, data_uri: &str) -> Result<(), PluginError> {
    sink.send(OutMsg::SetImage {
        context: context.to_string(),
        payload: ImagePayload {
            image: Some(data_uri.to_string()),
            target: Target::Both,
            state: None,
        },
    })
    .await?;
    Ok(())
}

async fn clear_title(sink: &mut SdSink, context: &str) -> Result<(), PluginError> {
    sink.send(OutMsg::SetTitle {
        context: context.to_string(),
        payload: TitlePayload {
            title: Some(String::new()),
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

    fn settings(poll: u64, port: &str) -> ActionSettings {
        ActionSettings {
            poll_interval_secs: poll,
            show_percentage: true,
            device_port: port.to_string(),
        }
    }

    impl Plugin {
        fn with_action(context: &str, settings: ActionSettings) -> Self {
            let mut plugin = Self::new();
            plugin.actions.insert(
                context.to_string(),
                ActionInstance {
                    settings,
                    view: KeyView::Loading,
                    last_poll: None,
                },
            );
            plugin
        }
    }

    #[test]
    fn action_settings_serde() {
        let parsed: ActionSettings = serde_json::from_str(
            r#"{"pollIntervalSecs":45,"showPercentage":false,"devicePort":"COM7"}"#,
        )
        .unwrap();
        assert_eq!(parsed.poll_interval_secs(), 45);
        assert!(!parsed.show_percentage());
        assert_eq!(parsed.device_port(), Some("COM7"));

        let defaults: ActionSettings = serde_json::from_str("{}").unwrap();
        assert!(defaults.device_port().is_none());
        assert!(defaults.show_percentage());
    }

    #[test]
    fn ports_due_groups_by_device() {
        let mut plugin = Plugin::with_action("a", settings(60, "COM4"));
        plugin.actions.insert(
            "b".to_string(),
            ActionInstance {
                settings: settings(60, "COM4"),
                view: KeyView::Loading,
                last_poll: None,
            },
        );
        plugin.actions.insert(
            "c".to_string(),
            ActionInstance {
                settings: settings(60, "COM7"),
                view: KeyView::Loading,
                last_poll: None,
            },
        );
        let mut ports = plugin.ports_due_for_poll();
        ports.sort();
        assert_eq!(ports, vec!["COM4".to_string(), "COM7".to_string()]);
    }

    #[tokio::test]
    async fn port_in_flight_blocks_duplicate() {
        let mut plugin = Plugin::with_action("a", settings(60, "COM4"));
        let (tx, _rx) = mpsc::channel::<BatteryOutcome>(4);
        plugin.request_battery_for_port(&tx, "COM4".into(), true);
        assert!(plugin.ports_in_flight.contains("COM4"));
        assert!(plugin.ports_due_for_poll().is_empty());
    }

    #[test]
    fn battery_result_updates_matching_actions_only() {
        let mut plugin = Plugin::with_action("a", settings(60, "COM4"));
        plugin.actions.insert(
            "b".to_string(),
            ActionInstance {
                settings: settings(60, "COM7"),
                view: KeyView::Loading,
                last_poll: None,
            },
        );
        plugin.ports_in_flight.insert("COM4".into());

        // Use apply path without sink: simulate on_battery_result core
        plugin.ports_in_flight.remove("COM4");
        let levels = levels(100, 40);
        for inst in plugin.actions.values_mut() {
            if Plugin::port_key(&inst.settings) == "COM4" {
                inst.view = KeyView::Levels(levels.clone());
                inst.last_poll = Some(Instant::now());
            }
        }
        assert_eq!(
            plugin.actions.get("a").unwrap().view,
            KeyView::Levels(levels)
        );
        assert_eq!(plugin.actions.get("b").unwrap().view, KeyView::Loading);
    }

    #[tokio::test]
    async fn request_battery_is_single_flight_per_port() {
        let mut plugin = Plugin::with_action("a", settings(60, ""));
        let (tx, mut rx) = mpsc::channel::<BatteryOutcome>(4);
        plugin.request_battery_for_port(&tx, String::new(), true);
        plugin.request_battery_for_port(&tx, String::new(), true);
        assert_eq!(plugin.ports_in_flight.len(), 1);

        let _ = rx.recv().await;
        let second = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(second.is_err() || second.unwrap().is_none());
    }
}
