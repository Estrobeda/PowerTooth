use std::{collections::BTreeMap, env, fs, process::Command};

const DEFAULT_VERSION: &str = "0.0.0-default";
const DEFAULTS_FILE: &str = "build-defaults.conf";
const DEFAULTS: [(&str, &str); 7] = [
    ("PROTOCOL_PREFIX", "PT/1"),
    ("DEVICE", "/dev/ttyACM0"),
    ("BAUD", "115200"),
    ("CONNECT_DELAY_MS", "1000"),
    ("HANDSHAKE_ATTEMPTS", "3"),
    ("INTERVAL_SECONDS", "10"),
    ("PAIR_TIMEOUT_SECONDS", "30"),
];

fn nonempty_environment(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn exact_git_tag() -> Option<String> {
    let output = Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let tag = String::from_utf8(output.stdout).ok()?;
    let tag = tag.trim();
    (!tag.is_empty()).then(|| tag.to_owned())
}

fn read_build_defaults() -> BTreeMap<String, String> {
    let mut values: BTreeMap<String, String> = DEFAULTS
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    let contents = fs::read_to_string(DEFAULTS_FILE)
        .unwrap_or_else(|error| panic!("read {DEFAULTS_FILE}: {error}"));

    for (index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("{DEFAULTS_FILE}:{}: expected KEY=VALUE", index + 1));
        let key = key.trim();
        let value = value.trim();
        if !values.contains_key(key) {
            panic!("{DEFAULTS_FILE}:{}: unknown setting {key}", index + 1);
        }
        if value.is_empty() {
            panic!("{DEFAULTS_FILE}:{}: {key} cannot be empty", index + 1);
        }
        values.insert(key.to_owned(), value.to_owned());
    }

    for (key, value) in &mut values {
        let environment_name = format!("POWERTOOTH_DEFAULT_{key}");
        println!("cargo:rerun-if-env-changed={environment_name}");
        if let Some(override_value) = nonempty_environment(&environment_name) {
            *value = override_value;
        }
    }

    for key in [
        "BAUD",
        "CONNECT_DELAY_MS",
        "HANDSHAKE_ATTEMPTS",
        "INTERVAL_SECONDS",
        "PAIR_TIMEOUT_SECONDS",
    ] {
        let value = &values[key];
        let parsed = value
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("{key} must be a non-negative integer, got {value:?}"));
        if matches!(key, "BAUD" | "HANDSHAKE_ATTEMPTS" | "INTERVAL_SECONDS") && parsed == 0 {
            panic!("{key} must be greater than zero");
        }
    }

    if values["PROTOCOL_PREFIX"].chars().any(char::is_whitespace) {
        panic!("PROTOCOL_PREFIX cannot contain whitespace");
    }

    values
}

fn main() {
    println!("cargo:rerun-if-changed={DEFAULTS_FILE}");
    println!("cargo:rerun-if-env-changed=POWERTOOTH_VERSION");
    println!("cargo:rerun-if-env-changed=TAG");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/tags");

    let version = nonempty_environment("POWERTOOTH_VERSION")
        .or_else(|| nonempty_environment("TAG"))
        .or_else(|| nonempty_environment("GITHUB_REF_NAME"))
        .or_else(exact_git_tag)
        .unwrap_or_else(|| DEFAULT_VERSION.to_owned());

    println!("cargo:rustc-env=POWERTOOTH_BUILD_VERSION={version}");

    for (key, value) in read_build_defaults() {
        println!("cargo:rustc-env=POWERTOOTH_DEFAULT_{key}={value}");
    }
}
