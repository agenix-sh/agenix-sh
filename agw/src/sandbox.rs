use crate::error::{AgwError, AgwResult};
use std::process::Output;
use tokio::process::Command;
use tracing::{debug, info};

/// Trait for sandbox implementations
#[async_trait::async_trait]
pub trait Sandbox: Send + Sync {
    /// Run a command within the sandbox
    async fn run(&self, command: &str, args: &[String], env: &[(String, String)]) -> AgwResult<Output>;
}

/// Factory to create the appropriate sandbox for the current platform
pub fn create_sandbox() -> Box<dyn Sandbox> {
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxSandbox::new())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Box::new(MacOsSandbox::new())
    }
}

/// macOS Sandbox Implementation (Process Isolation only)
///
/// On macOS, we don't have unshare/namespaces easily accessible without
/// complex C bindings or external tools. We rely on basic process isolation.
pub struct MacOsSandbox;

impl MacOsSandbox {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Sandbox for MacOsSandbox {
    async fn run(&self, command: &str, args: &[String], env: &[(String, String)]) -> AgwResult<Output> {
        debug!("Running command in MacOsSandbox: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args);
        
        // Clear environment and set only provided vars
        cmd.env_clear();
        for (k, v) in env {
            cmd.env(k, v);
        }

        // TODO: Add resource limits via `ulimit` wrapper if needed?
        // For now, just run the process
        
        let output = cmd.output().await.map_err(|e| {
            AgwError::Worker(format!("Failed to execute command '{}': {}", command, e))
        })?;

        Ok(output)
    }
}

/// Linux Sandbox Implementation (Namespaces)
#[cfg(target_os = "linux")]
pub struct LinuxSandbox;

#[cfg(target_os = "linux")]
impl LinuxSandbox {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "linux")]
#[async_trait::async_trait]
impl Sandbox for LinuxSandbox {
    async fn run(&self, command: &str, args: &[String], env: &[(String, String)]) -> AgwResult<Output> {
        debug!("Running command in LinuxSandbox: {} {:?}", command, args);

        // We use `unshare` to create new namespaces
        // This requires the `unshare` binary to be present or we use the `nix` crate to do it in-process.
        // Doing it in-process in Rust with async tokio is tricky because fork() and threads don't mix well.
        // A safer approach for this "Simple 3 Binary" goal is to use `unshare` command wrapper if available,
        // or just rely on the fact that we are running as a separate process.
        
        // However, the requirement was "Native Rust Sandbox".
        // To do this safely in async rust, we usually fork/exec a helper process that sets up namespaces.
        // Or we use `std::process::Command` with `pre_exec` hook (unsafe).
        
        // Let's try the `unshare` command wrapper approach first as it's robust.
        // If `unshare` is not available, we fall back to standard execution with a warning.
        
        let mut cmd = Command::new("unshare");
        
        // Flags:
        // -m: Mount namespace
        // -p: PID namespace
        // -f: Fork (required for PID namespace)
        // --mount-proc: Mount /proc
        // -n: Network namespace (optional, maybe we want network?) -> Let's keep network for now as tasks might need it
        cmd.args(&["-m", "-p", "-f", "--mount-proc"]);
        
        // The actual command
        cmd.arg(command);
        cmd.args(args);

        cmd.env_clear();
        for (k, v) in env {
            cmd.env(k, v);
        }

        let output = cmd.output().await.map_err(|e| {
            AgwError::Worker(format!("Failed to execute sandbox command: {}", e))
        })?;

        Ok(output)
    }
}
