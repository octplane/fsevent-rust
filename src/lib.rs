#![cfg(target_os = "macos")]
#![deny(
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

#[macro_use]
extern crate bitflags;

extern crate fsevent_sys as fsevent;

use fsevent as fs;
use fsevent::core_foundation as cf;

use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::os::raw::{c_char, c_void};
use std::ptr;

use std::sync::mpsc::Sender;

// Helper to send the runloop from an observer thread.
struct CFRunLoopSendWrapper(cf::CFRunLoopRef);

// Safety: According to the Apple documentation, it is safe to send CFRef types across threads.
//
// https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html
unsafe impl Send for CFRunLoopSendWrapper {}

pub struct FsEvent {
    paths: Vec<String>,
    since_when: fs::FSEventStreamEventId,
    latency: cf::CFTimeInterval,
    flags: fs::FSEventStreamCreateFlags,
    runloop: Option<cf::CFRunLoopRef>,
}

#[derive(Debug)]
pub struct Event {
    pub event_id: u64,
    pub flag: StreamFlags,
    pub path: String,
}

// Synchronize with
// /System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/FSEvents.framework/Versions/A/Headers/FSEvents.h
bitflags! {
  #[repr(C)]
  pub struct StreamFlags: u32 {
    const NONE = 0x00000000;
    const MUST_SCAN_SUBDIRS = 0x00000001;
    const USER_DROPPED = 0x00000002;
    const KERNEL_DROPPED = 0x00000004;
    const IDS_WRAPPED = 0x00000008;
    const HISTORY_DONE = 0x00000010;
    const ROOT_CHANGED = 0x00000020;
    const MOUNT = 0x00000040;
    const UNMOUNT = 0x00000080;
    const ITEM_CREATED = 0x00000100;
    const ITEM_REMOVED = 0x00000200;
    const INODE_META_MOD = 0x00000400;
    const ITEM_RENAMED = 0x00000800;
    const ITEM_MODIFIED = 0x00001000;
    const FINDER_INFO_MOD = 0x00002000;
    const ITEM_CHANGE_OWNER = 0x00004000;
    const ITEM_XATTR_MOD = 0x00008000;
    const IS_FILE = 0x00010000;
    const IS_DIR = 0x00020000;
    const IS_SYMLINK = 0x00040000;
    const OWN_EVENT = 0x00080000;
    const IS_HARDLINK = 0x00100000;
    const IS_LAST_HARDLINK = 0x00200000;
    const ITEM_CLONED = 0x400000;
  }
}

