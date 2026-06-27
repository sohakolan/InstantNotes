use std::str::FromStr;
use std::sync::Mutex;

use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::panel::PanelState;

/// Raccourcis enregistrés : `toggle` ouvre/ferme, `newnote` (= toggle + Maj)
/// ouvre et crée une note instantanément.
#[derive(Default)]
pub struct ShortcutStore(pub Mutex<ShortcutCfg>);

#[derive(Default)]
pub struct ShortcutCfg {
    pub toggle: Option<Shortcut>,
    pub newnote: Option<Shortcut>,
}

/// (Ré)enregistre les raccourcis globaux et mémorise la largeur du panneau.
#[tauri::command]
pub fn set_shortcut(
    app: AppHandle,
    toggle: String,
    newnote: String,
    width_pct: f64,
    panel: State<PanelState>,
    shortcuts: State<ShortcutStore>,
) -> Result<(), String> {
    *panel.0.lock().unwrap() = width_pct;

    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let toggle_sc = Shortcut::from_str(&toggle).map_err(|e| format!("Raccourci invalide : {e}"))?;
    gs.register(toggle_sc.clone())
        .map_err(|e| format!("Raccourci déjà pris : {e}"))?;

    // La variante +Maj est optionnelle (ignorée si elle échoue ou est vide).
    let newnote_sc = if newnote.is_empty() {
        None
    } else if let Ok(sc) = Shortcut::from_str(&newnote) {
        let _ = gs.register(sc.clone());
        Some(sc)
    } else {
        None
    };

    let mut cfg = shortcuts.0.lock().unwrap();
    cfg.toggle = Some(toggle_sc);
    cfg.newnote = newnote_sc;
    Ok(())
}
