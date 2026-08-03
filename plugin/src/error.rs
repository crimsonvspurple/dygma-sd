//! Application-level errors for the Stream Deck plugin binary.

use dygma_focus::errors::FocusError;
use streamdeck_rs::registration::RegistrationParamsError;
use streamdeck_rs::socket::{ConnectError, StreamDeckSocketError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("stream deck registration failed: {0}")]
    Registration(#[from] RegistrationParamsError),

    #[error("stream deck websocket connect failed: {0}")]
    Connect(#[from] ConnectError),

    #[error("stream deck protocol error: {0}")]
    Protocol(#[from] StreamDeckSocketError),

    #[error(transparent)]
    Battery(#[from] FocusError),
}
