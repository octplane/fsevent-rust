#![deny(trivial_numeric_casts, unstable_features, unused_import_braces, unused_qualifications)]
#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

#[macro_use]
extern crate bitflags;

extern crate fsevent_sys as fsevent;

use fsevent::core_foundation as cf;
use fsevent as fs;

use std::convert::AsRef;
use std::ffi::CStr;
use std::ptr;
use std::slice;
use std::slice::from_raw_parts_mut;
use std::str::from_utf8;

use std::sync::mpsc::Sender;

pub struct FsEvent {
    paths: cf::CFMutableArrayRef,
    since_when: fs::FSEventStreamEventId,
    latency: cf::CFTimeInterval,
    flags: fs::FSEventStreamCreateFlags,
    sender: Sender<Event>,
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
    const INOTE_META_MOD = 0x00000400;
    const ITEM_RENAMED = 0x00000800;
    const ITEM_MODIFIED = 0x00001000;
    const FINDER_INFO_MOD = 0x00002000;
    const ITEM_CHANGE_OWNER = 0x00004000;
    const ITEM_XATTR_MOD = 0x00008000;
    const IS_FILE = 0x00010000;
    const IS_DIR = 0x00020000;
    const IS_SYMLIMK = 0x00040000;
    const OWN_EVENT = 0x00080000;
    const IS_HARDLINK = 0x00100000;
    const IS_LAST_HARDLINK = 0x00200000;
    const ITEM_CLONED = 0x400000;
  }
}

impl std::fmt::Display for StreamFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        if self.contains(StreamFlags::INOTE_META_MOD) {
            let _d = write!(f, "INOTE_META_MOD ");
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
        if self.contains(StreamFlags::IS_SYMLIMK) {
            let _d = write!(f, "IS_SYMLIMK ");
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

fn default_stream_context(info: *const FsEvent) -> fs::FSEventStreamContext {
    let ptr = info as *mut ::std::os::raw::c_void;
    fs::FSEventStreamContext {
        version: 0,
        info: ptr,
        retain: cf::NULL,
        copy_description: cf::NULL,
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

impl From<std::sync::mpsc::RecvTimeoutError> for Error {
    fn from(err: std::sync::mpsc::RecvTimeoutError) -> Error {
        use std::error::Error;

        Self {
            msg: err.description().to_string(),
        }
    }
}

impl FsEvent {
    pub fn new(sender: Sender<Event>) -> FsEvent {
        let fsevent: FsEvent;

        unsafe {
            fsevent = FsEvent {
                paths: FsEventHandle::from(cf::CFArrayCreateMutable(
                    cf::kCFAllocatorDefault,
                    0,
                    &cf::kCFTypeArrayCallBacks,
                )),
                since_when: fs::kFSEventStreamEventIdSinceNow,
                latency: 0.0,
                flags: fs::kFSEventStreamCreateFlagFileEvents | fs::kFSEventStreamCreateFlagNoDefer,
                sender,
            };
        }
        fsevent
    }

    // https://github.com/thibaudgg/rb-fsevent/blob/master/ext/fsevent_watch/main.c
    pub fn append_path(&self, source: &str) -> Result<()> {
        unsafe {
            let mut err = ptr::null_mut();
            let cf_path = cf::str_path_to_cfstring_ref(source, &mut err);
            if !err.is_null() {
                let cf_str = cf::CFCopyDescription(err as cf::CFRef);
                let mut buf = [0; 1024];
                cf::CFStringGetCString(
                    cf_str,
                    buf.as_mut_ptr(),
                    buf.len() as cf::CFIndex,
                    cf::kCFStringEncodingUTF8,
                );
                Err(Error {
                    msg: CStr::from_ptr(buf.as_ptr())
                        .to_str()
                        .unwrap_or("Unknown error")
                        .to_string(),
                })
            } else {
                cf::CFArrayAppendValue(self.paths, cf_path);
                cf::CFRelease(cf_path);
                Ok(())
            }
        }
    }
    pub fn observe(&self) {
        let stream_context = default_stream_context(self);

        let cb = callback as *mut _;

        unsafe {
            let stream = fs::FSEventStreamCreate(
                cf::kCFAllocatorDefault,
                cb,
                &stream_context,
                self.paths,
                self.since_when,
                self.latency,
                self.flags,
            );

            // fs::FSEventStreamShow(stream);

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
    }

    pub fn observe_async(&self) -> Result<FsEventHandle> {
        let (ret_tx, ret_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let stream_context = default_stream_context(self);

            let cb = callback as *mut _;

            unsafe {
                let stream = fs::FSEventStreamCreate(
                    cf::kCFAllocatorDefault,
                    cb,
                    &stream_context,
                    self.paths.into(),
                    self.since_when,
                    self.latency,
                    self.flags,
                );

                // fs::FSEventStreamShow(stream);

                let runloop_ref = cf::CFRunLoopGetCurrent();
                let runloop_ref_safe = FsEventHandle::from(runloop_ref);
                ret_tx.send(runloop_ref_safe).expect(&format!("Unable to return CFRunLoopRef ({:#X})", runloop_ref_safe.ptr));

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
        });

        match ret_rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::from(e))
        }
    }
}

#[allow(unused_variables)]
unsafe fn callback(
    stream_ref: fs::FSEventStreamRef,
    info: *mut ::std::os::raw::c_void,
    num_events: usize,                                 // size_t numEvents
    event_paths: *const *const ::std::os::raw::c_char, // void *eventPaths
    event_flags: *mut ::std::os::raw::c_void,          // const FSEventStreamEventFlags eventFlags[]
    event_ids: *mut ::std::os::raw::c_void,            // const FSEventStreamEventId eventIds[]
) {
    let num = num_events;
    let e_ptr = event_flags as *mut u32;
    let i_ptr = event_ids as *mut u64;
    let fs_event = info as *mut FsEvent;

    let paths: &[*const ::std::os::raw::c_char] =
        std::mem::transmute(slice::from_raw_parts(event_paths, num));
    let flags = from_raw_parts_mut(e_ptr, num);
    let ids = from_raw_parts_mut(i_ptr, num);

    for p in 0..num {
        let i = CStr::from_ptr(paths[p]).to_bytes();
        let path = from_utf8(i).expect("Invalid UTF8 string.");
        let flag: StreamFlags = StreamFlags::from_bits(flags[p])
            .expect(format!("Unable to decode StreamFlags: {} for {}", flags[p], path).as_ref());
        // println!("{}: {}", ids[p], flag);

        let event = Event {
            event_id: ids[p],
            flag,
            path: path.to_string(),
        };
        let _s = (*fs_event).sender.send(event);
    }
}
