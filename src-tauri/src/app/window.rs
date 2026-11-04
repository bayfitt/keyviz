pub fn config_window(window: &tauri::WebviewWindow) {
    window
        .set_ignore_cursor_events(true)
        .expect("Failed to set ignore cursor events");

    #[cfg(target_os = "windows")]
    {
        let hwnd = window.hwnd().unwrap().0 as isize;
        #[link(name = "dwmapi")]
        extern "system" { fn DwmSetWindowAttribute(hwnd: isize, attribute: u32, value: *const u32, size: u32) -> i32; }
        // Windows 11 can draw a frame around an otherwise transparent window.
        let no_border = 0xffff_fffeu32;
        unsafe { let _ = DwmSetWindowAttribute(hwnd, 34, &no_border, 4); }
        place_indicator(hwnd);
        // The work area and DPI can change while the utility is running.
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                place_indicator(hwnd);
            }
        });
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let position = monitor.position();
            let size = monitor.size();

            window
                .set_position(tauri::PhysicalPosition {
                    x: position.x,
                    y: position.y,
                })
                .unwrap();
            window
                .set_size(tauri::PhysicalSize {
                    width: size.width,
                    height: size.height,
                })
                .unwrap();
        }

        use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
        use cocoa::base::id;

        unsafe {
            let ns_window = window.ns_window().unwrap() as id;
            ns_window.setLevel_(1000);

            ns_window.setCollectionBehavior_(
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces,
            );
        }
    }

    #[cfg(not(target_os = "windows"))]
    window.show().expect("Failed to show window");
}

#[cfg(target_os = "windows")]
fn place_indicator(hwnd: isize) {
    #[repr(C)]
    #[derive(Default)]
    struct Point { x: i32, y: i32 }
    #[repr(C)]
    #[derive(Default)]
    struct Rect { left: i32, top: i32, right: i32, bottom: i32 }
    #[repr(C)]
    #[derive(Default)]
    struct MonitorInfo { size: u32, monitor: Rect, work: Rect, flags: u32 }

    #[link(name = "user32")]
    extern "system" {
        fn MonitorFromPoint(point: Point, flags: u32) -> isize;
        fn GetMonitorInfoW(monitor: isize, info: *mut MonitorInfo) -> i32;
        fn GetDpiForWindow(hwnd: isize) -> u32;
        fn GetWindowRect(hwnd: isize, rect: *mut Rect) -> i32;
        fn SetWindowPos(hwnd: isize, after: isize, x: i32, y: i32, width: i32, height: i32, flags: u32) -> i32;
    }

    unsafe {
        // The primary monitor contains the desktop origin, (0, 0).
        let monitor = MonitorFromPoint(Point::default(), 1);
        let mut info = MonitorInfo { size: std::mem::size_of::<MonitorInfo>() as u32, ..Default::default() };
        if monitor == 0 || GetMonitorInfoW(monitor, &mut info) == 0 { return; }
        let dpi = GetDpiForWindow(hwnd).max(96) as i32;
        let scale = |value: i32| (value * dpi + 48) / 96;
        let width = scale(256);
        let height = scale(78);
        let x = info.work.right - scale(16) - width;
        let y = info.work.bottom - scale(16) - height;
        let mut current = Rect::default();
        if GetWindowRect(hwnd, &mut current) == 0 ||
            (current.left, current.top, current.right, current.bottom) != (x, y, x + width, y + height) {
            // Keep the overlay above normal windows without activating it.
            let _ = SetWindowPos(hwnd, -1, x, y, width, height, 0x0010 | 0x0200);
        }
    }
}
