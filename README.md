# cmd-spec

A serializable shell command builder. Define commands as data, pass them around, convert to `std::process::Command` (or `tokio::process::Command` with the `tokio` feature) when ready to run.

```toml
[dependencies]
cmd-spec = "0.1"
```

## Usage

Fluent builder:

```rust
let cmd = ShellCommand::new("cargo")
    .arg("build")
    .arg("--release")
    .env("RUST_LOG", "debug")
    .working_dir("/tmp");
```

Conditional args with `.mutate()`:

```rust
let mut cmd = ShellCommand::new("cargo").arg("build").mutate();

if release {
    cmd.arg("--release");
}

let cmd = cmd.finish();
```

Scoped mutation with `.with()` — stays in functional style:

```rust
let cmd = ShellCommand::new("cargo")
    .arg("build")
    .with(|cmd| {
        if release {
            cmd.arg("--release");
        }
        cmd.env("RUST_LOG", "debug");
    })
    .working_dir("/project");
```

Converting to `std::process::Command`:

```rust
let cmd = ShellCommand::new("echo").arg("hi");
let output = std::process::Command::from(&cmd).output()?;
```

Serde roundtrip (requires `serde` feature, enabled by default):

```rust
let cmd = ShellCommand::new("ls").arg("-la").working_dir("/home");

let json = serde_json::to_string(&cmd)?;
let restored: ShellCommand = serde_json::from_str(&json)?;
assert_eq!(cmd, restored);
```

## Features

| Feature | Default | What it does |
| --- | --- | --- |
| serde | yes | Serialize/Deserialize derives |
| tokio | no | From<&ShellCommand> for tokio::process::Command |

## License

MIT