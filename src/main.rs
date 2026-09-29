#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let message = format!("Stride could not continue.\n\n{info}\n\nPlease update your graphics driver. Details were saved to stride-error.log.");
        let directory = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf));
        if let Some(dir) = directory {
            let _ = std::fs::write(dir.join("stride-error.log"), &message);
        }
        eprintln!("{message}");
        #[cfg(target_os = "windows")]
        {
            #[link(name = "user32")]
            unsafe extern "system" {
                fn MessageBoxW(
                    hwnd: *mut std::ffi::c_void,
                    text: *const u16,
                    caption: *const u16,
                    kind: u32,
                ) -> i32;
            }
            let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
            let title: Vec<u16> = "Stride / startup error"
                .encode_utf16()
                .chain(Some(0))
                .collect();
            // SAFETY: both strings are null-terminated and alive for the call;
            // null HWND is permitted for an unowned error dialog.
            unsafe {
                MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
            }
        }
    }));
    miniquad::start(
        miniquad::conf::Conf {
            window_title: "STRIDE / Rust3D Movement Lab".into(),
            window_width: 1280,
            window_height: 800,
            high_dpi: true,
            sample_count: 4,
            ..Default::default()
        },
        || Box::new(app::Game::new()),
    );
}
