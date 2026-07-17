mod protocol;

#[cfg(target_os = "linux")]
mod bluez;
#[cfg(target_os = "linux")]
mod bridge;
#[cfg(target_os = "linux")]
mod device_link;

#[cfg(target_os = "linux")]
use clap::Parser;

#[cfg(target_os = "linux")]
#[derive(Parser, Debug)]
#[command(version, about = "Synchronize Linux BlueZ gamepads with PowerTooth")]
pub struct Args {
    #[arg(long, default_value = "/dev/ttyACM0")]
    device: String,
    #[arg(long, default_value_t = 115_200)]
    baud: u32,
    #[arg(long, default_value_t = 5)]
    interval_seconds: u64,
    #[arg(long, default_value_t = 30)]
    pair_timeout_seconds: u64,
    #[arg(long)]
    list_bluez: bool,
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bridge::run(Args::parse()).await
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("powertooth-host requires Linux because it integrates with BlueZ");
    std::process::exit(1);
}
