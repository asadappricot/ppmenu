use anyhow::{Result, anyhow};
use argh::FromArgs;
use notify_rust::Notification;
use std::io::Write;
use std::process::{Command, Stdio};
use zbus::blocking::Connection;
use zbus::blocking::Proxy;

#[derive(FromArgs)]
/// Power profile selector for power-profiles-daemon (or tlp-pd) using a dmenu-compatible launcher
struct PPPArgs {
    /// launcher to use (supported launchers are fuzzel, dmenu, rofi)
    #[argh(option, short = 'l')]
    launcher: String,

    /// custom arguments to pass to the launcher
    #[argh(option, short = 'a')]
    launcher_args: Option<String>,
}

struct PowerProfile {
    name: String,
    entry: String
}

fn get_command(launcher: &str, current_profile: &str, custom_args: Option<&str>) -> Command {
    let mut cmd = match launcher {
        "fuzzel" => {
            let mut cmd = Command::new("fuzzel");
            cmd.arg("--dmenu")
                .arg("--index")
                .arg("--placeholder")
                .arg(&format!("Current profile: {}", current_profile));
            cmd
        },
        "rofi" => {
            let mut cmd = Command::new("rofi");
            cmd.arg("-dmenu")
                .arg("-i")
                .arg("-p")
                .arg(&format!("Current profile: {}", current_profile));
            cmd
        },
        _ => {
            let mut cmd = Command::new("dmenu");
            cmd.arg("-p")
                .arg(&format!("Current profile: {}", current_profile));
            cmd
        },
    };
    
    // Add custom arguments if provided
    if let Some(args) = custom_args {
        for arg in args.split_whitespace() {
            cmd.arg(arg);
        }
    }
    
    cmd
}

fn main() -> Result<()> {
    let profiles: Vec<PowerProfile> = vec![
        PowerProfile {name: String::from("power-saver"), entry: String::from("󰌪 | Power Saver")},
        PowerProfile {name: String::from("balanced"), entry: String::from("󰾅 | Balanced")},
        PowerProfile {name: String::from("performance"), entry: String::from("󱐋 | Performance")}
    ];

    let connection = Connection::system()?;
    
    let proxy = Proxy::new(
        &connection,
        "org.freedesktop.UPower.PowerProfiles",
        "/org/freedesktop/UPower/PowerProfiles",
        "org.freedesktop.UPower.PowerProfiles"
    )?;

    let current_profile: String = proxy.get_property("ActiveProfile")?;

    let args: PPPArgs = argh::from_env();

    let valid_launchers = ["fuzzel", "dmenu", "rofi"];
    if !valid_launchers.contains(&args.launcher.as_str()) {
        anyhow::bail!(
            "Invalid launcher '{}'. Must be one of: {}",
            args.launcher,
            valid_launchers.join(", ")
        );
    }

    let mut dmenu_proc = get_command(&args.launcher, &current_profile, args.launcher_args.as_deref())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let dmenu_stdin = dmenu_proc.stdin.as_mut().unwrap();

    let input = profiles.iter()
        .map(|p| &p.entry)
        .fold(String::new(), |a, b| a + b + "\n");
    
    dmenu_stdin.write_all(input.as_bytes())?;

    let output = dmenu_proc.wait_with_output()?;
    
    let index = match args.launcher.as_str() {
        "fuzzel" | "rofi" => String::from_utf8(output.stdout)?
                                .trim()
                                .parse::<usize>()?,
        "dmenu" => { // dmenu doesn't support indexing out of the box
            let selected_entry = String::from_utf8(output.stdout)?
                .trim()
                .to_string();
        
            profiles.iter()
                .position(|p| p.entry == selected_entry)
                .ok_or_else(|| anyhow!("Selected entry not found"))?
        },
        &_ => todo!()
    };

    let new_profile = &profiles[index].name;

    match proxy.set_property("ActiveProfile", new_profile) {
        Ok(()) => {
            Notification::new()
                .summary("Power Profile Picker")
                .body(&format!("Power profile set to {}", new_profile))
                .show()?;
        },
        Err(e) => {
            Notification::new()
                .summary("Power Profile Picker")
                .body(&format!("Unable to set power profile: {:?}", e))
                .show()?;
        }
    }
     
    Ok(())
}
