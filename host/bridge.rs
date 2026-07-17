use std::{collections::BTreeSet, time::Duration};

use anyhow::{Context, Result};
use bluer::{Adapter, Address, Session};
use tokio::time;

use crate::{
    Args,
    bluez::{pair_first_gamepad, paired_gamepads},
    device_link::HostLink,
    protocol::{Command, Message},
};

pub async fn run(args: Args) -> Result<()> {
    let session = Session::new().await.context("connect to BlueZ D-Bus")?;
    let adapter = session
        .default_adapter()
        .await
        .context("find default Bluetooth adapter")?;
    adapter
        .set_powered(true)
        .await
        .context("power on Bluetooth adapter")?;

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

async fn run_connection(args: &Args, adapter: &Adapter) -> Result<()> {
    let mut link = HostLink::open(&args.device, args.baud)?;
    link.send(Command::Hello).await?;
    if link.take_pair_request() {
        pair_and_sync(args, adapter, &mut link).await?;
    } else {
        reconcile(&mut link, &paired_gamepads(adapter).await?).await?;
    }

    let mut ticker = time::interval(Duration::from_secs(args.interval_seconds));
    ticker.tick().await;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                reconcile(&mut link, &paired_gamepads(adapter).await?).await?;
                handle_deferred_pair(args, adapter, &mut link).await?;
            },
            message = link.next_unsolicited() => {
                if message? == Message::Pair {
                    pair_and_sync(args, adapter, &mut link).await?;
                }
            }
            _ = tokio::signal::ctrl_c() => return Ok(()),
        }
    }
}

async fn handle_deferred_pair(args: &Args, adapter: &Adapter, link: &mut HostLink) -> Result<()> {
    if link.take_pair_request() {
        pair_and_sync(args, adapter, link).await?;
    }
    Ok(())
}

async fn pair_and_sync(args: &Args, adapter: &Adapter, link: &mut HostLink) -> Result<()> {
    eprintln!("PAIRING");
    match pair_first_gamepad(adapter, Duration::from_secs(args.pair_timeout_seconds)).await {
        Ok(address) => {
            eprintln!("Pairing with {address}");
            reconcile(link, &paired_gamepads(adapter).await?).await
        }
        Err(error) => {
            eprintln!("pairing failed: {error:#}");
            Ok(())
        }
    }
}

async fn reconcile(link: &mut HostLink, wanted: &BTreeSet<Address>) -> Result<()> {
    let stored = link.list().await?;
    for address in stored.difference(wanted) {
        link.send(Command::Remove(address.to_string())).await?;
    }
    for address in wanted.difference(&stored) {
        link.send(Command::Add(address.to_string())).await?;
    }
    link.send(Command::Sync).await
}
