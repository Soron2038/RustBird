#[cfg(target_os = "macos")]
mod macos {
    use crate::commands::{AppStateMutex, AudioEngineMutex};
    use crate::state::save_state;
    use std::ffi::{c_char, c_void, CString};
    use tauri::Manager;

    mod sys {
        use std::ffi::{c_char, c_void};

        pub type CFNotificationCenterRef = *mut c_void;
        pub type CFStringRef = *const c_void;
        pub type CFDictionaryRef = *const c_void;
        pub type CFNotificationName = CFStringRef;
        pub type CFNotificationCallback = extern "C" fn(
            CFNotificationCenterRef,
            *mut c_void,
            CFNotificationName,
            *const c_void,
            CFDictionaryRef,
        );

        pub const CF_STRING_ENCODING_UTF8: u32 = 0x08000100;
        pub const SUSPENSION_BEHAVIOR_DELIVER_IMMEDIATELY: i64 = 4;

        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            pub fn CFNotificationCenterGetDistributedCenter() -> CFNotificationCenterRef;
            pub fn CFNotificationCenterAddObserver(
                center: CFNotificationCenterRef,
                observer: *const c_void,
                callback: CFNotificationCallback,
                name: CFStringRef,
                object: *const c_void,
                suspension_behavior: i64,
            );
            pub fn CFStringCreateWithCString(
                alloc: *mut c_void,
                c_str: *const c_char,
                encoding: u32,
            ) -> CFStringRef;
            pub fn CFRunLoopRun();
        }
    }

    extern "C" fn on_screen_locked(
        _center: sys::CFNotificationCenterRef,
        observer: *mut c_void,
        _name: sys::CFNotificationName,
        _object: *const c_void,
        _user_info: sys::CFDictionaryRef,
    ) {
        let handle = unsafe { &*(observer as *const tauri::AppHandle) };
        let app_state_mutex = handle.state::<AppStateMutex>();
        let audio_mutex = handle.state::<AudioEngineMutex>();

        let should_pause = {
            let mut state = app_state_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
            if state.autopause_on_lock && !state.is_paused {
                state.is_paused = true;
                state.lock_triggered_pause = true;
                let _ = save_state(&state);
                true
            } else {
                false
            }
        };

        if should_pause {
            let engine = audio_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
            engine.pause_all();
        }
    }

    extern "C" fn on_screen_unlocked(
        _center: sys::CFNotificationCenterRef,
        observer: *mut c_void,
        _name: sys::CFNotificationName,
        _object: *const c_void,
        _user_info: sys::CFDictionaryRef,
    ) {
        let handle = unsafe { &*(observer as *const tauri::AppHandle) };
        let app_state_mutex = handle.state::<AppStateMutex>();
        let audio_mutex = handle.state::<AudioEngineMutex>();

        let should_resume = {
            let mut state = app_state_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
            if state.lock_triggered_pause {
                state.is_paused = false;
                state.lock_triggered_pause = false;
                let _ = save_state(&state);
                true
            } else {
                false
            }
        };

        if should_resume {
            let engine = audio_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
            engine.resume_all();
        }
    }

    pub fn start(app_handle: tauri::AppHandle) {
        // Box + leak the handle so its lifetime is 'static (lives as long as the app).
        // Cast to usize so the closure is Send (raw pointers are not Send).
        // Validity is guaranteed because the Box is never freed.
        let observer_addr = Box::into_raw(Box::new(app_handle)) as usize;

        std::thread::spawn(move || unsafe {
            let observer = observer_addr as *const c_void;
            let center = sys::CFNotificationCenterGetDistributedCenter();

            let lock_name_c = CString::new("com.apple.screenIsLocked").unwrap();
            let unlock_name_c = CString::new("com.apple.screenIsUnlocked").unwrap();

            let lock_name = sys::CFStringCreateWithCString(
                std::ptr::null_mut(),
                lock_name_c.as_ptr() as *const c_char,
                sys::CF_STRING_ENCODING_UTF8,
            );
            let unlock_name = sys::CFStringCreateWithCString(
                std::ptr::null_mut(),
                unlock_name_c.as_ptr() as *const c_char,
                sys::CF_STRING_ENCODING_UTF8,
            );

            sys::CFNotificationCenterAddObserver(
                center,
                observer,
                on_screen_locked,
                lock_name,
                std::ptr::null(),
                sys::SUSPENSION_BEHAVIOR_DELIVER_IMMEDIATELY,
            );
            sys::CFNotificationCenterAddObserver(
                center,
                observer,
                on_screen_unlocked,
                unlock_name,
                std::ptr::null(),
                sys::SUSPENSION_BEHAVIOR_DELIVER_IMMEDIATELY,
            );

            sys::CFRunLoopRun();
        });
    }
}

