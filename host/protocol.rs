const PREFIX: &str = env!("POWERTOOTH_DEFAULT_PROTOCOL_PREFIX");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Hello,
    Add(String),
    Remove(String),
    List,
    Reset,
    Sync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Ok,
    Error(String),
    Device(String),
    End,
    Pair,
    Wake(String),
}

pub fn encode(command: Command) -> String {
    let body = match command {
        Command::Hello => "HELLO".into(),
        Command::Add(address) => format!("ADD {}", address.to_lowercase()),
        Command::Remove(address) => format!("REMOVE {}", address.to_lowercase()),
        Command::List => "LIST".into(),
        Command::Reset => "RESET".into(),
        Command::Sync => "SYNC".into(),
    };
    format!("{PREFIX} {body}\n")
}

pub fn decode(line: &str) -> Option<Message> {
    let body = line.strip_prefix(PREFIX)?.strip_prefix(' ')?;
    if body == "OK" {
        return Some(Message::Ok);
    }
    if body == "END" {
        return Some(Message::End);
    }
    if body == "PAIR" {
        return Some(Message::Pair);
    }
    if let Some(value) = body.strip_prefix("ERR ") {
        return Some(Message::Error(value.into()));
    }
    if let Some(value) = body.strip_prefix("DEVICE ") {
        return Some(Message::Device(value.into()));
    }
    if let Some(value) = body.strip_prefix("WAKE ") {
        return Some(Message::Wake(value.into()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_framed_and_addresses_are_normalized() {
        assert_eq!(encode(Command::Hello), format!("{PREFIX} HELLO\n"));
        assert_eq!(
            encode(Command::Add("AA:BB:CC:DD:EE:FF".into())),
            format!("{PREFIX} ADD aa:bb:cc:dd:ee:ff\n")
        );
        assert_eq!(
            encode(Command::Remove("AA:BB:CC:DD:EE:FF".into())),
            format!("{PREFIX} REMOVE aa:bb:cc:dd:ee:ff\n")
        );
        assert_eq!(encode(Command::List), format!("{PREFIX} LIST\n"));
        assert_eq!(encode(Command::Reset), format!("{PREFIX} RESET\n"));
        assert_eq!(encode(Command::Sync), format!("{PREFIX} SYNC\n"));
    }

    #[test]
    fn responses_and_events_decode() {
        assert_eq!(decode(&format!("{PREFIX} OK")), Some(Message::Ok));
        assert_eq!(
            decode(&format!("{PREFIX} ERR registry-full")),
            Some(Message::Error("registry-full".into()))
        );
        assert_eq!(
            decode(&format!("{PREFIX} DEVICE aa:bb:cc:dd:ee:ff")),
            Some(Message::Device("aa:bb:cc:dd:ee:ff".into()))
        );
        assert_eq!(decode(&format!("{PREFIX} END")), Some(Message::End));
        assert_eq!(decode(&format!("{PREFIX} PAIR")), Some(Message::Pair));
        assert_eq!(
            decode(&format!("{PREFIX} WAKE aa:bb:cc:dd:ee:ff")),
            Some(Message::Wake("aa:bb:cc:dd:ee:ff".into()))
        );
    }

    #[test]
    fn unrelated_console_logs_are_ignored() {
        assert_eq!(decode("I (123) powertooth: booted"), None);
        assert_eq!(decode(""), None);
    }
}
