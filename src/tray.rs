//! `kept tray`: the Kept mark in the menu bar (macOS) or the system tray
//! (Linux, Windows), in colour while the warm process runs and grey otherwise, with a
//! menu to reindex, open the notes, start or stop the process. Built with the `tray`
//! feature only. `kept tray install` starts it at login.
//!
//! macOS and Windows go through tray-icon and a tao event loop. Linux speaks the
//! StatusNotifierItem protocol over D-Bus (ksni, pure Rust): KDE, and GNOME with the
//! AppIndicator extension, show it without GTK on either side.

use crate::{root, socket_path, spawn_daemon, ui};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
#[cfg(not(target_os = "linux"))]
use std::time::Instant;
#[cfg(not(target_os = "linux"))]
use tao::event::{Event, StartCause};
#[cfg(not(target_os = "linux"))]
use tao::event_loop::{ControlFlow, EventLoop};
#[cfg(not(target_os = "linux"))]
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
#[cfg(not(target_os = "linux"))]
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const EVERY: Duration = Duration::from_secs(5);
const IDLE_LINE: &str = "Idle: the next search starts the warm process";
const LABEL: &str = "io.github.codexofc.kept";

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

#[cfg(target_os = "linux")]
pub fn run() -> Result<(), String> {
    use ksni::blocking::TrayMethods;
    use ksni::menu::{MenuItem, StandardItem};

    struct Item {
        running: Option<String>,
        quit: bool,
    }

    impl ksni::Tray for Item {
        fn id(&self) -> String {
            "kept".into()
        }
        fn title(&self) -> String {
            "Kept".into()
        }
        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            // The same mark as on macOS, ARGB in network byte order.
            let rgba = ui::logo_rgba(64, if self.running.is_some() { None } else { Some([235, 235, 235]) });
            let data = rgba.as_chunks::<4>().0.iter().flat_map(|p| [p[3], p[0], p[1], p[2]]).collect();
            vec![ksni::Icon { width: 64, height: 64, data }]
        }
        fn menu(&self) -> Vec<MenuItem<Self>> {
            let up = self.running.is_some();
            vec![
                StandardItem { label: self.running.clone().unwrap_or_else(|| IDLE_LINE.into()), enabled: false, ..Default::default() }.into(),
                MenuItem::Separator,
                StandardItem { label: "Reindex now".into(), activate: Box::new(|_| detached(&["index"])), ..Default::default() }.into(),
                StandardItem { label: "Open the notes folder".into(), activate: Box::new(|_| open_notes()), ..Default::default() }.into(),
                MenuItem::Separator,
                StandardItem { label: "Start the warm process".into(), enabled: !up, activate: Box::new(|_| spawn_daemon()), ..Default::default() }.into(),
                StandardItem { label: "Stop the warm process".into(), enabled: up, activate: Box::new(|_| detached(&["stop"])), ..Default::default() }.into(),
                MenuItem::Separator,
                StandardItem { label: "Quit".into(), activate: Box::new(|item: &mut Self| item.quit = true), ..Default::default() }.into(),
            ]
        }
    }

    let handle = Item { running: status_line(), quit: false }
        .spawn()
        .map_err(|e| format!("no status notifier host on this desktop (KDE, or GNOME with the AppIndicator extension): {e}"))?;
    loop {
        std::thread::sleep(EVERY);
        let line = status_line();
        match handle.update(|item| {
            item.running = line;
            item.quit
        }) {
            Some(false) => {}
            _ => break,
        }
    }
    handle.shutdown().wait();
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn run() -> Result<(), String> {
    let mut event_loop: EventLoop<()> = EventLoop::new();
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }
    let status = MenuItem::new("Kept", false, None);
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
    let mut shown = false;
    let refresh = {
        let (status, start, stop, lit, grey) = (status.clone(), start.clone(), stop.clone(), lit.clone(), grey.clone());
        move |tray: &Option<TrayIcon>, running: &mut Option<bool>| {
            let line = status_line();
            let is_up = line.is_some();
            status.set_text(line.as_deref().unwrap_or(IDLE_LINE));
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
                    let _ = t.set_tooltip(Some(if is_up { "Kept: running" } else { "Kept: idle" }));
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
                    .with_tooltip("Kept")
                    .build()
                    .ok();
                refresh(&tray, &mut running);
                *control_flow = ControlFlow::WaitUntil(Instant::now() + EVERY);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                refresh(&tray, &mut running);
                // KEPT_TRAY_SHOW_MENU opens the menu once the item is up: for the
                // documentation captures, nothing else.
                if std::env::var_os("KEPT_TRAY_SHOW_MENU").is_some() && !shown {
                    if let Some(t) = &tray {
                        if let Some(r) = t.rect() {
                            println!("item at {:?} size {:?}", r.position, r.size);
                        }
                        t.show_menu();
                    }
                    shown = true;
                }
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
        crate::paths::home().join(".config/autostart/kept.desktop")
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
        format!("[Desktop Entry]\nType=Application\nName=Kept\nExec={} tray\nX-GNOME-Autostart-enabled=true\n", exe.display())
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
