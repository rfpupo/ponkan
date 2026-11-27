//! Native desktop lifecycle and preferences.
use crate::{profiles, trash};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    },
    time::Duration,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt;
type R<T> = Result<T, String>;
static PREF_LOCK: Mutex<()> = Mutex::new(());
static BUSY: AtomicUsize = AtomicUsize::new(0);
static EXITING: AtomicBool = AtomicBool::new(false);
pub struct Operation<'a> {
    busy: &'a AtomicUsize,
    file: Option<fs::File>,
}
impl Drop for Operation<'_> {
    fn drop(&mut self) {
        drop(self.file.take());
        self.busy.store(0, Ordering::SeqCst);
    }
}
fn acquire_operation<'a>(busy: &'a AtomicUsize, exiting: &AtomicBool) -> R<Operation<'a>> {
    if exiting.load(Ordering::SeqCst) {
        return Err("O aplicativo está encerrando.".into());
    }
    busy.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .map_err(|_| "Outra operação está em andamento.")?;
    let guard = Operation { busy, file: None };
    if exiting.load(Ordering::SeqCst) {
        return Err("O aplicativo está encerrando.".into());
    }
    Ok(guard)
}
pub fn operation() -> R<Operation<'static>> {
    let mut guard = acquire_operation(&BUSY, &EXITING)?;
    guard.file = Some(crate::storage_lock::acquire_at(
        &profiles::data_root()?,
        "library.lock",
    )?);
    profiles::recover_saves()?;
    Ok(guard)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub start_minimized: bool,
    pub close_to_tray: bool,
    pub retention_days: u16,
    pub language: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            start_minimized: true,
            close_to_tray: true,
            retention_days: 7,
            language: "pt-BR".into(),
        }
    }
}
#[derive(Serialize)]
pub struct DesktopState {
    #[serde(flatten)]
    pub preferences: Preferences,
    pub autostart: bool,
}
fn path() -> R<std::path::PathBuf> {
    Ok(profiles::data_root()?.join("desktop-preferences.json"))
}
fn read() -> R<Preferences> {
    let p = path()?;
    if !p.exists() {
        return Ok(Preferences::default());
    }
    let b = fs::read(p).map_err(|e| e.to_string())?;
    let s: Preferences =
        serde_json::from_slice(&b).map_err(|_| "Preferências de desktop inválidas.")?;
    if ![0, 1, 7, 15, 30, 60, 90].contains(&s.retention_days)
        || !["pt-BR", "en-US", "es-ES"].contains(&s.language.as_str())
    {
        return Err("Preferências de desktop inválidas.".into());
    }
    Ok(s)
}
pub fn get(app: &AppHandle) -> R<DesktopState> {
    let _g = PREF_LOCK
        .lock()
        .map_err(|_| "Preferências indisponíveis.")?;
    Ok(DesktopState {
        preferences: read()?,
        autostart: app.autolaunch().is_enabled().map_err(|e| e.to_string())?,
    })
}
struct TrayMenu {
    open: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}
fn labels(lang: &str) -> [&'static str; 3] {
    match lang {
        "en-US" => ["Open pon//kan.", "Settings", "Quit"],
        "es-ES" => ["Abrir pon//kan.", "Configuración", "Salir"],
        _ => ["Abrir pon//kan.", "Configurações", "Encerrar"],
    }
}
pub fn update(app: &AppHandle, key: &str, value: serde_json::Value) -> R<DesktopState> {
    let _g = PREF_LOCK
        .lock()
        .map_err(|_| "Preferências indisponíveis.")?;
    let mut s = read()?;
    match key {
        "autostart" => {
            if value.as_bool().ok_or("Preferência inválida.")? {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            }
            .map_err(|e| e.to_string())?;
        }
        "start_minimized" => s.start_minimized = value.as_bool().ok_or("Preferência inválida.")?,
        "close_to_tray" => s.close_to_tray = value.as_bool().ok_or("Preferência inválida.")?,
        "retention_days" => {
            s.retention_days = value
                .as_u64()
                .and_then(|n| u16::try_from(n).ok())
                .filter(|n| [0, 1, 7, 15, 30, 60, 90].contains(n))
                .ok_or("Prazo inválido.")?
        }
        "language" => {
            s.language = value
                .as_str()
                .filter(|x| ["pt-BR", "en-US", "es-ES"].contains(x))
                .ok_or("Idioma inválido.")?
                .into()
        }
        _ => return Err("Preferência inválida.".into()),
    }
    if key != "autostart" {
        trash::atomic_json(&path()?, &s)?;
    }
    if key == "language" {
        if let Some(m) = app.try_state::<TrayMenu>() {
            let l = labels(&s.language);
            let _ = m.open.set_text(l[0]);
            let _ = m.settings.set_text(l[1]);
            let _ = m.quit.set_text(l[2]);
        }
    }
    Ok(DesktopState {
        preferences: s,
        autostart: app.autolaunch().is_enabled().map_err(|e| e.to_string())?,
    })
}
pub fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}
pub fn quit(app: &AppHandle) {
    if EXITING.swap(true, Ordering::SeqCst) {
        return;
    }
    if BUSY.load(Ordering::SeqCst) > 0 {
        EXITING.store(false, Ordering::SeqCst);
        show(app);
        let _ = app.emit("ponkan-desktop-busy", ());
        return;
    }
    app.exit(0)
}
pub fn close(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.label() != "main" {
            return;
        }
        api.prevent_close();
        if read().map(|s| s.close_to_tray).unwrap_or(true) {
            let _ = window.hide();
        } else {
            quit(window.app_handle())
        }
    }
}
fn cleanup(app: &AppHandle) {
    // The same gate protects interactive writes and background retention.
    let Ok(_operation) = operation() else { return };
    if let Ok(_g) = PREF_LOCK.lock() {
        if let Ok(p) = read() {
            match trash::expire(p.retention_days) {
                Ok(ids) if !ids.is_empty() => {
                    let _ = app.emit("ponkan-trash-changed", ());
                }
                Err(e) => eprintln!("Trash cleanup: {e}"),
                _ => {}
            }
        }
    }
}
pub fn setup(app: &mut tauri::App) -> R<()> {
    let startup_operation = operation()?;
    let s = read().unwrap_or_default();
    let l = labels(&s.language);
    let open = MenuItem::with_id(app, "ponkan-open", l[0], true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings = MenuItem::with_id(app, "ponkan-settings", l[1], true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app, "ponkan-quit", l[2], true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let menu = Menu::with_items(app, &[&open, &settings, &quit_item]).map_err(|e| e.to_string())?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("Ícone do aplicativo ausente.")?;
    TrayIconBuilder::with_id("ponkan-tray")
        .icon(icon)
        .tooltip("pon//kan.")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "ponkan-open" => show(app),
            "ponkan-settings" => {
                show(app);
                let _ = app.emit("ponkan-open-settings", ());
            }
            "ponkan-quit" => quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show(tray.app_handle())
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;
    app.manage(TrayMenu {
        open,
        settings,
        quit: quit_item,
    });
    let args: Vec<_> = std::env::args().skip(1).collect();
    let startup = args.iter().any(|x| x == "--autostart");
    if !startup || !s.start_minimized || args.iter().any(|x| x.starts_with("ponkan")) {
        show(app.handle())
    }
    // Fail closed if a move recovery cannot be completed.
    let recovery = trash::recover();
    drop(startup_operation);
    match recovery {
        Ok(()) => {
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                cleanup(&handle);
                std::thread::sleep(Duration::from_secs(3600));
            });
        }
        Err(e) => eprintln!("Trash recovery: {e}"),
    }
    Ok(())
}

