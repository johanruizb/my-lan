//! Gestión del System Tray (Bandeja del Sistema) de MyLAN en Tauri 2.
//!
//! Permite que MyLAN se ejecute 24/7 en segundo plano minimizado a la bandeja,
//! mostrando estado en vivo ("Monitoreando: <red>"), permitiendo restaurar la
//! ventana, disparar un escaneo inmediato, pausar/reanudar y salir limpiamente.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

/// Identificadores de los items del menú contextual del System Tray.
pub const TRAY_ID_STATUS: &str = "tray_status";
pub const TRAY_ID_OPEN: &str = "tray_open";
pub const TRAY_ID_SCAN_NOW: &str = "tray_scan_now";
pub const TRAY_ID_TOGGLE_PAUSE: &str = "tray_toggle_pause";
pub const TRAY_ID_QUIT: &str = "tray_quit";

/// Estructura que conserva las referencias a los ítems dinámicos del menú del tray
/// para permitir actualizaciones de texto en vivo sin reconstruir el menú completo.
#[derive(Clone)]
pub struct TrayHandles {
    pub status_item: MenuItem<tauri::Wry>,
    pub pause_item: MenuItem<tauri::Wry>,
}

/// Construye e inicializa el System Tray en la aplicación.
pub fn setup_tray(app: &AppHandle) -> Result<(TrayIcon, TrayHandles), Box<dyn std::error::Error>> {
    // 1. Ítems del menú contextual
    let status_item = MenuItem::with_id(
        app,
        TRAY_ID_STATUS,
        "Monitoreo inicializando…",
        false, // Deshabilitado: solo informativo
        None::<&str>,
    )?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let open_item = MenuItem::with_id(app, TRAY_ID_OPEN, "Abrir MyLAN", true, None::<&str>)?;
    let scan_now_item =
        MenuItem::with_id(app, TRAY_ID_SCAN_NOW, "Escanear ahora", true, None::<&str>)?;
    let pause_item = MenuItem::with_id(
        app,
        TRAY_ID_TOGGLE_PAUSE,
        "Pausar monitoreo",
        true,
        None::<&str>,
    )?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, TRAY_ID_QUIT, "Salir de MyLAN", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &status_item,
            &sep1,
            &open_item,
            &scan_now_item,
            &pause_item,
            &sep2,
            &quit_item,
        ],
    )?;

    let handles = TrayHandles {
        status_item: status_item.clone(),
        pause_item: pause_item.clone(),
    };

    // 2. Icono del System Tray (usar el icono de la ventana o icon.png empaquetado)
    let icon = match app.default_window_icon() {
        Some(i) => i.clone(),
        None => {
            let bytes = include_bytes!("../icons/icon.png");
            tauri::image::Image::from_bytes(bytes)?
        }
    };

    // 3. TrayIconBuilder
    let tray = TrayIconBuilder::with_id("mylan_system_tray")
        .menu(&menu)
        .icon(icon)
        .tooltip("MyLAN — Monitoreo continuo de red")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app_handle, event| {
            match event.id.as_ref() {
                TRAY_ID_OPEN => {
                    open_main_window(app_handle);
                }
                TRAY_ID_SCAN_NOW => {
                    let app_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_clone.state::<crate::state::DesktopState>();
                        state.monitor.trigger_scan(app_clone.clone()).await;
                    });
                }
                TRAY_ID_TOGGLE_PAUSE => {
                    let app_clone = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_clone.state::<crate::state::DesktopState>();
                        let is_paused = state.monitor.toggle_pause();
                        state.monitor.update_tray(&app_clone, is_paused);
                    });
                }
                TRAY_ID_QUIT => {
                    // Cierre completo y limpio de la aplicación
                    app_handle.exit(0);
                }
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
                open_main_window(app);
            }
        })
        .build(app)?;

    Ok((tray, handles))
}

/// Restaura, desminimiza y enfoca la ventana principal `"main"`.
pub fn open_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
