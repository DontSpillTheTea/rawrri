use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::env;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSurfaceSnapshot {
    pub front_wid: Option<u64>,
    pub rear_wid: Option<u64>,
    pub parent_hwnd_raw: Option<isize>,
    pub front_hwnd_raw: Option<isize>,
    pub rear_hwnd_raw: Option<isize>,
    pub front_visible: bool,
    pub rear_visible: bool,
    pub front_window_rect: Option<NativeRect>,
    pub rear_window_rect: Option<NativeRect>,
    pub front_client_rect: Option<NativeRect>,
    pub rear_client_rect: Option<NativeRect>,
    pub last_front_layout: Option<VideoRect>,
    pub last_rear_layout: Option<VideoRect>,
    pub debug_visual_hosts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Default)]
pub struct VideoSurfaceController {
    manager: Mutex<VideoSurfaceManager>,
}

impl VideoSurfaceController {
    pub fn ensure_for_window(&self, window: &tauri::Window) -> Result<VideoSurfaceSnapshot, String> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| "Video surface lock poisoned".to_string())?;
        manager.ensure_for_window(window)
    }

    pub fn update_layout(
        &self,
        window: &tauri::Window,
        front: VideoRect,
        rear: VideoRect,
    ) -> Result<VideoSurfaceSnapshot, String> {
        let mut manager = self
            .manager
            .lock()
            .map_err(|_| "Video surface lock poisoned".to_string())?;
        manager.ensure_for_window(window)?;
        manager.update_layout(front, rear)?;
        Ok(manager.snapshot())
    }
}

#[cfg(target_os = "windows")]
struct VideoSurfaceManager {
    // Store HWNDs in Tauri-managed state as integer values (Send + Sync safe).
    parent_hwnd_raw: Option<isize>,
    front_hwnd_raw: Option<isize>,
    rear_hwnd_raw: Option<isize>,
    last_front_layout: Option<VideoRect>,
    last_rear_layout: Option<VideoRect>,
    has_promoted_front: bool,
    has_promoted_rear: bool,
    debug_visual_hosts: bool,
}

#[cfg(not(target_os = "windows"))]
#[derive(Default)]
struct VideoSurfaceManager;

#[cfg(target_os = "windows")]
impl VideoSurfaceManager {
    fn new() -> Self {
        let debug_visual_hosts = env::var("RAWRII_SURFACE_DEBUG")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self {
            parent_hwnd_raw: None,
            front_hwnd_raw: None,
            rear_hwnd_raw: None,
            last_front_layout: None,
            last_rear_layout: None,
            has_promoted_front: false,
            has_promoted_rear: false,
            debug_visual_hosts,
        }
    }

    fn ensure_for_window(&mut self, window: &tauri::Window) -> Result<VideoSurfaceSnapshot, String> {
        let parent_raw = window_hwnd_raw(window)?;
        if self.parent_hwnd_raw == Some(parent_raw) && self.front_hwnd_raw.is_some() && self.rear_hwnd_raw.is_some() {
            println!(
                "video_surface.ensure reuse parent_raw={} front_raw={} rear_raw={}",
                self.parent_hwnd_raw.unwrap_or(0),
                self.front_hwnd_raw.unwrap_or(0),
                self.rear_hwnd_raw.unwrap_or(0)
            );
            return Ok(self.snapshot());
        }

        self.destroy_existing();

        let front_raw = create_child_surface(parent_raw, self.debug_visual_hosts)?;
        let rear_raw = create_child_surface(parent_raw, self.debug_visual_hosts)?;
        self.parent_hwnd_raw = Some(parent_raw);
        self.front_hwnd_raw = Some(front_raw);
        self.rear_hwnd_raw = Some(rear_raw);
        println!(
            "video_surface.ensure create parent_raw={} front_raw={} rear_raw={}",
            parent_raw, front_raw, rear_raw
        );
        Ok(self.snapshot())
    }

