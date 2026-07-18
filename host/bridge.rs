use std::{collections::BTreeSet, time::Duration};

use anyhow::{Context, Result, bail};
use bluer::{Adapter, Address, Session};
use tokio::time;

use crate::{
    Args, HostCommand,
    bluez::{pair_first_gamepad, paired_gamepads},
    device_link::HostLink,
    protocol::{Command, Message},
};

pub async fn run(args: Args) -> Result<()> {
    #[cfg(feature = "debug-logging")]
    eprintln!("PowerTooth debug build: ESP32 firmware output is included in this log");

    if args.command == Some(HostCommand::Reset) {
        return reset_registry(&args).await;
    }

    let session = Session::new().await.context("connect to BlueZ D-Bus")?;
    let adapter = session
        .default_adapter()
        .await
        .context("find default Bluetooth adapter")?;

    // Ensure the adapter is powered on.
    // This may or may not be wanted, in ny case it is but if the user wants the bluetooth adapter to be off,
    // its probably just their choice. We do however need it for powertooth so meh... I think this is generally
    // the prefered behavior for now.
    adapter
        .set_powered(true)
        .await
        .context("power on Bluetooth adapter")?;

    // Commandline argument
    if args.list_bluez {
        for address in paired_gamepads(&adapter).await? {
            println!("{address}");
        }
        return Ok(());
    }

    loop {
        match run_connection(&args, &adapter).await {
            Ok(()) => return Ok(()),
            Err(error) => {
                eprintln!("host link error: {error:#}; retrying in 2 seconds");
                time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn reset_registry(args: &Args) -> Result<()> {
    let mut link = open_and_handshake(args).await?;
    link.send(Command::Reset).await?;
    eprintln!("ESP32 controller registry cleared");
    Ok(())
}

async fn run_connection(args: &Args, adapter: &Adapter) -> Result<()> {
    let mut link = open_and_handshake(args).await?;
    // The handshake already confirmed the ESP32 is listening, so the startup
    // session goes straight to LIST.
    let mut synced = reconcile(&mut link, &paired_gamepads(adapter).await?).await?;
    if link.take_pair_request() {
        pair_and_sync(args, adapter, &mut link, &mut synced).await?;
    }

    let mut ticker = time::interval(Duration::from_secs(args.interval_seconds));
    ticker.tick().await;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // The tick polls BlueZ over D-Bus only. The serial link
                // stays quiet unless the paired set differs from what the
                // ESP32 registry last converged on.
                let wanted = paired_gamepads(adapter).await?;
                if wanted != synced {
                    synced = sync_registry(&mut link, &wanted).await?;
                }
                if link.take_pair_request() {
                    pair_and_sync(args, adapter, &mut link, &mut synced).await?;
                }
            },
            message = link.next_unsolicited() => {
                if message? == Message::Pair {
                    pair_and_sync(args, adapter, &mut link, &mut synced).await?;
                }
            }
            _ = tokio::signal::ctrl_c() => return Ok(()),
        }
    }
}

async fn open_and_handshake(args: &Args) -> Result<HostLink> {
    let mut link = HostLink::open(&args.device, args.baud)?;
    // Opening ESP32 USB Serial/JTAG can reset the board. Let app_main and the
    // host-link task start before sending the first command.
    time::sleep(Duration::from_millis(args.connect_delay_ms)).await;
    let mut handshake_attempt = 1;
    loop {
        match link.send(Command::Hello).await {
            Ok(()) => break,
            Err(error) if handshake_attempt < args.handshake_attempts => {
                eprintln!(
                    "ESP32 handshake attempt {handshake_attempt}/{} failed: {error:#}; retrying without reopening USB",
                    args.handshake_attempts
                );
                handshake_attempt += 1;
                time::sleep(Duration::from_secs(1)).await;
            }
            Err(error) => {
                // A chip wedged in ROM download mode never answers, no matter
                // how often we retry. Force a clean reboot into the app, then
                // let the outer loop reopen once the USB device re-enumerates.
                eprintln!("ESP32 handshake failed: {error:#}; pulsing chip reset and reconnecting");
                link.pulse_reset().await;
                drop(link);
                time::sleep(Duration::from_secs(2)).await;
                return Err(error.context("ESP32 handshake failed; chip reset issued"));
            }
        }
    }
    Ok(link)
}

async fn pair_and_sync(
    args: &Args,
    adapter: &Adapter,
    link: &mut HostLink,
    synced: &mut BTreeSet<Address>,
) -> Result<()> {
    eprintln!("PAIRING");
    let pairing = pair_first_gamepad(adapter, Duration::from_secs(args.pair_timeout_seconds));
    tokio::pin!(pairing);
    let outcome = loop {
        tokio::select! {
            outcome = &mut pairing => break outcome,
            // Keep draining the serial link while BlueZ pairs. The ESP32
            // logs continuously in debug builds;
            message = link.next_unsolicited() => {
                message?;
            }
        }
    };
    let paired = match outcome {
        Ok(address) => {
            eprintln!("Paired {address}");
            Some(address)
        }
        // A controller that BlueZ already knows reconnects without a new
        // pairing event, so a "failed" discovery is still a normal outcome.
        Err(error) => {
            eprintln!("pairing did not complete: {error:#}");
            None
        }
    };
    // SYNC answers the case's PAIR request: pairing has finished. It is sent
    // even when nothing paired so the case never sticks in pairing mode.
    link.send(Command::Sync).await?;
    if let Some(address) = paired {
        if !synced.contains(&address) {
            link.send(Command::Add(address.to_string())).await?;
            synced.insert(address);
        }
    }
    Ok(())
}

/// One on-demand command session: confirm the ESP32 is still listening, then
/// converge its registry on the wanted set.
async fn sync_registry(
    link: &mut HostLink,
    wanted: &BTreeSet<Address>,
) -> Result<BTreeSet<Address>> {
    link.send(Command::Hello).await?;
    reconcile(link, wanted).await
}

/// Diff the ESP32 registry against `wanted`, push ADD/REMOVE commands, and
/// LIST again to verify, repeating until both sides agree. Returns the set
/// the ESP32 converged on so callers can cache it and skip future sessions.
async fn reconcile(link: &mut HostLink, wanted: &BTreeSet<Address>) -> Result<BTreeSet<Address>> {
    for _ in 0..3 {
        let stored = link.list().await?;
        let stale: Vec<Address> = stored.difference(wanted).copied().collect();
        let missing: Vec<Address> = wanted.difference(&stored).copied().collect();
        if stale.is_empty() && missing.is_empty() {
            return Ok(stored);
        }
        for address in stale {
            link.send(Command::Remove(address.to_string())).await?;
        }
        for address in missing {
            link.send(Command::Add(address.to_string())).await?;
        }
    }
    bail!("ESP32 registry did not converge after 3 attempts")
}
