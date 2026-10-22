mod config;
mod converters;
mod installer;
mod lang;
mod spawn;
mod thumbnail;
mod tools;
mod wheel;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        // resize_and_move blocks ~200 ms on purpose; keep the event loop free
                        let app = app.clone();
                        std::thread::spawn(move || wheel::toggle_at_cursor(&app));
                    }
                })
                .build(),
        )
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            // The equivalent of the Nautilus scripts-accels. Wayland will simply refuse to
            // register; the app keeps working from the file-manager entry either way.
            if let Err(e) = app.global_shortcut().register("Alt+M") {
                eprintln!("citrus: Alt+M not available: {e}");
            }
            // A tray icon so the always-borderless, taskbar-skipping app is always
            // reachable: Open summons the card at the cursor, Quit ends the process.
            let handle = app.handle();
            let open = MenuItem::with_id(handle, "open", "Open Citrus", true, None::<&str>)?;
            let quit = MenuItem::with_id(handle, "quit", "Quit Citrus", true, None::<&str>)?;
            let menu = Menu::with_items(handle, &[&open, &quit])?;
            let mut builder = TrayIconBuilder::with_id("citrus-tray")
                .menu(&menu)
                .tooltip("Citrus")
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        let app = app.clone();
                        std::thread::spawn(move || wheel::show_at_cursor(&app));
                    }
                    "quit" => app.exit(0),
                    _ => {}
                });
            if let Some(icon) = handle.default_window_icon() {
                builder = builder.icon(icon.clone());
            }
            builder.build(handle)?;
            Ok(())
        })
        // Closing the window (Alt+F4, etc.) hides it to the tray rather than quitting, so
        // there is always a way back; the tray Quit (or Esc on the drop card) truly exits.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            lang::get_language,
            config::get_prefs,
            config::set_language,
            config::set_accent,
            config::get_config,
            config::set_formats,
            config::reset_formats,
            config::mark_setup_seen,
            converters::get_targets,
            converters::convert,
            converters::get_targets_many,
            converters::take_warnings,
            converters::convert_many,
            thumbnail::thumbnail,
            wheel::initial_paths,
            wheel::file_info,
            wheel::show_wheel,
            wheel::cursor_page_pos,
            wheel::place_wheel,
            wheel::center_wheel,
            wheel::resize_center,
            tools::get_tools,
            tools::run_tool,
            tools::media_info,
            tools::preview_image,
            tools::video_frame,
            tools::waveform,
            installer::get_tool_status,
            installer::install_tools,
            installer::cancel_install,
            installer::minimize_window,
            installer::set_install_taskbar,
            wheel::cancel,
            wheel::quit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
