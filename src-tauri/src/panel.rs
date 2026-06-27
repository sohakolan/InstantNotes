use std::sync::Mutex;

use tauri::{Emitter, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

/// Largeur du panneau en pourcentage de la largeur de l'écran.
pub struct PanelState(pub Mutex<f64>);

impl Default for PanelState {
    fn default() -> Self {
        PanelState(Mutex::new(20.0))
    }
}

/// Insets de la zone utile (barre de menus en haut, Dock à droite), en pixels
/// physiques : `(top, usable_height, right)`. Sur macOS on lit la `visibleFrame`
/// de l'écran pour que le panneau s'aligne sous la barre de menus.
#[cfg(target_os = "macos")]
fn visible_insets(scale: f64, monitor_h_px: u32) -> (i32, u32, i32) {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    // AppKit ne doit être touché que sur le thread principal.
    let inner = || -> Option<(i32, u32, i32)> {
        let _mtm = MainThreadMarker::new()?;
        let screen = NSScreen::mainScreen(_mtm)?;
        let frame = screen.frame();
        let vis = screen.visibleFrame();
        let top_pt = (frame.origin.y + frame.size.height) - (vis.origin.y + vis.size.height);
        let right_pt = (frame.origin.x + frame.size.width) - (vis.origin.x + vis.size.width);
        let usable_pt = vis.size.height;
        Some((
            (top_pt.max(0.0) * scale).round() as i32,
            (usable_pt * scale).round() as u32,
            (right_pt.max(0.0) * scale).round() as i32,
        ))
    };
    inner().unwrap_or((0, monitor_h_px, 0))
}

#[cfg(not(target_os = "macos"))]
fn visible_insets(_scale: f64, monitor_h_px: u32) -> (i32, u32, i32) {
    (0, monitor_h_px, 0)
}

/// Positionne la fenêtre sur le bord droit du moniteur courant, sur la hauteur
/// utile (sous la barre de menus, au-dessus du Dock), largeur `width_pct` %.
pub fn position_panel<R: Runtime>(win: &WebviewWindow<R>, width_pct: f64) -> tauri::Result<()> {
    let monitor = match win.current_monitor()? {
        Some(m) => m,
        None => match win.primary_monitor()? {
            Some(m) => m,
            None => return Ok(()),
        },
    };
    let size = monitor.size(); // pixels physiques
    let origin = monitor.position();
    let scale = monitor.scale_factor();

    let pct = width_pct.clamp(10.0, 60.0) / 100.0;
    let w = (size.width as f64 * pct).round() as u32;

    let (top, usable_h, right) = visible_insets(scale, size.height);

    win.set_size(PhysicalSize::new(w, usable_h))?;
    let x = origin.x + (size.width as i32 - w as i32) - right;
    let y = origin.y + top;
    win.set_position(PhysicalPosition::new(x, y))?;
    Ok(())
}

pub fn show_panel<R: Runtime>(win: &WebviewWindow<R>, width_pct: f64) -> tauri::Result<()> {
    position_panel(win, width_pct)?;
    win.show()?;
    win.set_focus()?;
    // Déclenche l'animation de glissement côté frontend.
    let _ = win.emit("panel-shown", ());
    Ok(())
}

pub fn toggle_panel<R: Runtime>(win: &WebviewWindow<R>, width_pct: f64) -> tauri::Result<()> {
    if win.is_visible().unwrap_or(false) {
        win.hide()?;
    } else {
        show_panel(win, width_pct)?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_panel(win: WebviewWindow) -> Result<(), String> {
    win.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_panel_cmd(
    win: WebviewWindow,
    state: tauri::State<PanelState>,
) -> Result<(), String> {
    let pct = *state.0.lock().unwrap();
    show_panel(&win, pct).map_err(|e| e.to_string())
}

/// Ouvre (ou ramène au premier plan) la fenêtre « Organisation » (mode worktree).
#[tauri::command]
pub fn open_manager(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
    if let Some(w) = app.get_webview_window("manager") {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, "manager", WebviewUrl::App("index.html".into()))
        .title("InstantNotes — Organisation")
        .inner_size(960.0, 640.0)
        .min_inner_size(680.0, 440.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}
