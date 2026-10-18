//! Minimal Linux libmpv 2 ABI. All native objects stay on their owning worker.

use super::{Artifact, Backend, Command, Fault, Phase, Sample, bitmap, display_size};
use std::{
    ffi::{CString, c_char, c_int, c_void},
    ptr,
    time::{Duration, Instant},
};

#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(name: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}

struct Library(*mut c_void);
impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: this non-null dlopen handle is uniquely owned, after native destruction.
        unsafe {
            dlclose(self.0);
        }
    }
}

#[repr(C)]
struct Param {
    kind: c_int,
    data: *mut c_void,
}
#[repr(C)]
struct Event {
    id: c_int,
    error: c_int,
    reply: u64,
    data: *mut c_void,
}
#[repr(C)]
struct EndFile {
    reason: c_int,
    error: c_int,
}

struct Api {
    _library: Library,
    create: unsafe extern "C" fn() -> *mut c_void,
    option: unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int,
    initialize: unsafe extern "C" fn(*mut c_void) -> c_int,
    destroy: unsafe extern "C" fn(*mut c_void),
    command: unsafe extern "C" fn(*mut c_void, u64, *const *const c_char) -> c_int,
    observe: unsafe extern "C" fn(*mut c_void, u64, *const c_char, c_int) -> c_int,
    event: unsafe extern "C" fn(*mut c_void, f64) -> *const Event,
    render_create: unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *mut Param) -> c_int,
    render_callback:
        unsafe extern "C" fn(*mut c_void, Option<unsafe extern "C" fn(*mut c_void)>, *mut c_void),
    render_update: unsafe extern "C" fn(*mut c_void) -> u64,
    render: unsafe extern "C" fn(*mut c_void, *mut Param) -> c_int,
    render_free: unsafe extern "C" fn(*mut c_void),
}

impl Api {
    fn load() -> Result<Self, Fault> {
        // SAFETY: fixed nul-terminated installed library name, RTLD_NOW, no caller path.
        let handle = unsafe { dlopen(c"libmpv.so.2".as_ptr(), 2) };
        if handle.is_null() {
            return Err(Fault::Backend);
        }
        let library = Library(handle);
        macro_rules! symbol {
            ($name:literal, $ty:ty) => {{
                // SAFETY: fixed C symbol name and a live library. Null is rejected.
                let address = unsafe { dlsym(handle, concat!($name, "\0").as_ptr().cast()) };
                if address.is_null() {
                    return Err(Fault::Backend);
                }
                // SAFETY: signatures match libmpv client.h/render.h API major 2.
                unsafe { std::mem::transmute::<*mut c_void, $ty>(address) }
            }};
        }
        let version = symbol!(
            "mpv_client_api_version",
            unsafe extern "C" fn() -> std::ffi::c_ulong
        );
        // SAFETY: version is the resolved zero-argument client ABI function.
        if unsafe { version() } >> 16 != 2 {
            return Err(Fault::Backend);
        }
        Ok(Self {
            create: symbol!("mpv_create", unsafe extern "C" fn() -> *mut c_void),
            option: symbol!(
                "mpv_set_option_string",
                unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int
            ),
            initialize: symbol!("mpv_initialize", unsafe extern "C" fn(*mut c_void) -> c_int),
            destroy: symbol!("mpv_terminate_destroy", unsafe extern "C" fn(*mut c_void)),
            command: symbol!(
                "mpv_command_async",
                unsafe extern "C" fn(*mut c_void, u64, *const *const c_char) -> c_int
            ),
            observe: symbol!(
                "mpv_observe_property",
                unsafe extern "C" fn(*mut c_void, u64, *const c_char, c_int) -> c_int
            ),
            event: symbol!(
                "mpv_wait_event",
                unsafe extern "C" fn(*mut c_void, f64) -> *const Event
            ),
            render_create: symbol!(
                "mpv_render_context_create",
                unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *mut Param) -> c_int
            ),
            render_callback: symbol!(
                "mpv_render_context_set_update_callback",
                unsafe extern "C" fn(
                    *mut c_void,
                    Option<unsafe extern "C" fn(*mut c_void)>,
                    *mut c_void,
                )
            ),
            render_update: symbol!(
                "mpv_render_context_update",
                unsafe extern "C" fn(*mut c_void) -> u64
            ),
            render: symbol!(
                "mpv_render_context_render",
                unsafe extern "C" fn(*mut c_void, *mut Param) -> c_int
            ),
            render_free: symbol!("mpv_render_context_free", unsafe extern "C" fn(*mut c_void)),
            _library: library,
        })
    }
}

