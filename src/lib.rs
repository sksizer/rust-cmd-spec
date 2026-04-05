#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ShellCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub working_dir: Option<String>,
}

/// Mutable handle for in-place mutation of a ShellCommand.
/// Call `.finish()` to get the ShellCommand back.
///
/// This is a wrapper around the primary ShellCommand struct that
/// provides in place mutation of the values for ergonomic reasons.
///
/// It requires explicit calls to break out of a functional style, so i think
/// its a worthwhile tradeoff
pub struct ShellCommandMut {
    inner: ShellCommand,
}

impl ShellCommandMut {
    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.inner.args.push(arg.into());
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.inner.args.extend(args.into_iter().map(|a| a.into()));
        self
    }

    pub fn env(&mut self, key: impl Into<String>, val: impl Into<String>) -> &mut Self {
        self.inner.env.push((key.into(), val.into()));
        self
    }

    pub fn working_dir(&mut self, dir: impl Into<String>) -> &mut Self {
        self.inner.working_dir = Some(dir.into());
        self
    }

    pub fn finish(self) -> ShellCommand {
        self.inner
    }
}

// Functional style API - consumes self, which also gives the nice one liner constructions to start
// Also has 'escape hatch' mutability functions for later modification or inline modification based
// on other application state
impl ShellCommand {
    pub fn new(program: impl Into<String>) -> Self {
        Self { program: program.into(), args: vec![], env: vec![], working_dir: None }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(|a| a.into()));
        self
    }

    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.push((key.into(), val.into()));
        self
    }

    pub fn working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Gives a mutable version for ergonomic assignment of additional values
    pub fn mutate(self) -> ShellCommandMut {
        ShellCommandMut { inner: self }
    }

    /// Scoped mutation via closure — stays in functional style.
    pub fn with(self, f: impl FnOnce(&mut ShellCommandMut)) -> Self {
        let mut handle = ShellCommandMut { inner: self };
        f(&mut handle);
        handle.inner
    }
}

/// Displays how the command would actually be called in shell.
impl Display for ShellCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.program, self.args.join(" "))
    }
}

#[cfg(feature = "tokio")]
impl From<&ShellCommand> for tokio::process::Command {
    fn from(cmd: &ShellCommand) -> Self {
        let mut command = tokio::process::Command::new(&cmd.program);
        command.args(&cmd.args);
        for (key, val) in &cmd.env {
            command.env(key, val);
        }
        if let Some(dir) = &cmd.working_dir {
            command.current_dir(dir);
        }
        command
    }
}

impl From<&ShellCommand> for std::process::Command {
    fn from(cmd: &ShellCommand) -> Self {
        let mut command = std::process::Command::new(&cmd.program);
        command.args(&cmd.args);
        for (key, val) in &cmd.env {
            command.env(key, val);
        }
        if let Some(dir) = &cmd.working_dir {
            command.current_dir(dir);
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluent_one_liner() {
        let cmd = ShellCommand::new("cargo").arg("build").arg("--release").env("RUST_LOG", "debug").working_dir("/tmp");

        assert_eq!(cmd.program, "cargo");
        assert_eq!(cmd.args, vec!["build", "--release"]);
        assert_eq!(cmd.env, vec![("RUST_LOG".into(), "debug".into())]);
        assert_eq!(cmd.working_dir, Some("/tmp".into()));
    }

    #[test]
    fn fluent_batch_args() {
        let cmd = ShellCommand::new("echo").args(["hello", "world"]);

        assert_eq!(cmd.to_string(), "echo hello world");
    }

    #[test]
    fn mutate_for_conditional_args() {
        let release = true;
        let verbose = false;

        let mut cmd = ShellCommand::new("cargo").arg("build").mutate();

        if release {
            cmd.arg("--release");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.env("RUST_LOG", "info");

        let cmd = cmd.finish();

        assert_eq!(cmd.to_string(), "cargo build --release");
        assert_eq!(cmd.env, vec![("RUST_LOG".into(), "info".into())]);
    }

    #[test]
    fn mutate_loop() {
        let extra_args = vec!["--features", "serde,tokio"];

        let mut cmd = ShellCommand::new("cargo").arg("build").mutate();

        for a in extra_args {
            cmd.arg(a);
        }

        let cmd = cmd.finish();
        assert_eq!(cmd.to_string(), "cargo build --features serde,tokio");
    }

    // ── Scoped mutation with .with() ──

    #[test]
    fn with_closure_stays_fluent() {
        let release = true;

        let cmd = ShellCommand::new("cargo")
            .arg("build")
            .with(|cmd| {
                if release {
                    cmd.arg("--release");
                }
                cmd.env("RUST_LOG", "debug");
            })
            .working_dir("/project");

        assert_eq!(cmd.to_string(), "cargo build --release");
        assert_eq!(cmd.working_dir, Some("/project".into()));
    }

    #[test]
    fn with_closure_no_ops() {
        let cmd = ShellCommand::new("ls").arg("-la").with(|_| {
            // nothing to add
        });

        assert_eq!(cmd.to_string(), "ls -la");
    }

    #[test]
    fn with_closure_dynamic_args() {
        let user_flags: Vec<&str> = vec!["-v", "--color=always"];

        let cmd = ShellCommand::new("ls").with(|cmd| {
            for flag in &user_flags {
                cmd.arg(*flag);
            }
        });

        assert_eq!(cmd.to_string(), "ls -v --color=always");
    }

    #[test]
    fn display_matches_to_command() {
        let cmd = ShellCommand::new("ls").arg("-la");
        assert_eq!(format!("{}", cmd), cmd.to_string());
    }

    #[test]
    fn into_process_command() {
        let cmd = ShellCommand::new("echo").arg("hi");
        let mut proc_cmd = std::process::Command::from(&cmd);
        let output = proc_cmd.output().unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn roundtrip_json() {
        let cmd = ShellCommand::new("ls").arg("-la").working_dir("/home");

        let json = serde_json::to_string(&cmd).unwrap();
        let restored: ShellCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, restored);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn into_tokio_command() {
        let cmd = ShellCommand::new("echo").arg("hi");
        let mut tokio_cmd = tokio::process::Command::from(&cmd);
        let output = tokio_cmd.output().await.unwrap();
        assert!(output.status.success());
    }
}
