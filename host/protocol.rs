const PREFIX: &str = "PT/1 ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Hello,
    Add(String),
    Remove(String),
    List,
    Reset,
    Sync,
    Power,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Ok,
    Error(String),
    Device(String),
    End,
    Pair,
    Wake(String),
    Power(bool),
}

pub fn encode(command: Command) -> String {
    let body = match command {
        Command::Hello => "HELLO".into(),
        Command::Add(address) => format!("ADD {}", address.to_lowercase()),
        Command::Remove(address) => format!("REMOVE {}", address.to_lowercase()),
        Command::List => "LIST".into(),
        Command::Reset => "RESET".into(),
        Command::Sync => "SYNC".into(),
        Command::Power => "POWER?".into(),
    };
    format!("{PREFIX}{body}\n")
}

pub fn decode(line: &str) -> Option<Message> {
    let body = line.strip_prefix(PREFIX)?;
    if body == "OK" {
        return Some(Message::Ok);
    }
    if body == "END" {
        return Some(Message::End);
    }
    if body == "PAIR" {
        return Some(Message::Pair);
    }
    if body == "POWER ON" {
        return Some(Message::Power(true));
    }
    if body == "POWER OFF" {
        return Some(Message::Power(false));
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
        assert_eq!(encode(Command::Hello), "PT/1 HELLO\n");
        assert_eq!(
            encode(Command::Add("AA:BB:CC:DD:EE:FF".into())),
            "PT/1 ADD aa:bb:cc:dd:ee:ff\n"
        );
        assert_eq!(
            encode(Command::Remove("AA:BB:CC:DD:EE:FF".into())),
            "PT/1 REMOVE aa:bb:cc:dd:ee:ff\n"
        );
        assert_eq!(encode(Command::List), "PT/1 LIST\n");
        assert_eq!(encode(Command::Reset), "PT/1 RESET\n");
        assert_eq!(encode(Command::Sync), "PT/1 SYNC\n");
        assert_eq!(encode(Command::Power), "PT/1 POWER?\n");
    }

    #[test]
    fn responses_and_events_decode() {
        assert_eq!(decode("PT/1 OK"), Some(Message::Ok));
        assert_eq!(
            decode("PT/1 ERR registry-full"),
            Some(Message::Error("registry-full".into()))
        );
        assert_eq!(
            decode("PT/1 DEVICE aa:bb:cc:dd:ee:ff"),
            Some(Message::Device("aa:bb:cc:dd:ee:ff".into()))
        );
        assert_eq!(decode("PT/1 END"), Some(Message::End));
        assert_eq!(decode("PT/1 PAIR"), Some(Message::Pair));
        assert_eq!(
            decode("PT/1 WAKE aa:bb:cc:dd:ee:ff"),
            Some(Message::Wake("aa:bb:cc:dd:ee:ff".into()))
        );
        assert_eq!(decode("PT/1 POWER ON"), Some(Message::Power(true)));
        assert_eq!(decode("PT/1 POWER OFF"), Some(Message::Power(false)));
    }

    #[test]
    fn unrelated_console_logs_are_ignored() {
        assert_eq!(decode("I (123) powertooth: booted"), None);
        assert_eq!(decode(""), None);
    }
}