#[repr(C)]
struct Property {
    name: *const c_char,
    format: c_int,
    data: *mut c_void,
}

struct Properties {
    loaded: bool,
    paused: bool,
    eof: bool,
    position_ms: u64,
    video_width: i64,
    audio_rate: i64,
    audio_output: bool,
    track_count: usize,
    track_audio: [Option<bool>; 8],
}
impl Default for Properties {
    fn default() -> Self {
        Self {
            loaded: false,
            paused: true,
            eof: false,
            position_ms: 0,
            video_width: 0,
            audio_rate: 0,
            audio_output: false,
            track_count: 0,
            track_audio: [None; 8],
        }
    }
}

pub(super) struct Native {
    api: Api,
    client: *mut c_void,
    render: *mut c_void,
    size: (u32, u32),
    pixels: Vec<u32>,
    state: Properties,
    audio_wait: Option<Instant>,
    audio_ever_ready: bool,
    deadline: Instant,
}

// mpv can invoke this from any thread. No state or library-owned pointer is accessed.
unsafe extern "C" fn changed(_: *mut c_void) {}

impl Native {
    pub(super) fn open(artifact: &Artifact) -> Result<Self, Fault> {
        Self::open_with_audio_device(artifact, None)
    }

    fn open_with_audio_device(
        artifact: &Artifact,
        audio_device: Option<&std::ffi::CStr>,
    ) -> Result<Self, Fault> {
        let metadata = std::fs::symlink_metadata(&artifact.path).map_err(|_| Fault::Missing)?;
        if !metadata.file_type().is_file() || metadata.len() != artifact.rendered.size_bytes {
            return Err(Fault::Missing);
        }
        let path = artifact.path.canonicalize().map_err(|_| Fault::Missing)?;
        let path = path
            .to_str()
            .filter(|s| s.starts_with('/'))
            .ok_or(Fault::Invalid)?;
        let api = Api::load()?;
        // SAFETY: resolved function, no arguments; the returned unique client is checked.
        let client = unsafe { (api.create)() };
        if client.is_null() {
            return Err(Fault::Backend);
        }
        let size = display_size(artifact.dimensions);
        let mut native = Self {
            api,
            client,
            render: ptr::null_mut(),
            size,
            pixels: vec![0; size.0 as usize * size.1 as usize],
            state: Properties::default(),
            audio_wait: None,
            audio_ever_ready: false,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        for (key, value) in [
            ("config", "no"),
            ("load-scripts", "no"),
            ("input-default-bindings", "no"),
            ("input-vo-keyboard", "no"),
            ("ytdl", "no"),
            ("audio-file-auto", "no"),
            ("sub-auto", "no"),
            ("access-references", "no"),
            ("demuxer-lavf-o", "protocol_whitelist=file"),
            ("demuxer", "lavf"),
            ("demuxer-lavf-format", "mov"),
            ("vo", "libmpv"),
            ("hwdec", "no"),
            ("terminal", "no"),
            ("idle", "yes"),
            ("keep-open", "yes"),
            ("pause", "yes"),
            ("audio-display", "no"),
        ] {
            let key = CString::new(key).map_err(|_| Fault::Backend)?;
            let value = CString::new(value).map_err(|_| Fault::Backend)?;
            // SAFETY: live uniquely owned uninitialized client; C strings live through call.
            if unsafe { (native.api.option)(client, key.as_ptr(), value.as_ptr()) } < 0 {
                return Err(Fault::Backend);
            }
        }
        if let Some(device) = audio_device {
            // SAFETY: private constructor option, live client; the C string lives through call.
            if unsafe { (native.api.option)(client, c"audio-device".as_ptr(), device.as_ptr()) } < 0
            {
                return Err(Fault::Backend);
            }
        }
        // SAFETY: all options are set and the live client is uniquely owned.
        if unsafe { (native.api.initialize)(client) } < 0 {
            return Err(Fault::Backend);
        }
        let mut params = [
            Param {
                kind: 1,
                data: c"sw".as_ptr().cast_mut().cast(),
            },
            Param {
                kind: 0,
                data: ptr::null_mut(),
            },
        ];
        // SAFETY: output points to native.render; params is a terminated API_TYPE list.
        if unsafe { (native.api.render_create)(&mut native.render, client, params.as_mut_ptr()) }
            < 0
        {
            return Err(Fault::Backend);
        }
        if native.render.is_null() {
            return Err(Fault::Backend);
        }
        // SAFETY: callback is static, state-free and remains valid through context destruction.
        unsafe {
            (native.api.render_callback)(native.render, Some(changed), ptr::null_mut());
        }
        for (id, name, format) in [
            (1, c"pause", 3),
            (2, c"eof-reached", 3),
            (3, c"time-pos", 5),
            (4, c"video-params/w", 4),
            (5, c"audio-out-params/samplerate", 4),
            (6, c"current-ao", 1),
            (7, c"track-list/count", 4),
        ] {
            native.observe(id, name, format)?;
        }
        for index in 0..8 {
            let name = CString::new(format!("track-list/{index}/type")).expect("fixed property");
            native.observe(100 + index, &name, 1)?;
        }
        native.cmd(&["loadfile", path])?;
        Ok(native)
    }

    fn cmd(&self, args: &[&str]) -> Result<(), Fault> {
        let strings = args
            .iter()
            .map(|s| CString::new(*s).map_err(|_| Fault::Invalid))
            .collect::<Result<Vec<_>, _>>()?;
        let mut pointers = strings.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        pointers.push(ptr::null());
        // SAFETY: live client, terminated argument pointers and strings valid until command_async copies them during the call; no core wait.
        if unsafe { (self.api.command)(self.client, 1, pointers.as_ptr()) } < 0 {
            Err(Fault::Decode)
        } else {
            Ok(())
        }
    }

    fn observe(&self, id: u64, name: &std::ffi::CStr, format: c_int) -> Result<(), Fault> {
        // SAFETY: live client and fixed property name; observe_property is render-thread-safe
        // and copies the name. Values arrive only as typed events, never synchronous queries.
        if unsafe { (self.api.observe)(self.client, id, name.as_ptr(), format) } < 0 {
            Err(Fault::Backend)
        } else {
            Ok(())
        }
    }

    fn property_event(&mut self, id: u64, property: &Property) -> Result<(), Fault> {
        // SAFETY: mpv event storage is valid until the next wait_event. Each dereference below
        // checks its exact requested format and non-null data first; no borrowed value escapes.
        unsafe {
            if !property.data.is_null() && property.format == 3 {
                let value = *property.data.cast::<c_int>() != 0;
                match id {
                    1 => self.state.paused = value,
                    2 => self.state.eof = value,
                    _ => {}
                }
            } else if !property.data.is_null() && property.format == 4 {
                let value = *property.data.cast::<i64>();
                match id {
                    4 => self.state.video_width = value,
                    5 => self.state.audio_rate = value,
                    7 => {
                        if !(0..=8).contains(&value) {
                            return Err(Fault::Decode);
                        }
                        self.state.track_count = value as usize;
                    }
                    _ => {}
                }
            } else if !property.data.is_null() && property.format == 5 && id == 3 {
                let value = *property.data.cast::<f64>();
                if value.is_finite() && value >= 0.0 {
                    self.state.position_ms = (value * 1000.0).round() as u64;
                }
            } else if property.format == 1 {
                let value = if property.data.is_null() {
                    ptr::null()
                } else {
                    *property.data.cast::<*const c_char>()
                };
                let bytes = if value.is_null() {
                    None
                } else {
                    Some(std::ffi::CStr::from_ptr(value).to_bytes())
                };
                match id {
                    6 => {
                        self.state.audio_output =
                            bytes.is_some_and(|b| !b.is_empty() && b != b"null")
                    }
                    100..=107 => {
                        self.state.track_audio[(id - 100) as usize] = bytes.map(|b| b == b"audio")
                    }
                    _ => {}
                }
            } else if property.format == 0 {
                match id {
                    4 => self.state.video_width = 0,
                    5 => self.state.audio_rate = 0,
                    6 => self.state.audio_output = false,
                    100..=107 => self.state.track_audio[(id - 100) as usize] = None,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Backend for Native {
    fn command(&mut self, command: Command) -> Result<(), Fault> {
        match command {
            Command::Pause(paused) => {
                self.cmd(&["set", "pause", if paused { "yes" } else { "no" }])
            }
            Command::Loop(looping) => {
                self.cmd(&["set", "loop-file", if looping { "inf" } else { "no" }])
            }
            Command::Seek(ms) => self.cmd(&[
                "seek",
                &format!("{:.3}", ms as f64 / 1000.0),
                "absolute+exact",
            ]),
        }
    }

    fn sample(&mut self) -> Result<Sample, Fault> {
        for _ in 0..64 {
            // SAFETY: wait_event(0) is render-thread-safe. Data is consumed before the next
            // event call; all normal control/property operations are asynchronous.
            let event =
                unsafe { (self.api.event)(self.client, 0.0).as_ref() }.ok_or(Fault::Decode)?;
            match event.id {
                0 => break,
                1 => return Err(Fault::Decode),
                5 if event.error < 0 => return Err(Fault::Decode),
                8 => self.state.loaded = true,
                7 if !event.data.is_null() => {
                    // SAFETY: END_FILE begins with these two int fields in client.h.
                    let end = unsafe { &*event.data.cast::<EndFile>() };
                    if end.error < 0 || end.reason == 4 {
                        return Err(Fault::Decode);
                    }
                }
                22 if !event.data.is_null() => {
                    let id = event.reply;
                    // SAFETY: PROPERTY_CHANGE data is a client.h mpv_event_property.
                    let property = unsafe { &*event.data.cast::<Property>() };
                    self.property_event(id, property)?;
                }
                _ => {}
            }
        }
        let tracks_ready = self.state.track_count > 0
            && self.state.track_audio[..self.state.track_count]
                .iter()
                .all(Option::is_some);
        let ready = self.state.loaded && self.state.video_width > 0 && tracks_ready;
        if !ready && Instant::now() >= self.deadline {
            return Err(Fault::Decode);
        }
        let mut frame = None;
        // SAFETY: render context is initialized, called only on its owner thread.
        if unsafe { (self.api.render_update)(self.render) } & 1 != 0 {
            let mut size = [self.size.0 as c_int, self.size.1 as c_int];
            let mut stride = self.size.0 as usize * 4;
            let mut block: c_int = 0;
            let mut params = [
                Param {
                    kind: 17,
                    data: size.as_mut_ptr().cast(),
                },
                Param {
                    kind: 18,
                    data: c"bgr0".as_ptr().cast_mut().cast(),
                },
                Param {
                    kind: 19,
                    data: (&mut stride as *mut usize).cast(),
                },
                Param {
                    kind: 20,
                    data: self.pixels.as_mut_ptr().cast(),
                },
                Param {
                    kind: 12,
                    data: (&mut block as *mut c_int).cast(),
                },
                Param {
                    kind: 0,
                    data: ptr::null_mut(),
                },
            ];
            // SAFETY: pixel buffer is 4-byte aligned, full stride*height writable; all parameter
            // storage outlives the synchronous call and the terminated list matches render.h.
            if unsafe { (self.api.render)(self.render, params.as_mut_ptr()) } < 0 {
                return Err(Fault::Decode);
            }
            frame = Some(bitmap(self.size.0, self.size.1, &self.pixels));
        }
        let has_audio = self.state.track_audio[..self.state.track_count].contains(&Some(true));
        let audio_ready = self.state.audio_output && self.state.audio_rate > 0;
        self.audio_ever_ready |= audio_ready;
        if ready && has_audio && self.state.eof && !self.audio_ever_ready {
            return Err(Fault::Decode);
        }
        if ready && has_audio && !self.state.paused && !audio_ready {
            let since = *self.audio_wait.get_or_insert_with(Instant::now);
            if since.elapsed() > Duration::from_secs(1) {
                return Err(Fault::Decode);
            }
        } else if audio_ready {
            self.audio_wait = None;
        }
        let phase = if !ready {
            Phase::Loading
        } else if self.state.eof {
            Phase::Ended
        } else if self.state.paused {
            Phase::Paused
        } else if has_audio && !audio_ready {
            Phase::Loading
        } else {
            Phase::Playing
        };
        Ok(Sample {
            phase,
            position_ms: self.state.position_ms,
            frame,
        })
    }
}

#[cfg(test)]
pub(super) fn assert_audio_output_failure(artifact: &Artifact) {
    let mut native =
        Native::open_with_audio_device(artifact, Some(c"alsa/opencut-nonexistent-device")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while native.sample().unwrap().phase == Phase::Loading {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    native.command(Command::Pause(false)).unwrap();
    loop {
        if matches!(native.sample(), Err(Fault::Decode)) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "audio output failure must not masquerade as success"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

impl Drop for Native {
    fn drop(&mut self) {
        // SAFETY: unique native handles, owner thread only; render is released before client,
        // and the Api's library remains loaded until this body and all native destruction finish.
        unsafe {
            if !self.render.is_null() {
                (self.api.render_callback)(self.render, None, ptr::null_mut());
                (self.api.render_free)(self.render);
            }
            (self.api.destroy)(self.client);
        }
    }
}
