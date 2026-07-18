mod protocol;

#[cfg(target_os = "linux")]
mod bluez;
#[cfg(target_os = "linux")]
mod bridge;
#[cfg(target_os = "linux")]
mod device_link;

use clap::{Parser, Subcommand};

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum HostCommand {
    /// Clear every controller address stored by the ESP32.
    Reset,
}

#[derive(Parser, Debug)]
#[command(
    name = "powertooth",
    version = env!("POWERTOOTH_BUILD_VERSION"),
    about = "Synchronize Linux BlueZ gamepads with PowerTooth"
)]
pub struct Args {
    #[command(subcommand)]
    command: Option<HostCommand>,
    #[arg(long, default_value = env!("POWERTOOTH_DEFAULT_DEVICE"))]
    device: String,
    #[arg(long, default_value = env!("POWERTOOTH_DEFAULT_BAUD"))]
    baud: u32,
    #[arg(long, default_value = env!("POWERTOOTH_DEFAULT_CONNECT_DELAY_MS"))]
    connect_delay_ms: u64,
    #[arg(long, default_value = env!("POWERTOOTH_DEFAULT_HANDSHAKE_ATTEMPTS"))]
    handshake_attempts: u32,

    /// How often to poll BlueZ (over D-Bus) for paired-set changes; the
    /// serial link is only used when the set actually changed.
    #[arg(long, default_value = env!("POWERTOOTH_DEFAULT_INTERVAL_SECONDS"))]
    interval_seconds: u64,
    #[arg(long, default_value = env!("POWERTOOTH_DEFAULT_PAIR_TIMEOUT_SECONDS"))]
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
    eprintln!("powertooth requires Linux because it integrates with BlueZ");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_help_flag_is_accepted() {
        let error = Args::try_parse_from(["powertooth", "--help"]).unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn reset_subcommand_is_accepted() {
        let args = Args::try_parse_from(["powertooth", "reset"]).unwrap();
        assert_eq!(args.command, Some(HostCommand::Reset));
    }

    #[test]
    fn help_subcommand_is_accepted() {
        let error = Args::try_parse_from(["powertooth", "help"]).unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn version_comes_from_the_build_version() {
        let error = Args::try_parse_from(["powertooth", "-V"]).unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
        assert_eq!(
            error.to_string(),
            format!("powertooth {}\n", env!("POWERTOOTH_BUILD_VERSION"))
        );
    }

    #[test]
    fn command_defaults_come_from_the_build_configuration() {
        let args = Args::try_parse_from(["powertooth"]).unwrap();
        assert_eq!(args.command, None);
        assert_eq!(args.device, env!("POWERTOOTH_DEFAULT_DEVICE"));
        assert_eq!(args.baud.to_string(), env!("POWERTOOTH_DEFAULT_BAUD"));
        assert_eq!(
            args.connect_delay_ms.to_string(),
            env!("POWERTOOTH_DEFAULT_CONNECT_DELAY_MS")
        );
        assert_eq!(
            args.handshake_attempts.to_string(),
            env!("POWERTOOTH_DEFAULT_HANDSHAKE_ATTEMPTS")
        );
        assert_eq!(
            args.interval_seconds.to_string(),
            env!("POWERTOOTH_DEFAULT_INTERVAL_SECONDS")
        );
        assert_eq!(
            args.pair_timeout_seconds.to_string(),
            env!("POWERTOOTH_DEFAULT_PAIR_TIMEOUT_SECONDS")
        );
    }
}