    fn update_layout(&mut self, front: VideoRect, rear: VideoRect) -> Result<(), String> {
        let front_raw = self
            .front_hwnd_raw
            .ok_or_else(|| "Front video surface not initialized".to_string())?;
        let rear_raw = self
            .rear_hwnd_raw
            .ok_or_else(|| "Rear video surface not initialized".to_string())?;

        let front_changed = self.last_front_layout.as_ref() != Some(&front);
        let rear_changed = self.last_rear_layout.as_ref() != Some(&rear);

        if !front_changed && !rear_changed {
            println!(
                "video_surface.layout unchanged front_raw={} rear_raw={}",
                front_raw, rear_raw
            );
            return Ok(());
        }

        println!(
            "video_surface.layout front_raw={} rear_raw={} front=({}, {}, {}, {}) rear=({}, {}, {}, {}) debug_visual_hosts={}",
            front_raw,
            rear_raw,
            front.x,
            front.y,
            front.width,
            front.height,
            rear.x,
            rear.y,
            rear.width,
            rear.height,
            self.debug_visual_hosts
        );
        if front_changed {
            move_surface(front_raw, &front, !self.has_promoted_front)?;
            self.last_front_layout = Some(front);
            self.has_promoted_front = true;
        }
        if rear_changed {
            move_surface(rear_raw, &rear, !self.has_promoted_rear)?;
            self.last_rear_layout = Some(rear);
            self.has_promoted_rear = true;
        }
        Ok(())
    }

    fn snapshot(&self) -> VideoSurfaceSnapshot {
        VideoSurfaceSnapshot {
            // mpv expects --wid as an integer representation of the native handle.
            front_wid: self.front_hwnd_raw.map(|value| value as u64),
            rear_wid: self.rear_hwnd_raw.map(|value| value as u64),
            parent_hwnd_raw: self.parent_hwnd_raw,
            front_hwnd_raw: self.front_hwnd_raw,
            rear_hwnd_raw: self.rear_hwnd_raw,
            front_visible: self.front_hwnd_raw.map(is_window_visible).unwrap_or(false),
            rear_visible: self.rear_hwnd_raw.map(is_window_visible).unwrap_or(false),
            front_window_rect: self.front_hwnd_raw.and_then(window_rect),
            rear_window_rect: self.rear_hwnd_raw.and_then(window_rect),
            front_client_rect: self.front_hwnd_raw.and_then(client_rect),
            rear_client_rect: self.rear_hwnd_raw.and_then(client_rect),
            last_front_layout: self.last_front_layout.clone(),
            last_rear_layout: self.last_rear_layout.clone(),
            debug_visual_hosts: self.debug_visual_hosts,
        }
    }

    fn destroy_existing(&mut self) {
        if let Some(front_raw) = self.front_hwnd_raw.take() {
            destroy_surface(front_raw);
        }
        if let Some(rear_raw) = self.rear_hwnd_raw.take() {
            destroy_surface(rear_raw);
        }
        self.parent_hwnd_raw = None;
        self.last_front_layout = None;
        self.last_rear_layout = None;
        self.has_promoted_front = false;
        self.has_promoted_rear = false;
    }
}

#[cfg(target_os = "windows")]
impl Default for VideoSurfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "windows"))]
impl VideoSurfaceManager {
    fn ensure_for_window(&mut self, _window: &tauri::Window) -> Result<VideoSurfaceSnapshot, String> {
        Err("Embedded surfaces are currently supported on Windows only".to_string())
    }

    fn update_layout(&mut self, _front: VideoRect, _rear: VideoRect) -> Result<(), String> {
        Err("Embedded surfaces are currently supported on Windows only".to_string())
    }

    fn snapshot(&self) -> VideoSurfaceSnapshot {
        VideoSurfaceSnapshot {
            front_wid: None,
            rear_wid: None,
            parent_hwnd_raw: None,
            front_hwnd_raw: None,
            rear_hwnd_raw: None,
            front_visible: false,
            rear_visible: false,
            front_window_rect: None,
            rear_window_rect: None,
            front_client_rect: None,
            rear_client_rect: None,
            last_front_layout: None,
            last_rear_layout: None,
            debug_visual_hosts: false,
        }
    }
}

#[cfg(target_os = "windows")]
fn window_hwnd_raw(window: &tauri::Window) -> Result<isize, String> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window
        .window_handle()
        .map_err(|err| format!("Failed to get window handle: {}", err))?;
    let raw = handle.as_raw();
    match raw {
        // raw-window-handle exposes HWND as a non-zero integer; store as raw isize in managed state.
        RawWindowHandle::Win32(win32) => Ok(win32.hwnd.get()),
        _ => Err("Unsupported window handle type for embedding".to_string()),
    }
}

