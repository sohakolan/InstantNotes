mod git;
mod panel;
mod shortcut;
mod vault;

use panel::PanelState;
use shortcut::ShortcutStore;
use vault::VaultState;

#[cfg(desktop)]
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    use tauri::{Emitter, Manager};

    let open_i = MenuItem::with_id(app, "open", "Ouvrir InstantNotes", true, None::<&str>)?;
    let manager_i = MenuItem::with_id(app, "manager", "Organiser les notes…", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Réglages…", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_i, &manager_i, &settings_i, &quit_i])?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let pct = *app.state::<PanelState>().0.lock().unwrap();
            match event.id.as_ref() {
                "open" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = panel::show_panel(&w, pct);
                    }
                }
                "manager" => {
                    let _ = panel::open_manager(app.clone());
                }
                "settings" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = panel::show_panel(&w, pct);
                        let _ = w.emit("open-settings", ());
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let pct = *app.state::<PanelState>().0.lock().unwrap();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = panel::toggle_panel(&w, pct);
                }
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(VaultState::default())
        .manage(PanelState::default())
        .manage(ShortcutStore::default());

    #[cfg(desktop)]
    {
        use tauri::{Emitter, Manager};
        use tauri_plugin_global_shortcut::ShortcutState as GsState;

        builder = builder
            .plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(|app, shortcut, event| {
                        if event.state() != GsState::Pressed {
                            return;
                        }
                        let pct = *app.state::<PanelState>().0.lock().unwrap();
                        let store = app.state::<ShortcutStore>();
                        let is_newnote = {
                            let cfg = store.0.lock().unwrap();
                            cfg.newnote.as_ref() == Some(shortcut)
                        };
                        if let Some(win) = app.get_webview_window("main") {
                            if is_newnote {
                                let _ = panel::show_panel(&win, pct);
                                let _ = win.emit("new-note", ());
                            } else {
                                let _ = panel::toggle_panel(&win, pct);
                            }
                        }
                    })
                    .build(),
            )
            .setup(|app| {
                #[cfg(target_os = "macos")]
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.set_visible_on_all_workspaces(true);
                }
                setup_tray(app)?;
                Ok(())
            });
    }

    builder
        .invoke_handler(tauri::generate_handler![
            vault::set_vault,
            vault::read_tree,
            vault::read_note,
            vault::write_note,
            vault::create_note,
            vault::create_folder,
            vault::move_path,
            vault::delete_path,
            panel::hide_panel,
            panel::show_panel_cmd,
            panel::open_manager,
            shortcut::set_shortcut,
            git::git_init,
            git::git_commit,
            git::git_set_remote,
            git::git_get_remote,
            git::git_push,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
