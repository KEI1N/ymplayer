use crate::utils::{Result, YPlayerError};
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct MpvPlayer {
    ctx: *mut libmpv_sys::mpv_handle,
    running: Arc<AtomicBool>,
}

unsafe impl Send for MpvPlayer {}
unsafe impl Sync for MpvPlayer {}

impl MpvPlayer {
    pub fn new(token: Option<&str>) -> Result<Self> {
        let ctx = unsafe { libmpv_sys::mpv_create() };
        if ctx.is_null() {
            return Err(YPlayerError::Mpv("mpv_create failed".into()));
        }

        // Set options before init
        set_opt(ctx, "vo", "null")?;
        set_opt(ctx, "video", "no")?;
        set_opt(ctx, "audio-display", "no")?;
        set_opt(ctx, "force-seekable", "yes")?;
        set_opt(ctx, "msg-level", "all=warn")?;
        set_opt(ctx, "cache", "yes")?;
        set_opt(ctx, "cache-secs", "60")?;
        set_opt(ctx, "user-agent", "YandexMusicAndroid/24023621")?;

        if let Some(t) = token {
            let hdr = format!("Authorization: OAuth {t}");
            set_opt(ctx, "http-header-fields", &hdr)?;
        }

        let ret = unsafe { libmpv_sys::mpv_initialize(ctx) };
        if ret < 0 {
            let err = unsafe {
                let ptr = libmpv_sys::mpv_error_string(ret);
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            };
            return Err(YPlayerError::Mpv(format!("mpv_initialize: {err}")));
        }

        Ok(Self {
            ctx,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn load(&self, url: &str) -> Result<()> {
        let cmd = CString::new("loadfile").map_err(|_| YPlayerError::Mpv("Invalid cmd".into()))?;
        let url_c = CString::new(url).map_err(|_| YPlayerError::Mpv("Invalid URL".into()))?;
        let mode_c = CString::new("replace").map_err(|_| YPlayerError::Mpv("Invalid mode".into()))?;

        let mut args: [*const std::os::raw::c_char; 4] = [
            cmd.as_ptr(),
            url_c.as_ptr(),
            mode_c.as_ptr(),
            std::ptr::null(),
        ];

        let ret = unsafe {
            libmpv_sys::mpv_command(self.ctx, args.as_mut_ptr())
        };

        if ret < 0 {
            let err = unsafe {
                let ptr = libmpv_sys::mpv_error_string(ret);
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            };
            return Err(YPlayerError::Mpv(format!("loadfile: {err}")));
        }

        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        set_flag(self.ctx, "pause", false)
    }

    pub fn pause(&self) -> Result<()> {
        set_flag(self.ctx, "pause", true)
    }

    pub fn toggle_playback(&self) -> Result<bool> {
        let paused = self.paused();
        set_flag(self.ctx, "pause", !paused)?;
        Ok(!paused)
    }

    pub fn paused(&self) -> bool {
        get_flag(self.ctx, "pause").unwrap_or(true)
    }

    pub fn stop(&self) -> Result<()> {
        let cmd_c = CString::new("stop").map_err(|_| YPlayerError::Mpv("Invalid cmd".into()))?;
        let mut args: [*const std::os::raw::c_char; 2] = [cmd_c.as_ptr(), std::ptr::null()];

        let ret = unsafe { libmpv_sys::mpv_command(self.ctx, args.as_mut_ptr()) };
        if ret < 0 {
            let err = unsafe { CStr::from_ptr(libmpv_sys::mpv_error_string(ret)).to_string_lossy().into_owned() };
            return Err(YPlayerError::Mpv(format!("stop: {err}")));
        }
        Ok(())
    }

    pub fn volume(&self) -> u8 {
        let vol = get_double(self.ctx, "volume").unwrap_or(80.0);
        vol as u8
    }

    pub fn set_volume(&self, vol: u8) -> Result<()> {
        let vol_c = CString::new(vol.min(100).to_string())
            .map_err(|_| YPlayerError::Mpv("Invalid volume".into()))?;
        set_opt(self.ctx, "volume", vol_c.to_str().unwrap_or("80"))
    }

    pub fn adjust_volume(&self, delta: i8) -> Result<u8> {
        let current = self.volume();
        let new = (current as i16 + delta as i16).clamp(0, 100) as u8;
        self.set_volume(new)?;
        Ok(new)
    }

    pub fn seek(&self, secs: f64) -> Result<()> {
        let cmd_name = CString::new("seek").map_err(|_| YPlayerError::Mpv("Invalid cmd".into()))?;
        let rel = if secs >= 0.0 { "+" } else { "" };
        let val = format!("{}{}", rel, secs);
        let val_c = CString::new(val).map_err(|_| YPlayerError::Mpv("Invalid seek value".into()))?;
        let mode_c = CString::new("relative").map_err(|_| YPlayerError::Mpv("Invalid mode".into()))?;

        let mut args: [*const std::os::raw::c_char; 4] = [
            cmd_name.as_ptr(),
            val_c.as_ptr(),
            mode_c.as_ptr(),
            std::ptr::null(),
        ];

        let ret = unsafe { libmpv_sys::mpv_command(self.ctx, args.as_mut_ptr()) };
        if ret < 0 {
            let err = unsafe { CStr::from_ptr(libmpv_sys::mpv_error_string(ret)).to_string_lossy().into_owned() };
            return Err(YPlayerError::Mpv(format!("seek: {err}")));
        }
        Ok(())
    }

    pub fn duration(&self) -> Option<f64> {
        get_double(self.ctx, "duration")
    }

    pub fn time_pos(&self) -> Option<f64> {
        get_double(self.ctx, "time-pos")
    }

    pub fn eof_reached(&self) -> bool {
        get_flag(self.ctx, "eof-reached").unwrap_or(true)
    }

    /// Drain all pending mpv events without processing them.
    /// Use after loadfile replace to discard stale END_FILE events.
    pub fn drain_events(&self) {
        while self.poll_event().is_some() {}
    }

    /// Poll for mpv events - call this periodically
    pub fn poll_event(&self) -> Option<i64> {
        unsafe {
            let event = libmpv_sys::mpv_wait_event(self.ctx, 0.0);
            if event.is_null() {
                return None;
            }
            let event_id = (*event).event_id as i64;
            if event_id == libmpv_sys::mpv_event_id_MPV_EVENT_NONE as i64 {
                return None;
            }
            Some(event_id)
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if !self.ctx.is_null() {
            unsafe {
                libmpv_sys::mpv_terminate_destroy(self.ctx);
            }
        }
    }
}

fn set_opt(ctx: *mut libmpv_sys::mpv_handle, name: &str, val: &str) -> Result<()> {
    let name_c = CString::new(name).map_err(|_| YPlayerError::Mpv("Invalid option name".into()))?;
    let val_c = CString::new(val).map_err(|_| YPlayerError::Mpv("Invalid option value".into()))?;

    let ret = unsafe { libmpv_sys::mpv_set_option_string(ctx, name_c.as_ptr(), val_c.as_ptr()) };
    if ret < 0 {
        let err = unsafe { CStr::from_ptr(libmpv_sys::mpv_error_string(ret)).to_string_lossy().into_owned() };
        return Err(YPlayerError::Mpv(format!("set_option '{name}': {err}")));
    }
    Ok(())
}

fn set_flag(ctx: *mut libmpv_sys::mpv_handle, name: &str, val: bool) -> Result<()> {
    let name_c = CString::new(name).map_err(|_| YPlayerError::Mpv("Invalid name".into()))?;
    let val_i64: i64 = if val { 1 } else { 0 };

    let val_void: *mut std::ffi::c_void = &val_i64 as *const _ as *mut _;
    let ret = unsafe {
        libmpv_sys::mpv_set_property(
            ctx,
            name_c.as_ptr(),
            libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
            val_void,
        )
    };
    if ret < 0 {
        let err = unsafe { CStr::from_ptr(libmpv_sys::mpv_error_string(ret)).to_string_lossy().into_owned() };
        return Err(YPlayerError::Mpv(format!("set_flag '{name}': {err}")));
    }
    Ok(())
}

fn get_flag(ctx: *mut libmpv_sys::mpv_handle, name: &str) -> Option<bool> {
    let name_c = CString::new(name).ok()?;
    let mut val: i64 = 0;

    let ret = unsafe {
        libmpv_sys::mpv_get_property(
            ctx,
            name_c.as_ptr(),
            libmpv_sys::mpv_format_MPV_FORMAT_FLAG,
            &mut val as *mut _ as *mut std::ffi::c_void,
        )
    };
    if ret < 0 {
        return None;
    }
    Some(val != 0)
}

fn get_double(ctx: *mut libmpv_sys::mpv_handle, name: &str) -> Option<f64> {
    let name_c = CString::new(name).ok()?;
    let mut val: f64 = 0.0;

    let ret = unsafe {
        libmpv_sys::mpv_get_property(
            ctx,
            name_c.as_ptr(),
            libmpv_sys::mpv_format_MPV_FORMAT_DOUBLE,
            &mut val as *mut _ as *mut std::ffi::c_void,
        )
    };
    if ret < 0 {
        return None;
    }
    Some(val)
}