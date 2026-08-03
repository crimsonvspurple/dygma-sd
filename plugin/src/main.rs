//! Stream Deck plugin: show Dygma wireless battery on a key.
//!
//! Protocol: Elgato WebSocket SDK via `streamdeck-rs`.
//! Hardware: Neuron Focus serial via `dygma_focus`.

mod battery;
mod error;
mod plugin;
mod visual;

use error::PluginError;
use futures::StreamExt;
use plugin::{BatteryOutcome, Plugin, SdSocket};
use std::env;
use std::process::ExitCode;
use std::time::Duration;
use streamdeck_rs::registration::RegistrationParams;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{error, info};

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    // Resolve mode before Stream Deck registration args.
    if env::args().any(|a| a == "--self-test") {
        return run_self_test();
    }

    match run_async_plugin() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!(error = %e, "plugin failed");
            ExitCode::FAILURE
        }
    }
}

fn run_self_test() -> ExitCode {
    info!("self-test: enumerating devices + reading battery");
    match battery::list_devices() {
        Ok(devs) => {
            println!("devices ({}):", devs.len());
            for d in &devs {
                println!("  {} — {}", d.port, d.label);
            }
            if devs.is_empty() {
                eprintln!("No Dygma Focus devices found.");
            }
        }
        Err(e) => {
            eprintln!("device list failed: {e}");
            return ExitCode::FAILURE;
        }
    }

    match battery::read_battery(None, true, Duration::from_secs(2)) {
        Ok(levels) => {
            println!("{}", levels.title());
            println!("{levels}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("battery read failed: {e}");
            eprintln!("Close Bazecor and ensure a Neuron is on USB (COM port available).");
            ExitCode::FAILURE
        }
    }
}

#[tokio::main]
async fn run_async_plugin() -> Result<(), PluginError> {
    let params = RegistrationParams::from_args(env::args())?;

    info!(
        port = params.port,
        uuid = %params.uuid,
        "connecting to Stream Deck"
    );

    let socket = SdSocket::connect(params.port, params.event, params.uuid).await?;
    run_plugin(socket).await
}

/// Event loop: Stream Deck messages, poll timer, and battery results.
async fn run_plugin(socket: SdSocket) -> Result<(), PluginError> {
    let (mut sink, mut stream) = socket.split();
    let (bat_tx, mut bat_rx) = mpsc::channel::<BatteryOutcome>(8);
    let mut plugin = Plugin::new();

    let mut tick = interval(Duration::from_secs(5));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    // Skip the immediate first tick so we don't race willAppear's own refresh.
    tick.tick().await;

    loop {
        tokio::select! {
            item = stream.next() => {
                match item {
                    None => {
                        info!("stream deck connection closed");
                        break;
                    }
                    Some(Err(e)) => {
                        return Err(PluginError::from(e));
                    }
                    Some(Ok(msg)) => {
                        plugin.handle_message(&mut sink, &bat_tx, msg).await?;
                    }
                }
            }
            _ = tick.tick() => {
                for port in plugin.ports_due_for_poll() {
                    plugin.request_battery_for_port(&bat_tx, port, true);
                }
            }
            outcome = bat_rx.recv() => {
                match outcome {
                    None => break,
                    Some(outcome) => {
                        plugin.on_battery_result(&mut sink, outcome).await?;
                    }
                }
            }
        }
    }

    drop(bat_tx);
    info!("plugin event loop stopped");
    Ok(())
}
