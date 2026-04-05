use std::fmt::{Display, Formatter};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ShellCommand {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub working_dir: Option<String>,
}

pub struct ShellCommandBuilder {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    working_dir: Option<String>,
}

impl ShellCommandBuilder {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: vec![],
            env: vec![],
            working_dir: None,
        }
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

    pub fn finish(self) -> ShellCommand {
        ShellCommand {
            program: self.program,
            args: self.args,
            env: self.env,
            working_dir: self.working_dir,
        }
    }
}

impl ShellCommand {
    pub fn build(program: impl Into<String>) -> ShellCommandBuilder {
        ShellCommandBuilder::new(program)
    }

    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl Into<String>>) -> &mut Self {
        self.args.extend(args.into_iter().map(|a| a.into()));
        self
    }

    pub fn env(&mut self, key: impl Into<String>, val: impl Into<String>) -> &mut Self {
        self.env.push((key.into(), val.into()));
        self
    }

    pub fn working_dir(&mut self, dir: impl Into<String>) -> &mut Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn to_command(&self) -> String {
        format!("{} {}", self.program, self.args.join(" "))
    }
}

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
    fn builder_one_liner() {
        let cmd = ShellCommand::build("cargo")
            .arg("build")
            .arg("--release")
            .env("RUST_LOG", "debug")
            .working_dir("/tmp")
            .finish();

        assert_eq!(cmd.program, "cargo");
        assert_eq!(cmd.args, vec!["build", "--release"]);
        assert_eq!(cmd.env, vec![("RUST_LOG".into(), "debug".into())]);
        assert_eq!(cmd.working_dir, Some("/tmp".into()));
    }

    #[test]
    fn builder_then_mutate() {
        let mut cmd = ShellCommand::build("which")
            .args(vec!["a", "b", "c"])
            .finish();

        cmd.arg("d").arg("e");

        assert_eq!(cmd.to_command(), "which a b c d e");
    }

    #[test]
    fn conditional_mutation() {
        let mut cmd = ShellCommand::build("cargo")
            .arg("build")
            .finish();

        let release = true;
        if release {
            cmd.arg("--release");
        }

        let verbose = false;
        if verbose {
            cmd.arg("--verbose");
        }

        assert_eq!(cmd.to_command(), "cargo build --release");
    }

    #[test]
    fn builder_with_batch_args() {
        let cmd = ShellCommand::build("echo")
            .args(["hello", "world"])
            .finish();

        assert_eq!(cmd.args, vec!["hello", "world"]);
    }

    #[test]
    fn display_matches_to_command() {
        let cmd = ShellCommand::build("ls")
            .arg("-la")
            .finish();

        assert_eq!(format!("{}", cmd), cmd.to_command());
    }

    #[test]
    fn mutate_env_after_build() {
        let mut cmd = ShellCommand::build("cargo")
            .arg("test")
            .finish();

        cmd.env("RUST_LOG", "trace");
        cmd.env("RUST_BACKTRACE", "1");

        assert_eq!(cmd.env.len(), 2);
    }

    #[test]
    fn into_process_command() {
        let cmd = ShellCommand::build("echo")
            .arg("hi")
            .finish();

        let mut proc_cmd = std::process::Command::from(&cmd);
        let output = proc_cmd.output().unwrap();
        assert!(output.status.success());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn roundtrip_json() {
        let cmd = ShellCommand::build("ls")
            .arg("-la")
            .working_dir("/home")
            .finish();

        let json = serde_json::to_string(&cmd).unwrap();
        let restored: ShellCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, restored);
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn into_tokio_command() {
        let cmd = ShellCommand::build("echo")
            .arg("hi")
            .finish();

        let mut tokio_cmd = tokio::process::Command::from(&cmd);
        let output = tokio_cmd.output().await.unwrap();
        assert!(output.status.success());
    }
}