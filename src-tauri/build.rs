fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "list_profiles",
            "save_profile",
            "apply_profile",
            "matching_profiles",
            "rename_profile",
            "delete_profile",
            "list_trash",
            "restore_trash",
            "purge_trash",
            "generate_share_code",
            "clipboard_read_text",
            "clipboard_write_text",
            "import_share_code",
            "has_pending_share",
            "import_next_share",
            "export_pkn",
            "select_pkn",
            "import_pkn",
            "cancel_pkn",
            "desktop_preferences",
            "update_desktop_preference",
        ]),
    ))
    .expect("Failed to build Ponkan command permissions");
}