#[cfg(target_os = "macos")]
pub use macos::start;

#[cfg(target_os = "windows")]
mod windows_impl {
    use crate::commands::{AppStateMutex, AudioEngineMutex};
    use crate::state::save_state;
    use tauri::Manager;
    use windows::Win32::Foundation::{HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::RemoteDesktop::{
        WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongPtrW,
        RegisterClassW, SetWindowLongPtrW, GWLP_USERDATA, HWND_MESSAGE, MSG, WINDOW_EX_STYLE,
        WNDCLASSW, WS_OVERLAPPED,
    };

    // Raw WinAPI constants not re-exported by the windows crate at this version
    const WM_WTSSESSION_CHANGE: u32 = 0x02B1;
    const WTS_SESSION_LOCK: u32 = 7;
    const WTS_SESSION_UNLOCK: u32 = 8;

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_WTSSESSION_CHANGE {
            let handle_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const tauri::AppHandle;
            if !handle_ptr.is_null() {
                let handle = &*handle_ptr;
                let app_state_mutex = handle.state::<AppStateMutex>();
                let audio_mutex = handle.state::<AudioEngineMutex>();

                match wparam.0 as u32 {
                    WTS_SESSION_LOCK => {
                        let should_pause = {
                            let mut state =
                                app_state_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
                            if state.autopause_on_lock && !state.is_paused {
                                state.is_paused = true;
                                state.lock_triggered_pause = true;
                                let _ = save_state(&state);
                                true
                            } else {
                                false
                            }
                        };
                        if should_pause {
                            let engine = audio_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
                            engine.pause_all();
                        }
                        return LRESULT(0);
                    }
                    WTS_SESSION_UNLOCK => {
                        let should_resume = {
                            let mut state =
                                app_state_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
                            if state.lock_triggered_pause {
                                state.is_paused = false;
                                state.lock_triggered_pause = false;
                                let _ = save_state(&state);
                                true
                            } else {
                                false
                            }
                        };
                        if should_resume {
                            let engine = audio_mutex.0.lock().unwrap_or_else(|e| e.into_inner());
                            engine.resume_all();
                        }
                        return LRESULT(0);
                    }
                    _ => {}
                }
            }
            // Fall through to DefWindowProcW for unhandled subtypes
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    pub fn start(app_handle: tauri::AppHandle) {
        // Box + leak the handle so its lifetime is 'static, same pattern as macOS side.
        // Cast to usize to make the value Send (raw pointers are not Send).
        let handle_addr = Box::into_raw(Box::new(app_handle)) as usize;

        std::thread::spawn(move || unsafe {
            use windows::core::w;

            let hmodule: HMODULE = GetModuleHandleW(None).expect("GetModuleHandleW failed");
            let hinstance = HINSTANCE(hmodule.0);
            let class_name = w!("RustBirdLockListener");

            let wnd_class = WNDCLASSW {
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance,
                lpszClassName: class_name,
                ..Default::default()
            };
            let atom = RegisterClassW(&wnd_class);
            if atom == 0 {
                let err = windows::Win32::Foundation::GetLastError();
                // 1410 = ERROR_CLASS_ALREADY_EXISTS — safe to proceed
                if err.0 != 1410 {
                    log::error!(
                        "RegisterClassW failed: {:?}. Auto-pause on lock will not work.",
                        err
                    );
                    return;
                }
            }

            let hwnd = match CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!(""),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                None,
                hinstance,
                None,
            ) {
                Ok(h) => h,
                Err(e) => {
                    log::error!(
                        "CreateWindowExW failed: {:?}. Auto-pause on lock will not work.",
                        e
                    );
                    return;
                }
            };

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, handle_addr as isize);

            if let Err(e) = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) {
                log::error!("WTSRegisterSessionNotification failed: {:?}. Auto-pause on lock will not work.", e);
            }

            let mut msg = MSG::default();
            loop {
                match GetMessageW(&mut msg, None, 0, 0) {
                    result if result.0 > 0 => {
                        DispatchMessageW(&msg);
                    }
                    _ => break,
                }
            }
        });
    }
}

#[cfg(target_os = "windows")]
pub use windows_impl::start;
