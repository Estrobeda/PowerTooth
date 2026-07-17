use std::{collections::BTreeSet, time::Duration};

use anyhow::{Context, Result, bail};
use bluer::Address;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, ReadHalf, WriteHalf},
    time,
};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::protocol::{Command, Message, decode, encode};

#[cfg(feature = "debug-logging")]
fn log_outbound(line: &str) {
    eprintln!("host -> ESP32: {}", line.trim_end());
}

#[cfg(not(feature = "debug-logging"))]
fn log_outbound(_: &str) {}

#[cfg(feature = "debug-logging")]
fn log_inbound(line: &str) {
    eprintln!("ESP32 -> host: {}", line.trim_end());
}

#[cfg(not(feature = "debug-logging"))]
fn log_inbound(_: &str) {}

pub struct HostLink {
    lines: Lines<BufReader<ReadHalf<SerialStream>>>,
    writer: WriteHalf<SerialStream>,
    pair_requested: bool,
}

impl HostLink {
    pub fn open(path: &str, baud: u32) -> Result<Self> {
        let serial = tokio_serial::new(path, baud)
            .open_native_async()
            .with_context(|| format!("open ESP32 host link {path}"))?;
        let (reader, writer) = tokio::io::split(serial);
        Ok(Self {
            lines: BufReader::new(reader).lines(),
            writer,
            pair_requested: false,
        })
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
            log_inbound(&line);
            if let Some(message) = decode(&line) {
                return Ok(message);
            }
            eprintln!("ESP32 log: {line}");
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
            log_inbound(&line);
            if let Some(message) = decode(&line) {
                return Ok(message);
            }
            eprintln!("ESP32 log: {line}");
        }
    }
}
