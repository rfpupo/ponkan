mod apply_flow;
mod apply_recovery;
mod desktop;
mod migration;
mod portable;
mod portable_local;
mod profiles;
mod riot;
mod share;
mod storage_lock;
mod trash;
use tauri_plugin_clipboard_manager::ClipboardExt;
#[cfg(all(windows, debug_assertions))]
use tauri_plugin_deep_link::DeepLinkExt;

use profiles::Profile;
use tauri::Manager;

#[tauri::command]
fn list_profiles() -> Result<Vec<Profile>, String> {
    profiles::list()
}

#[tauri::command]
async fn save_profile(name: String, id: Option<String>) -> Result<Vec<Profile>, String> {
    let _operation = desktop::operation()?;
    let _game_lock = storage_lock::acquire_game()?;
    apply_recovery::reconcile_pending().await?;
    let session = riot::session().await?;
    let cloud = riot::read_for(&session).await?;
    riot::assert_current_session(&session).await?;
    profiles::save(name, id, cloud, &session).await
}

struct NativeApply<'a> {
    plan: portable_local::Plan,
    session: &'a riot::Session,
}
impl apply_flow::Backend for NativeApply<'_> {
    fn emergency_backup(&self, before: &str) -> Result<(), String> {
        profiles::emergency_backup(before).map(|_| ())
    }
    fn game_closed(&self) -> Result<(), String> {
        profiles::require_game_closed()
    }
    async fn assert_session(&self) -> Result<(), String> {
        riot::assert_current_session(self.session).await
    }
    async fn read_cloud(&self) -> Result<String, String> {
        riot::read_for(self.session).await
    }
    fn cloud_equals(&self, left: &str, right: &str) -> bool {
        riot::cloud_equals(left, right)
    }
    fn verify_before(&self) -> Result<(), String> {
        self.plan.verify_before()
    }
    fn begin(&self, before: &str, intended: &str) -> Result<(), String> {
        let (local_before, local_after) = self.plan.maps();
        apply_recovery::begin(self.session, before, intended, local_before, local_after)
    }
    async fn write_cloud(&self, data: &str) -> Result<(), String> {
        riot::write_for(self.session, data).await
    }
    async fn rollback_cloud(&self, before: &str, intended: &str) -> Result<(), String> {
        riot::rollback_for(self.session, before, intended).await
    }
    fn commit(&self) -> Result<(), String> {
        self.plan.commit()
    }
    fn verify_after(&self) -> Result<(), String> {
        self.plan.verify_after()
    }
    async fn reconcile(&self) -> Result<(), String> {
        apply_recovery::reconcile(self.session, false).await
    }
    fn finish(&self) {
        self.plan.finish();
    }
}
async fn apply_transaction(
    plan: portable_local::Plan,
    session: &riot::Session,
    before: &str,
    intended: &str,
    auto_backup: bool,
) -> Result<(), String> {
    apply_flow::run(
        &NativeApply { plan, session },
        before,
        intended,
        auto_backup,
    )
    .await
}

#[tauri::command]
async fn apply_profile(id: String, auto_backup: Option<bool>) -> Result<(), String> {
    let _operation = desktop::operation()?;
    let _game_lock = storage_lock::acquire_game()?;
    profiles::require_game_closed()?;
    apply_recovery::reconcile_pending().await?;
    let session = riot::session().await?;
    let previous = riot::read_for(&session).await?;
    let subject = session
        .subject
        .as_deref()
        .ok_or("Identidade Riot indisponível")?;
    let target = profiles::config_root()?;
    let (plan, selected) = if profiles::find(&id)?.shared == Some(true) {
        let share = share::load(&id)?;
        let cloud = portable::share_cloud(&share.settings)?;
        let merged = if share.settings.is_empty() {
            previous.clone()
        } else {
            portable::merge_cloud(&portable::encode_share_cloud(&cloud)?, &previous)?
        };
        (
            portable_local::Plan::prepare_shared(&target, subject, &cloud, &share.video)?,
            merged,
        )
    } else {
        let selected = profiles::cloud_for(&id)?;
        let source = profiles::config_for(&id)?;
        if portable::is_imported(&id)? {
            (
                portable_local::Plan::prepare(&source, &target, subject)?,
                portable::merge_cloud(&selected, &previous)?,
            )
        } else {
            (
                portable_local::Plan::prepare_full(&source, &target)?,
                selected,
            )
        }
    };
    apply_transaction(
        plan,
        &session,
        &previous,
        &selected,
        auto_backup.unwrap_or(true),
    )
    .await
}

