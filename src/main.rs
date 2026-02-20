use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// --- Config ---

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "default_brightness")]
    brightness: u8,
    #[serde(default = "default_temperature")]
    temperature: u16,
    light: Option<String>,
    ip_address: Option<String>,
}

fn default_brightness() -> u8 {
    10
}
fn default_temperature() -> u16 {
    5000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            brightness: default_brightness(),
            temperature: default_temperature(),
            light: None,
            ip_address: None,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".config/elgato-autolight/config.toml"))
}

fn load_config() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };

    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Warning: failed to parse {}: {}", path.display(), e);
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

// --- LaunchAgent ---

const LABEL: &str = "com.wassimk.elgato-autolight";

fn plist_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Library/LaunchAgents/com.wassimk.elgato-autolight.plist")
}

fn log_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join("Library/Logs/elgato-autolight")
}

fn current_uid() -> String {
    let output = Command::new("id").arg("-u").output().expect("failed to run id -u");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn generate_plist(binary_path: &str) -> String {
    let log_dir = log_dir();
    let stdout_log = log_dir.join("stdout.log");
    let stderr_log = log_dir.join("stderr.log");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary_path}</string>
        <string>start</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{stdout}</string>
    <key>StandardErrorPath</key>
    <string>{stderr}</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
</dict>
</plist>"#,
        stdout = stdout_log.display(),
        stderr = stderr_log.display(),
    )
}

fn install_launchagent(force: bool) -> Result<()> {
    let plist = plist_path();

    if plist.exists() && !force {
        anyhow::bail!(
            "LaunchAgent already installed at {}\nUse --force to overwrite.",
            plist.display()
        );
    }

    let binary_path = std::env::current_exe()
        .context("Failed to determine binary path")?
        .to_string_lossy()
        .to_string();

    // Unload existing agent if overwriting
    if plist.exists() {
        let _ = Command::new("launchctl")
            .args(["bootout", &format!("gui/{}/{LABEL}", current_uid())])
            .output();
    }

    // Create log directory
    std::fs::create_dir_all(log_dir()).context("Failed to create log directory")?;

    // Write plist
    let content = generate_plist(&binary_path);
    std::fs::write(&plist, content)
        .with_context(|| format!("Failed to write plist to {}", plist.display()))?;

    // Load agent
    let output = Command::new("launchctl")
        .args(["bootstrap", &format!("gui/{}", current_uid()), &plist.to_string_lossy()])
        .output()
        .context("Failed to run launchctl bootstrap")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("launchctl bootstrap failed: {}", stderr.trim());
    }

    println!("LaunchAgent installed and loaded.");
    println!("  Plist: {}", plist.display());
    println!("  Logs:  {}", log_dir().display());
    Ok(())
}

fn uninstall_launchagent() -> Result<()> {
    // Unload (ignore errors if not loaded)
    let _ = Command::new("launchctl")
        .args(["bootout", &format!("gui/{}/{LABEL}", current_uid())])
        .output();

    let plist = plist_path();
    if plist.exists() {
        std::fs::remove_file(&plist)
            .with_context(|| format!("Failed to remove {}", plist.display()))?;
        println!("LaunchAgent uninstalled.");
    } else {
        println!("LaunchAgent was not installed.");
    }

    Ok(())
}

fn stop_launchagent() -> Result<()> {
    let target = format!("gui/{}/{LABEL}", current_uid());

    let output = Command::new("launchctl")
        .args(["bootout", &target])
        .output()
        .context("Failed to run launchctl bootout")?;

    if output.status.success() {
        println!("Service stopped.");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No such process") || stderr.contains("Could not find service") {
            println!("Service is not running.");
        } else {
            anyhow::bail!("Failed to stop service: {}", stderr.trim());
        }
    }

    Ok(())
}

fn restart_launchagent() -> Result<()> {
    let target = format!("gui/{}/{LABEL}", current_uid());

    let output = Command::new("launchctl")
        .args(["kickstart", "-k", &target])
        .output()
        .context("Failed to run launchctl kickstart")?;

    if output.status.success() {
        println!("Service restarted.");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to restart service: {}", stderr.trim());
    }

    Ok(())
}

fn show_status() -> Result<()> {
    let config = load_config();

    // Check if loaded via launchctl
    let output = Command::new("launchctl")
        .args(["list", LABEL])
        .output()
        .context("Failed to run launchctl list")?;

    let running = output.status.success();
    let plist = plist_path();
    let installed = plist.exists();

    println!("Service:     {LABEL}");
    println!("Installed:   {}", if installed { "yes" } else { "no" });
    println!("Running:     {}", if running { "yes" } else { "no" });
    println!();
    println!("Config:");
    println!("  Brightness:   {}%", config.brightness);
    println!("  Temperature:  {}K", config.temperature);
    if let Some(ref light) = config.light {
        println!("  Light:        {}", light);
    }
    if let Some(ref ip) = config.ip_address {
        println!("  IP Address:   {}", ip);
    }
    println!();
    println!("Paths:");
    println!(
        "  Config: {}",
        config_path().map_or("N/A".into(), |p| p.to_string_lossy().to_string())
    );
    println!("  Plist:  {}", plist.display());
    println!("  Logs:   {}", log_dir().display());

    Ok(())
}

