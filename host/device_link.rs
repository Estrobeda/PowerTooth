use std::{
    collections::BTreeSet,
    os::fd::{AsRawFd, RawFd},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use bluer::Address;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, ReadHalf, WriteHalf},
    time,
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::protocol::{Command, Message, decode, encode};

fn log_outbound(line: &str) {
    eprintln!("host -> esp32: {}", line.trim_end());
}

fn log_inbound(line: &str) {
    eprintln!("esp32 -> host: {}", line.trim_end());
}

/// Non-protocol output from the ESP32 (its own firmware logs). Debug builds
/// include it in the host log; release builds discard it.
#[cfg(feature = "debug-logging")]
fn log_device(line: &str) {
    eprintln!("esp32 log: {line}");
}

#[cfg(not(feature = "debug-logging"))]
fn log_device(_: &str) {}

/// The ESP32-C6 USB Serial/JTAG controller emulates the classic esptool
/// auto-reset circuit: DTR selects download boot and RTS drives chip reset.
/// Both lines must idle inactive for the console to work (bench-verified on
/// Bazzite), and both must always change in a single TIOCMSET — one USB
/// control transfer — so the chip never observes a partial state.
fn set_modem_lines(fd: RawFd, dtr: bool, rts: bool) -> Result<()> {
    let mut bits: libc::c_int = 0;
    if unsafe { libc::ioctl(fd, libc::TIOCMGET, &mut bits) } != 0 {
        return Err(std::io::Error::last_os_error()).context("read serial modem line state");
    }
    bits &= !(libc::TIOCM_DTR | libc::TIOCM_RTS);
    if dtr {
        bits |= libc::TIOCM_DTR;
    }
    if rts {
        bits |= libc::TIOCM_RTS;
    }
    if unsafe { libc::ioctl(fd, libc::TIOCMSET, &bits) } != 0 {
        return Err(std::io::Error::last_os_error()).context("set serial DTR/RTS");
    }
    Ok(())
}

pub struct HostLink {
    lines: Lines<BufReader<ReadHalf<SerialStream>>>,
    writer: WriteHalf<SerialStream>,
    fd: RawFd,
    pair_requested: bool,
}

impl HostLink {
    pub fn open(path: &str, baud: u32) -> Result<Self> {
        let serial = tokio_serial::new(path, baud)
            .open_native_async()
            .with_context(|| format!("open ESP32 host link {path}"))?;
        // The cdc-acm driver raises DTR+RTS inside the kernel's open path;
        // drop them again before the chip can misread them as boot straps.
        let fd = serial.as_raw_fd();
        set_modem_lines(fd, false, false)?;
        let (reader, writer) = tokio::io::split(serial);
        Ok(Self {
            lines: BufReader::new(reader).lines(),
            writer,
            fd,
            pair_requested: false,
        })
    }

    /// esptool-style hard reset: pulse RTS with DTR held low, which reboots
    /// the ESP32 into a normal flash boot (DTR low means the download strap
    /// is never sampled). This recovers a chip that got wedged in ROM
    /// download mode by the kernel's DTR/RTS pulse landing mid-boot — that
    /// state is silent and ignores the text protocol, so no amount of
    /// handshake retries can fix it. The reset tears down USB Serial/JTAG,
    /// so the caller must drop this link, wait for re-enumeration, and
    /// reopen. Best-effort: the device may already be gone.
    pub async fn pulse_reset(&mut self) {
        let _ = set_modem_lines(self.fd, false, true);
        time::sleep(Duration::from_millis(100)).await;
        let _ = set_modem_lines(self.fd, false, false);
    }

    pub async fn send(&mut self, command: Command) -> Result<()> {
        let line = encode(command);
        log_outbound(&line);
        self.writer.write_all(line.as_bytes()).await?;
        self.expect_ok().await
    }

    pub async fn list(&mut self) -> Result<BTreeSet<Address>> {
        let line = encode(Command::List);
        log_outbound(&line);
        self.writer.write_all(line.as_bytes()).await?;
        let mut devices = BTreeSet::new();
        loop {
            match self.next_message().await? {
                Message::Device(value) => {
                    devices.insert(value.parse().context("ESP32 returned invalid address")?);
                }
                Message::End => return Ok(devices),
                Message::Pair => self.pair_requested = true,
                Message::Error(reason) => bail!("ESP32 rejected LIST: {reason}"),
                other => bail!("unexpected LIST response: {other:?}"),
            }
        }
    }

    pub fn take_pair_request(&mut self) -> bool {
        std::mem::take(&mut self.pair_requested)
    }

    pub async fn next_unsolicited(&mut self) -> Result<Message> {
        loop {
            let line = self
                .lines
                .next_line()
                .await?
                .context("ESP32 host link disconnected")?;
            if let Some(message) = decode(&line) {
                log_inbound(&line);
                return Ok(message);
            }
            log_device(&line);
        }
    }

    async fn expect_ok(&mut self) -> Result<()> {
        loop {
            match self.next_message().await? {
                Message::Ok => return Ok(()),
                Message::Pair => self.pair_requested = true,
                Message::Error(reason) => bail!("ESP32 rejected command: {reason}"),
                other => bail!("unexpected ESP32 response: {other:?}"),
            }
        }
    }

    async fn next_message(&mut self) -> Result<Message> {
        loop {
            let line = time::timeout(Duration::from_secs(3), self.lines.next_line())
                .await
                .context("timed out waiting for ESP32")??
                .context("ESP32 host link disconnected")?;
            if let Some(message) = decode(&line) {
                log_inbound(&line);
                return Ok(message);
            }
            log_device(&line);
        }
    }
}