#[tauri::command]
async fn matching_profiles() -> Result<Vec<String>, String> {
    profiles::require_game_closed()?;
    let cloud = riot::read_preferences().await?;
    let local = profiles::config_root()?;
    let mut matches = Vec::new();
    for p in profiles::list()? {
        if p.portable == Some(true) {
            continue;
        }
        if let (Ok(saved), Ok(config)) = (profiles::cloud_for(&p.id), profiles::config_for(&p.id)) {
            if riot::cloud_equals(&cloud, &saved) && profiles::verify_tree(&config, &local).is_ok()
            {
                matches.push(p.id);
            }
        }
    }
    profiles::require_game_closed()?;
    Ok(matches)
}

#[tauri::command]
fn rename_profile(id: String, name: String) -> Result<Vec<Profile>, String> {
    let _operation = desktop::operation()?;
    profiles::rename(&id, name)
}

#[tauri::command]
fn delete_profile(
    id: String,
    avatar: Option<String>,
    favorite: Option<bool>,
) -> Result<trash::Entry, String> {
    let _operation = desktop::operation()?;
    trash::delete(&id, avatar, favorite.unwrap_or(false))
}
#[tauri::command]
fn list_trash() -> Result<Vec<trash::Entry>, String> {
    trash::list()
}
#[tauri::command]
fn restore_trash(id: String) -> Result<trash::Entry, String> {
    let _operation = desktop::operation()?;
    trash::restore(&id)
}
#[tauri::command]
fn purge_trash(id: String) -> Result<(), String> {
    let _operation = desktop::operation()?;
    trash::purge(&id)
}

#[tauri::command]
async fn generate_share_code(id: String) -> Result<String, String> {
    let _operation = desktop::operation()?;
    share::short::generate(&id).await
}
#[tauri::command]
fn clipboard_read_text(app: tauri::AppHandle) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|_| "Não foi possível ler a área de transferência.".into())
}
#[tauri::command]
fn clipboard_write_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    if text.len() > 1024 * 1024 {
        return Err("Texto grande demais para copiar.".into());
    }
    app.clipboard()
        .write_text(text)
        .map_err(|_| "Não foi possível copiar o texto.".into())
}
#[tauri::command]
async fn import_share_code(input: String) -> Result<Profile, String> {
    let _operation = desktop::operation()?;
    share::short::import(&input).await
}
#[tauri::command]
fn has_pending_share(queue: tauri::State<'_, share::Queue>) -> Result<bool, String> {
    queue.has_pending()
}
#[tauri::command]
fn import_next_share(queue: tauri::State<'_, share::Queue>) -> Result<Option<Profile>, String> {
    let _operation = desktop::operation()?;
    queue.pop()?.map(|uri| share::import_uri(&uri)).transpose()
}