// --- Monitor ---

fn find_elgato_light() -> Option<PathBuf> {
    // Try PATH first
    if let Ok(output) = Command::new("which").arg("elgato-light").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    // Fallback locations
    for path in ["/opt/homebrew/bin/elgato-light", "/usr/local/bin/elgato-light"] {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

fn run_light_command(binary: &PathBuf, config: &Config, action: &str) {
    let mut cmd = Command::new(binary);
    cmd.arg(action);

    if action == "on" {
        cmd.args(["--brightness", &config.brightness.to_string()]);
        cmd.args(["--temperature", &config.temperature.to_string()]);
    }

    if let Some(ref light) = config.light {
        cmd.args(["--light", light]);
    }
    if let Some(ref ip) = config.ip_address {
        cmd.args(["--ip-address", ip]);
    }

    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("elgato-light {action} failed: {}", stderr.trim());
            }
        }
        Err(e) => eprintln!("Failed to run elgato-light: {e}"),
    }
}

fn run_monitor(verbose: bool) -> Result<()> {
    let config = load_config();

    let binary = find_elgato_light().ok_or_else(|| {
        anyhow::anyhow!(
            "elgato-light not found on PATH or in /opt/homebrew/bin or /usr/local/bin.\n\
             Install it with: brew install wassimk/tap/elgato-light"
        )
    })?;

    eprintln!("Using elgato-light at: {}", binary.display());
    eprintln!(
        "Settings: brightness={}%, temperature={}K",
        config.brightness, config.temperature
    );

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_flag = shutdown.clone();

    ctrlc::set_handler(move || {
        shutdown_flag.store(true, Ordering::SeqCst);
    })
    .context("Failed to set signal handler")?;

    eprintln!("Monitoring camera events...");

    while !shutdown.load(Ordering::SeqCst) {
        match spawn_log_stream() {
            Ok(mut child) => {
                let stdout = child.stdout.take().expect("stdout was piped");
                let reader = BufReader::new(stdout);

                for line in reader.lines() {
                    if shutdown.load(Ordering::SeqCst) {
                        let _ = child.kill();
                        break;
                    }

                    let line = match line {
                        Ok(l) => l,
                        Err(e) => {
                            eprintln!("Error reading log stream: {e}");
                            break;
                        }
                    };

                    if verbose {
                        eprintln!("[log] {line}");
                    }

                    if line.contains("= On") {
                        eprintln!("Camera ON - turning light on");
                        run_light_command(&binary, &config, "on");
                    } else if line.contains("= Off") {
                        eprintln!("Camera OFF - turning light off");
                        run_light_command(&binary, &config, "off");
                    }
                }

                let _ = child.kill();
                let _ = child.wait();
            }
            Err(e) => {
                eprintln!("Failed to start log stream: {e}");
            }
        }

        if !shutdown.load(Ordering::SeqCst) {
            eprintln!("Log stream ended, restarting in 2s...");
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    eprintln!("Shutting down.");
    Ok(())
}

fn spawn_log_stream() -> Result<std::process::Child> {
    Command::new("log")
        .args([
            "stream",
            "--predicate",
            "subsystem == \"com.apple.UVCExtension\" and composedMessage contains \"Post PowerLog\"",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn 'log stream'")
}

// --- CLI ---

#[derive(Parser, Debug)]
#[command(
    name = "elgato-autolight",
    about = "Automatically toggle Elgato lights when your Mac camera activates",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run the camera monitor in the foreground
    Start {
        #[arg(short, long, help = "Print every log stream line received")]
        verbose: bool,
    },
    /// Install the LaunchAgent for automatic startup
    Install {
        #[arg(short, long, help = "Overwrite existing LaunchAgent")]
        force: bool,
    },
    /// Uninstall the LaunchAgent
    Uninstall,
    /// Stop the background service
    Stop,
    /// Restart the background service
    Restart,
    /// Show running state, config, and log paths
    Status,
}

// --- main ---

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Cmd::Start { verbose } => run_monitor(verbose),
        Cmd::Install { force } => install_launchagent(force),
        Cmd::Uninstall => uninstall_launchagent(),
        Cmd::Stop => stop_launchagent(),
        Cmd::Restart => restart_launchagent(),
        Cmd::Status => show_status(),
    }
}
