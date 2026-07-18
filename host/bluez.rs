use std::{collections::BTreeSet, time::Duration};

use anyhow::{Context, Result, bail};
use bluer::{Adapter, AdapterEvent, Address};
use futures::StreamExt;
use tokio::time;

const HID_SERVICE: &str = "00001812-0000-1000-8000-00805f9b34fb";

pub async fn paired_gamepads(adapter: &Adapter) -> Result<BTreeSet<Address>> {
    let mut result = BTreeSet::new();
    for address in adapter.device_addresses().await? {
        let device = adapter.device(address)?;
        if device.is_paired().await.unwrap_or(false) && is_gamepad(&device).await {
            result.insert(address);
        }
    }
    Ok(result)
}

async fn is_gamepad(device: &bluer::Device) -> bool {
    if device.icon().await.ok().flatten().as_deref() == Some("input-gaming") {
        return true;
    }
    let has_hid_service = device
        .uuids()
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
        .iter()
        .any(|uuid| uuid.to_string() == HID_SERVICE);
    let name = device
        .name()
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
        .to_lowercase();
    // Possibly, this should be a build time config.... or not....
    let gamepad_name = ["controller", "gamepad", "8bitdo"]
        .iter()
        .any(|word| name.contains(word));
    has_hid_service && gamepad_name
}

pub async fn pair_first_gamepad(adapter: &Adapter, timeout: Duration) -> Result<Address> {
    let address = discover_new_gamepad(adapter, timeout).await?;
    time::sleep(Duration::from_millis(500)).await;
    let device = adapter.device(address)?;
    device
        .set_trusted(true)
        .await
        .with_context(|| format!("trust {address}"))?;
    let mut attempt = 1;
    loop {
        match device.pair().await {
            Ok(()) => break,
            Err(_) if attempt < 2 => {
                attempt += 1;
                time::sleep(Duration::from_secs(1)).await;
            }
            Err(error) => return Err(error).with_context(|| format!("pair {address}")),
        }
    }
    if !device.is_connected().await.unwrap_or(false) {
        device
            .connect()
            .await
            .with_context(|| format!("connect {address}"))?;
    }
    Ok(address)
}

async fn discover_new_gamepad(adapter: &Adapter, timeout: Duration) -> Result<Address> {
    let mut events = adapter.discover_devices_with_changes().await?;
    time::timeout(timeout, async {
        while let Some(event) = events.next().await {
            if let AdapterEvent::DeviceAdded(address) = event {
                let device = adapter.device(address)?;
                if !device.is_paired().await.unwrap_or(false) && is_gamepad(&device).await {
                    return Ok(address);
                }
            }
        }
        bail!("BlueZ discovery ended")
    })
    .await
    .context("no unpaired controller appeared during the pairing window")?
}