impl Display for StreamFlags {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.contains(StreamFlags::MUST_SCAN_SUBDIRS) {
            let _d = write!(f, "MUST_SCAN_SUBDIRS ");
        }
        if self.contains(StreamFlags::USER_DROPPED) {
            let _d = write!(f, "USER_DROPPED ");
        }
        if self.contains(StreamFlags::KERNEL_DROPPED) {
            let _d = write!(f, "KERNEL_DROPPED ");
        }
        if self.contains(StreamFlags::IDS_WRAPPED) {
            let _d = write!(f, "IDS_WRAPPED ");
        }
        if self.contains(StreamFlags::HISTORY_DONE) {
            let _d = write!(f, "HISTORY_DONE ");
        }
        if self.contains(StreamFlags::ROOT_CHANGED) {
            let _d = write!(f, "ROOT_CHANGED ");
        }
        if self.contains(StreamFlags::MOUNT) {
            let _d = write!(f, "MOUNT ");
        }
        if self.contains(StreamFlags::UNMOUNT) {
            let _d = write!(f, "UNMOUNT ");
        }
        if self.contains(StreamFlags::ITEM_CREATED) {
            let _d = write!(f, "ITEM_CREATED ");
        }
        if self.contains(StreamFlags::ITEM_REMOVED) {
            let _d = write!(f, "ITEM_REMOVED ");
        }
        if self.contains(StreamFlags::INODE_META_MOD) {
            let _d = write!(f, "INODE_META_MOD ");
        }
        if self.contains(StreamFlags::ITEM_RENAMED) {
            let _d = write!(f, "ITEM_RENAMED ");
        }
        if self.contains(StreamFlags::ITEM_MODIFIED) {
            let _d = write!(f, "ITEM_MODIFIED ");
        }
        if self.contains(StreamFlags::FINDER_INFO_MOD) {
            let _d = write!(f, "FINDER_INFO_MOD ");
        }
        if self.contains(StreamFlags::ITEM_CHANGE_OWNER) {
            let _d = write!(f, "ITEM_CHANGE_OWNER ");
        }
        if self.contains(StreamFlags::ITEM_XATTR_MOD) {
            let _d = write!(f, "ITEM_XATTR_MOD ");
        }
        if self.contains(StreamFlags::IS_FILE) {
            let _d = write!(f, "IS_FILE ");
        }
        if self.contains(StreamFlags::IS_DIR) {
            let _d = write!(f, "IS_DIR ");
        }
        if self.contains(StreamFlags::IS_SYMLINK) {
            let _d = write!(f, "IS_SYMLINK ");
        }
        if self.contains(StreamFlags::OWN_EVENT) {
            let _d = write!(f, "OWN_EVENT ");
        }
        if self.contains(StreamFlags::IS_LAST_HARDLINK) {
            let _d = write!(f, "IS_LAST_HARDLINK ");
        }
        if self.contains(StreamFlags::IS_HARDLINK) {
            let _d = write!(f, "IS_HARDLINK ");
        }
        if self.contains(StreamFlags::ITEM_CLONED) {
            let _d = write!(f, "ITEM_CLONED ");
        }
        write!(f, "")
    }
}