#[tauri::command]
async fn export_pkn(
    id: String,
    avatar: Option<String>,
    author: Option<String>,
    description: Option<String>,
) -> Result<bool, String> {
    let _operation = desktop::operation()?;
    tauri::async_runtime::spawn_blocking(move || {
        let profile = profiles::find(&id)?;
        let name = profile.name;
        let bytes = portable::export(&id, avatar, author, description)?;
        portable::choose_export(&name, &bytes)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn select_pkn(
    state: tauri::State<'_, PendingImport>,
) -> Result<Option<portable::Preview>, String> {
    let _operation = desktop::operation()?;
    let chosen = tauri::async_runtime::spawn_blocking(portable::choose_import)
        .await
        .map_err(|e| e.to_string())??;
    let mut pending = state.0.lock().map_err(|_| "Importação indisponível")?;
    *pending = None;
    let Some(bytes) = chosen else { return Ok(None) };
    let preview = portable::preview(&bytes)?;
    *pending = Some(bytes);
    Ok(Some(preview))
}
#[tauri::command]
fn import_pkn(name: String, state: tauri::State<'_, PendingImport>) -> Result<Profile, String> {
    let _operation = desktop::operation()?;
    let mut pending = state.0.lock().map_err(|_| "Importação indisponível")?;
    let bytes = pending.as_ref().ok_or("Selecione um arquivo .pkn")?;
    let p = portable::import(bytes, name)?;
    *pending = None;
    Ok(p)
}
#[tauri::command]
fn cancel_pkn(state: tauri::State<'_, PendingImport>) -> Result<(), String> {
    *state.0.lock().map_err(|_| "Importação indisponível")? = None;
    Ok(())
}
#[tauri::command]
fn desktop_preferences(app: tauri::AppHandle) -> Result<desktop::DesktopState, String> {
    desktop::get(&app)
}
#[tauri::command]
fn update_desktop_preference(
    app: tauri::AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<desktop::DesktopState, String> {
    desktop::update(&app, &key, value)
}
#[derive(Default)]
struct PendingImport(std::sync::Mutex<Option<Vec<u8>>>);
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(not(debug_assertions))]
    if let Err(error) = migration::ensure_release_ready() {
        rfd::MessageDialog::new()
            .set_title("Ponkan")
            .set_description(error)
            .set_level(rfd::MessageLevel::Error)
            .show();
        return;
    }
    let queue = share::Queue::default();
    queue.push_args(std::env::args().skip(1));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let startup = args.iter().any(|a| a == "--autostart");
            if app
                .state::<share::Queue>()
                .push_args(args.into_iter().skip(1))
                || !startup
            {
                desktop::show(app);
            }
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name(if cfg!(debug_assertions) {
                    "ponkan-dev"
                } else {
                    "ponkan"
                })
                .args(["--autostart"])
                .build(),
        )
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(queue)
        .manage(PendingImport::default())
        .on_window_event(desktop::close)
        .setup(move |app| {
            #[cfg(all(windows, debug_assertions))]
            app.deep_link().register(share::scheme())?;

            if let (Some(window), Some(icon)) =
                (app.get_webview_window("main"), app.default_window_icon())
            {
                window.set_icon(icon.clone())?;
            }
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    // Windows 11: request the native, small rounded corners.
                    #[link(name = "dwmapi")]
                    extern "system" {
                        fn DwmSetWindowAttribute(
                            hwnd: *mut std::ffi::c_void,
                            attribute: u32,
                            value: *const std::ffi::c_void,
                            size: u32,
                        ) -> i32;
                    }
                    let preference: u32 = 3;
                    unsafe {
                        let _ = DwmSetWindowAttribute(
                            hwnd.0 as _,
                            33,
                            (&preference as *const u32).cast(),
                            4,
                        );
                    }
                }
            }
            desktop::setup(app).map_err(std::io::Error::other)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            generate_share_code,
            clipboard_read_text,
            clipboard_write_text,
            import_share_code,
            has_pending_share,
            import_next_share,
            matching_profiles,
            save_profile,
            apply_profile,
            rename_profile,
            delete_profile,
            list_trash,
            restore_trash,
            purge_trash,
            desktop_preferences,
            update_desktop_preference,
            export_pkn,
            select_pkn,
            import_pkn,
            cancel_pkn,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Ponkan");
}
