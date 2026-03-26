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
    fn builder_sets_fields() {
        let cmd = ShellCommand::new("cargo").arg("build").arg("--release").env("RUST_LOG", "debug").working_dir("/tmp");

        assert_eq!(cmd.program, "cargo");
        assert_eq!(cmd.args, vec!["build", "--release"]);
        assert_eq!(cmd.env, vec![("RUST_LOG".into(), "debug".into())]);
        assert_eq!(cmd.working_dir, Some("/tmp".into()));
    }

    #[test]
    fn args_batch() {
        let cmd = ShellCommand::new("echo").args(["hello", "world"]);
        assert_eq!(cmd.args, vec!["hello", "world"]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn roundtrip_json() {
        let cmd = ShellCommand::new("ls").arg("-la").working_dir("/home");
        let json = serde_json::to_string(&cmd).unwrap();
        let restored: ShellCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, restored);
    }

    #[test]
    fn into_process_command() {
        let cmd = ShellCommand::new("echo").arg("hi");
        let mut proc_cmd = std::process::Command::from(&cmd);
        let output = proc_cmd.output().unwrap();
        assert!(output.status.success());
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
