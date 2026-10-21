mod config;
mod converters;
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
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            // The equivalent of the Nautilus scripts-accels. Wayland will simply refuse to
            // register; the app keeps working from the file-manager entry either way.
            if let Err(e) = app.global_shortcut().register("Alt+M") {
                eprintln!("citrus: Alt+M not available: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lang::get_language,
            config::get_prefs,
            config::set_language,
            config::set_accent,
            config::get_config,
            config::set_formats,
            config::reset_formats,
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
            wheel::cancel,
            wheel::quit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
