use std::process::{Command, Stdio};
use std::env;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: agx-train <config_path>");
        std::process::exit(1);
    }

    let config_path = &args[1];
    println!("Starting training with config: {}", config_path);

    // Ensure accelerate is installed
    let status = Command::new("accelerate")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if status.is_err() {
        eprintln!("Error: 'accelerate' command not found. Please install it via 'pip install accelerate'.");
        std::process::exit(1);
    }

    // Run training
    let mut child = Command::new("accelerate")
        .arg("launch")
        .arg("-m")
        .arg("axolotl.cli.train")
        .arg(config_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let exit_status = child.wait()?;

    if !exit_status.success() {
        eprintln!("Training failed with exit code: {}", exit_status);
        std::process::exit(exit_status.code().unwrap_or(1));
    }

    println!("Training completed successfully.");
    Ok(())
}