fn default_stream_context(event_sender: *const Sender<Event>) -> fs::FSEventStreamContext {
    let ptr = event_sender as *mut c_void;
    fs::FSEventStreamContext {
        version: 0,
        info: ptr,
        retain: None,
        release: None,
        copy_description: None,
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

impl From<std::sync::mpsc::RecvTimeoutError> for Error {
    fn from(err: std::sync::mpsc::RecvTimeoutError) -> Error {
        Self {
            msg: err.to_string(),
        }
    }
}

impl FsEvent {
    pub fn new(paths: Vec<String>) -> Self {
        Self {
            paths,
            since_when: fs::kFSEventStreamEventIdSinceNow,
            latency: 0.0,
            flags: fs::kFSEventStreamCreateFlagFileEvents | fs::kFSEventStreamCreateFlagNoDefer,
            runloop: None,
        }
    }

    // https://github.com/thibaudgg/rb-fsevent/blob/master/ext/fsevent_watch/main.c
    pub fn append_path(&mut self, source: &str) -> Result<()> {
        self.paths.push(source.to_string());
        Ok(())
    }

    fn build_native_paths(&self) -> Result<cf::CFMutableArrayRef> {
        let native_paths = unsafe {
            cf::CFArrayCreateMutable(cf::kCFAllocatorDefault, 0, &cf::kCFTypeArrayCallBacks)
        };

        if native_paths == std::ptr::null_mut() {
            Err(Error {
                msg: "Unable to allocate CFMutableArrayRef".to_string(),
            })
        } else {
            for path in &self.paths {
                unsafe {
                    let mut err = ptr::null_mut();
                    let cf_path = cf::str_path_to_cfstring_ref(path, &mut err);
                    if !err.is_null() {
                        let cf_str = cf::CFCopyDescription(err as cf::CFRef);
                        let mut buf = [0; 1024];
                        cf::CFStringGetCString(
                            cf_str,
                            buf.as_mut_ptr(),
                            buf.len() as cf::CFIndex,
                            cf::kCFStringEncodingUTF8,
                        );
                        return Err(Error {
                            msg: CStr::from_ptr(buf.as_ptr())
                                .to_str()
                                .unwrap_or("Unknown error")
                                .to_string(),
                        });
                    } else {
                        cf::CFArrayAppendValue(native_paths, cf_path);
                        cf::CFRelease(cf_path);
                    }
                }
            }

            Ok(native_paths)
        }
    }

    fn internal_observe(
        since_when: fs::FSEventStreamEventId,
        latency: cf::CFTimeInterval,
        flags: fs::FSEventStreamCreateFlags,
        paths: cf::CFMutableArrayRef,
        event_sender: Sender<Event>,
        runloop_sender: Option<Sender<CFRunLoopSendWrapper>>,
    ) -> Result<()> {
        let stream_context = default_stream_context(&event_sender);
        let paths = paths.into();

        unsafe {
            let stream = fs::FSEventStreamCreate(
                cf::kCFAllocatorDefault,
                callback,
                &stream_context,
                paths,
                since_when,
                latency,
                flags,
            );

            if let Some(ret_tx) = runloop_sender {
                let runloop = CFRunLoopSendWrapper(cf::CFRunLoopGetCurrent());
                ret_tx
                    .send(runloop)
                    .expect("unabe to send CFRunLoopRef");
            }

            fs::FSEventStreamScheduleWithRunLoop(
                stream,
                cf::CFRunLoopGetCurrent(),
                cf::kCFRunLoopDefaultMode,
            );

            fs::FSEventStreamStart(stream);
            cf::CFRunLoopRun();

            fs::FSEventStreamFlushSync(stream);
            fs::FSEventStreamStop(stream);
        }

        Ok(())
    }

    pub fn observe(&self, event_sender: Sender<Event>) {
        let native_paths = self
            .build_native_paths()
            .expect("Unable to build CFMutableArrayRef of watched paths.");
        Self::internal_observe(
            self.since_when,
            self.latency,
            self.flags,
            native_paths,
            event_sender,
            None,
        )
        .unwrap();
    }

    pub fn observe_async(&mut self, event_sender: Sender<Event>) -> Result<()> {
        let (ret_tx, ret_rx) = std::sync::mpsc::channel();
        let native_paths = self.build_native_paths()?;

        struct CFMutableArraySendWrapper(cf::CFMutableArrayRef);

        // Safety
        // - See comment on `CFRunLoopSendWrapper
        unsafe impl Send for CFMutableArraySendWrapper {}

        let safe_native_paths = CFMutableArraySendWrapper(native_paths);

        let since_when = self.since_when;
        let latency = self.latency;
        let flags = self.flags;

        std::thread::spawn(move || {
            Self::internal_observe(
                since_when,
                latency,
                flags,
                safe_native_paths.0,
                event_sender,
                Some(ret_tx),
            )
        });

        self.runloop = Some(ret_rx.recv().unwrap().0);

        Ok(())
    }

    // Shut down the event stream.
    pub fn shutdown_observe(&mut self) {
        if let Some(runloop) = self.runloop.take() {
            unsafe { cf::CFRunLoopStop(runloop); }
        }
    }
}

extern "C" fn callback(
    _stream_ref: fs::FSEventStreamRef,
    info: *mut c_void,
    num_events: usize,                               // size_t numEvents
    event_paths: *mut c_void,                        // void *eventPaths
    event_flags: *const fs::FSEventStreamEventFlags, // const FSEventStreamEventFlags eventFlags[]
    event_ids: *const fs::FSEventStreamEventId,      // const FSEventStreamEventId eventIds[]
) {
    unsafe {
        let event_paths = event_paths as *const *const c_char;
        let sender = info as *mut Sender<Event>;

        for pos in 0..num_events {
            let path = CStr::from_ptr(*event_paths.add(pos))
                .to_str()
                .expect("Invalid UTF8 string.");
            let flag = *event_flags.add(pos);
            let event_id = *event_ids.add(pos);

            let event = Event {
                event_id: event_id,
                flag: StreamFlags::from_bits(flag).unwrap_or_else(|| {
                    panic!("Unable to decode StreamFlags: {} for {}", flag, path)
                }),
                path: path.to_string(),
            };
            let _s = (*sender).send(event);
        }
    }
}
