//! `engram tray`: the Engrams mark in the menu bar (macOS) or the system tray
//! (Linux, Windows), in colour while the warm process runs and grey otherwise, with a
//! menu to reindex, open the notes, start or stop the process. Built with the `tray`
//! feature only. `engram tray install` starts it at login.

use crate::{root, socket_path, spawn_daemon, ui};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoop};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const EVERY: Duration = Duration::from_secs(5);
const LABEL: &str = "io.github.codexofc.engrams";

/// The warm process seen from its socket: one line, or None when it is down.
fn status_line() -> Option<String> {
    let mut s = std::os::unix::net::UnixStream::connect(socket_path()).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    s.write_all(b"status\n").ok()?;
    s.shutdown(std::net::Shutdown::Write).ok()?;
    let mut reply = String::new();
    s.read_to_string(&mut reply).ok()?;
    let field = |k: &str| reply.lines().find_map(|l| l.strip_prefix(k).and_then(|r| r.strip_prefix('\t'))).unwrap_or("0");
    let up = field("uptime_s").parse::<u64>().unwrap_or(0);
    Some(format!(
        "Running: up {}h{:02}, {} requests, {} MB",
        up / 3600,
        (up % 3600) / 60,
        field("requests"),
        field("rss_kb").parse::<u64>().unwrap_or(0) / 1024
    ))
}

fn detached(args: &[&str]) {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

fn open_notes() {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let _ = std::process::Command::new(opener).arg(root()).spawn();
}

pub fn run() -> Result<(), String> {
    let mut event_loop: EventLoop<()> = EventLoop::new();
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }
    let status = MenuItem::new("Engrams", false, None);
    let reindex = MenuItem::new("Reindex now", true, None);
    let open = MenuItem::new("Open the notes folder", true, None);
    let start = MenuItem::new("Start the warm process", true, None);
    let stop = MenuItem::new("Stop the warm process", true, None);
    let quit = MenuItem::new("Quit", true, None);
    let menu = Menu::new();
    menu.append_items(&[
        &status,
        &PredefinedMenuItem::separator(),
        &reindex,
        &open,
        &PredefinedMenuItem::separator(),
        &start,
        &stop,
        &PredefinedMenuItem::separator(),
        &quit,
    ])
    .map_err(|e| e.to_string())?;
    let size = 44u32;
    let lit = Icon::from_rgba(ui::logo_rgba(size as usize, None), size, size).map_err(|e| e.to_string())?;
    // Idle: a template icon on macOS, which the menu bar paints white or black by
    // itself, and a white mark elsewhere.
    let idle_rgb = if cfg!(target_os = "macos") { [0, 0, 0] } else { [235, 235, 235] };
    let grey = Icon::from_rgba(ui::logo_rgba(size as usize, Some(idle_rgb)), size, size).map_err(|e| e.to_string())?;
    let mut tray: Option<TrayIcon> = None;
    let mut running = None;
    let refresh = {
        let (status, start, stop, lit, grey) = (status.clone(), start.clone(), stop.clone(), lit.clone(), grey.clone());
        move |tray: &Option<TrayIcon>, running: &mut Option<bool>| {
            let line = status_line();
            let is_up = line.is_some();
            status.set_text(line.as_deref().unwrap_or("Idle: the next search starts the warm process"));
            start.set_enabled(!is_up);
            stop.set_enabled(is_up);
            if *running != Some(is_up) {
                if let Some(t) = tray {
                    let icon = if is_up { lit.clone() } else { grey.clone() };
                    if cfg!(target_os = "macos") {
                        let _ = t.set_icon_with_as_template(Some(icon), !is_up);
                    } else {
                        let _ = t.set_icon(Some(icon));
                    }
                    let _ = t.set_tooltip(Some(if is_up { "Engrams: running" } else { "Engrams: idle" }));
                }
                *running = Some(is_up);
            }
        }
    };
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                // macOS wants the item created once the loop runs, not before.
                tray = TrayIconBuilder::new()
                    .with_menu(Box::new(menu.clone()))
                    .with_icon(grey.clone())
                    .with_icon_as_template(true)
                    .with_tooltip("Engrams")
                    .build()
                    .ok();
                refresh(&tray, &mut running);
                *control_flow = ControlFlow::WaitUntil(Instant::now() + EVERY);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                refresh(&tray, &mut running);
                *control_flow = ControlFlow::WaitUntil(Instant::now() + EVERY);
            }
            _ => {}
        }
        while let Ok(e) = MenuEvent::receiver().try_recv() {
            let id = e.id();
            if id == reindex.id() {
                detached(&["index"]);
            } else if id == open.id() {
                open_notes();
            } else if id == start.id() {
                spawn_daemon();
                *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_secs(2));
            } else if id == stop.id() {
                detached(&["stop"]);
                *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_secs(1));
            } else if id == quit.id() {
                *control_flow = ControlFlow::Exit;
            }
        }
    })
}

fn launch_file() -> PathBuf {
    if cfg!(target_os = "macos") {
        crate::paths::home().join("Library/LaunchAgents").join(format!("{LABEL}.plist"))
    } else {
        crate::paths::home().join(".config/autostart/engrams.desktop")
    }
}

/// Starts the tray at login: a launchd agent on macOS, an XDG autostart entry elsewhere.
pub fn install() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let file = launch_file();
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let body = if cfg!(target_os = "macos") {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key><string>{LABEL}</string>\n  <key>ProgramArguments</key><array><string>{}</string><string>tray</string></array>\n  <key>RunAtLoad</key><true/>\n</dict>\n</plist>\n",
            exe.display()
        )
    } else {
        format!("[Desktop Entry]\nType=Application\nName=Engrams\nExec={} tray\nX-GNOME-Autostart-enabled=true\n", exe.display())
    };
    std::fs::write(&file, body).map_err(|e| format!("{}: {e}", file.display()))?;
    if cfg!(target_os = "macos") {
        let domain = format!("gui/{}", unsafe { libc::getuid() });
        let _ = std::process::Command::new("launchctl").args(["bootout", &domain, &file.display().to_string()]).output();
        let _ = std::process::Command::new("launchctl").args(["bootstrap", &domain, &file.display().to_string()]).output();
    } else {
        detached(&["tray"]);
    }
    ui::done(&format!("tray starts at login: {}", file.display()));
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let file = launch_file();
    if cfg!(target_os = "macos") {
        let domain = format!("gui/{}", unsafe { libc::getuid() });
        let _ = std::process::Command::new("launchctl").args(["bootout", &domain, &file.display().to_string()]).output();
    }
    match std::fs::remove_file(&file) {
        Ok(()) => ui::done(&format!("removed {}", file.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => ui::note("the tray was not installed"),
        Err(e) => return Err(format!("{}: {e}", file.display())),
    }
    Ok(())
}