#[cfg(target_os = "windows")]
fn create_child_surface(parent_raw: isize, debug_visual_hosts: bool) -> Result<isize, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOW, WS_BORDER};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
    };

    let class_name = to_wide("STATIC");
    let window_name = to_wide("rawrii_video_surface");
    let mut style = WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS;
    if debug_visual_hosts {
        // Debug-only visual hint so host surfaces are easy to see before mpv renders.
        style |= WS_BORDER;
    }

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_name.as_ptr(),
            style,
            0,
            0,
            1,
            1,
            raw_to_hwnd(parent_raw),
            // windows-sys uses pointer-shaped handle types; use null_mut() for null handles.
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    // HWND is a pointer type in windows-sys, so null-check with is_null().
    if hwnd.is_null() {
        return Err("Failed to create child video surface HWND".to_string());
    }
    let show_result = unsafe { ShowWindow(hwnd, SW_SHOW) };
    let hwnd_raw = hwnd_to_raw(hwnd);
    println!(
        "video_surface.create hwnd_raw={} parent_raw={} style={} show_result={}",
        hwnd_raw, parent_raw, style, show_result
    );
    Ok(hwnd_raw)
}

#[cfg(target_os = "windows")]
fn move_surface(hwnd_raw: isize, rect: &VideoRect, promote_on_first_layout: bool) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MoveWindow, ShowWindow, SetWindowPos, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW,
    };

    let width = rect.width.max(1);
    let height = rect.height.max(1);
    let hwnd = raw_to_hwnd(hwnd_raw);
    let moved = unsafe { MoveWindow(hwnd, rect.x, rect.y, width, height, 1) };
    if moved == 0 {
        return Err(format!(
            "Failed to move embedded surface hwnd_raw={} x={} y={} w={} h={}",
            hwnd_raw, rect.x, rect.y, width, height
        ));
    }
    // Avoid z-order thrashing: promote once, then keep stable with NOZORDER updates.
    let (insert_after, flags) = if promote_on_first_layout {
        (
            windows_sys::Win32::UI::WindowsAndMessaging::HWND_TOP,
            SWP_SHOWWINDOW | SWP_NOACTIVATE,
        )
    } else {
        (raw_to_hwnd(0), SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOZORDER)
    };
    let resized = unsafe { SetWindowPos(hwnd, insert_after, rect.x, rect.y, width, height, flags) };
    if resized == 0 {
        return Err(format!(
            "Failed to resize embedded surface hwnd_raw={} x={} y={} w={} h={}",
            hwnd_raw, rect.x, rect.y, width, height
        ));
    }
    let show_result = unsafe { ShowWindow(hwnd, SW_SHOW) };
    println!(
        "video_surface.move hwnd_raw={} moved={} resized={} show_result={} promote={} final_window_rect={:?} final_client_rect={:?}",
        hwnd_raw,
        moved,
        resized,
        show_result,
        promote_on_first_layout,
        window_rect(hwnd_raw),
        client_rect(hwnd_raw)
    );
    Ok(())
}

#[cfg(target_os = "windows")]
fn destroy_surface(hwnd_raw: isize) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyWindow, IsWindow};
    println!("video_surface.destroy hwnd_raw={}", hwnd_raw);
    let hwnd = raw_to_hwnd(hwnd_raw);
    let exists = unsafe { IsWindow(hwnd) != 0 };
    println!("video_surface.destroy exists_before={}", exists);
    let _ = unsafe { DestroyWindow(hwnd) };
}

#[cfg(target_os = "windows")]
fn hwnd_to_raw(hwnd: windows_sys::Win32::Foundation::HWND) -> isize {
    hwnd as isize
}

#[cfg(target_os = "windows")]
fn raw_to_hwnd(raw: isize) -> windows_sys::Win32::Foundation::HWND {
    raw as windows_sys::Win32::Foundation::HWND
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
fn raw_is_null(raw: isize) -> bool {
    raw == 0
}

#[cfg(target_os = "windows")]
fn is_window_visible(hwnd_raw: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible;
    if raw_is_null(hwnd_raw) {
        return false;
    }
    unsafe { IsWindowVisible(raw_to_hwnd(hwnd_raw)) != 0 }
}

#[cfg(target_os = "windows")]
fn window_rect(hwnd_raw: isize) -> Option<NativeRect> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect;
    if raw_is_null(hwnd_raw) {
        return None;
    }
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ok = unsafe { GetWindowRect(raw_to_hwnd(hwnd_raw), &mut rect) };
    if ok == 0 {
        return None;
    }
    Some(NativeRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    })
}

#[cfg(target_os = "windows")]
fn client_rect(hwnd_raw: isize) -> Option<NativeRect> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect;
    if raw_is_null(hwnd_raw) {
        return None;
    }
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ok = unsafe { GetClientRect(raw_to_hwnd(hwnd_raw), &mut rect) };
    if ok == 0 {
        return None;
    }
    Some(NativeRect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    })
}

#[cfg(target_os = "windows")]
fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
