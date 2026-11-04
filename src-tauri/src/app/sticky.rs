use std::{sync::Mutex, thread, time::Duration};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use crate::app::state::AppState;

#[repr(C)]
struct StickyKeys { size: u32, flags: u32 }

#[link(name = "user32")]
extern "system" {
    fn SystemParametersInfoW(action: u32, param: u32, data: *mut StickyKeys, flags: u32) -> i32;
    fn ShowWindow(hwnd: isize, command: i32) -> i32;
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Indicator { key: &'static str, state: &'static str }

fn indicators(ok: bool, flags: u32) -> Vec<Indicator> {
    let mut active = Vec::new();
    if ok && flags & 1 != 0 {
        for (key, latched, locked) in [
            ("Shift", 0x03000000, 0x00030000),
            ("Ctrl",  0x0c000000, 0x000c0000),
            ("Alt",   0x30000000, 0x00300000),
            ("Win",   0xc0000000, 0x00c00000),
        ] {
            if flags & locked != 0 {
                active.push(Indicator { key, state: "locked" });
            } else if flags & latched != 0 {
                active.push(Indicator { key, state: "latched" });
            }
        }
    }
    active
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
      let mut last = Vec::new();
      let mut ticks = 20;
      let mut shown = false;
      loop {
        let mut info = StickyKeys { size: std::mem::size_of::<StickyKeys>() as u32, flags: 0 };
        let ok = unsafe { SystemParametersInfoW(0x003A, info.size, &mut info, 0) } != 0;
        let active = indicators(ok, info.flags);
        if active != last || ticks >= 20 {
            let _ = app.emit_to("main", "sticky-state", active.clone());
            last = active.clone();
            ticks = 0;
        }
        let listening = app.state::<Mutex<AppState>>().lock().map(|s| s.listening).unwrap_or(false);
        let should_show = listening && !active.is_empty();
        if should_show != shown {
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    // Show without taking focus; zero hides the window.
                    unsafe { ShowWindow(hwnd.0 as isize, if should_show { 4 } else { 0 }); }
                    shown = should_show;
                }
            }
        }
        ticks += 1;
        thread::sleep(Duration::from_millis(50));
      }
    });
}

#[cfg(test)]
mod tests {
    use super::{indicators, Indicator};

    #[test]
    fn failed_read_or_disabled_sticky_keys_has_no_indicators() {
        assert!(indicators(false, 0xffff_ffff).is_empty());
        assert!(indicators(true, 0xffff_fffe).is_empty());
        assert!(indicators(true, 1).is_empty());
    }

    #[test]
    fn left_and_right_sides_of_every_modifier_report_latched_and_locked() {
        let cases = [
            ("Shift", 0x0100_0000, 0x0200_0000, 0x0001_0000, 0x0002_0000),
            ("Ctrl",  0x0400_0000, 0x0800_0000, 0x0004_0000, 0x0008_0000),
            ("Alt",   0x1000_0000, 0x2000_0000, 0x0010_0000, 0x0020_0000),
            ("Win",   0x4000_0000, 0x8000_0000, 0x0040_0000, 0x0080_0000),
        ];
        for (key, left_latched, right_latched, left_locked, right_locked) in cases {
            for bit in [left_latched, right_latched] {
                assert_eq!(indicators(true, 1 | bit), vec![Indicator { key, state: "latched" }]);
            }
            for bit in [left_locked, right_locked] {
                assert_eq!(indicators(true, 1 | bit), vec![Indicator { key, state: "locked" }]);
            }
        }
    }

    #[test]
    fn combined_modifiers_stay_distinct_and_locked_takes_precedence() {
        let flags = 1 | 0x0100_0000 | 0x0200_0000 | 0x0400_0000 | 0x0001_0000 | 0x0080_0000;
        assert_eq!(indicators(true, flags), vec![
            Indicator { key: "Shift", state: "locked" },
            Indicator { key: "Ctrl", state: "latched" },
            Indicator { key: "Win", state: "locked" },
        ]);
    }
}
