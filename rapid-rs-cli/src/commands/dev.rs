use std::process::Command;

pub async fn start_dev_server(port: u16) -> anyhow::Result<()> {
    println!("🔥 Starting development server with hot reload on port {}...", port);
    println!("💡 Watching for file changes...\n");

    // Check if cargo-watch is installed
    let watch_check = Command::new("cargo")
        .args(&["watch", "--version"])
        .output();

    if watch_check.is_err() {
        println!("⚠️  cargo-watch not found. Installing...");
        Command::new("cargo")
            .args(&["install", "cargo-watch"])
            .status()?;
    }

    // Run with cargo-watch
    let status = Command::new("cargo")
        .args(&[
            "watch",
            "-x",
            "run",
            "-w",
            "src",
            "-w",
            "Cargo.toml",
        ])
        .env("APP_PORT", port.to_string())
        .status()?;

    if !status.success() {
        anyhow::bail!("Development server exited with error");
    }

    Ok(())
}
