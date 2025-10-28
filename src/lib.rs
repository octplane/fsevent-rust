#![cfg(target_os = "macos")]
#![deny(
    trivial_numeric_casts,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use bitflags::bitflags;
use objc2_core_foundation::{
    kCFAllocatorDefault, kCFRunLoopDefaultMode, CFArray, CFRetained, CFRunLoop, CFString,
    CFTimeInterval,
};
#[allow(deprecated)]
use objc2_core_services::FSEventStreamScheduleWithRunLoop;
use objc2_core_services::{
    kFSEventStreamCreateFlagFileEvents, kFSEventStreamCreateFlagNoDefer,
    kFSEventStreamEventFlagEventIdsWrapped, kFSEventStreamEventFlagHistoryDone,
    kFSEventStreamEventFlagItemChangeOwner, kFSEventStreamEventFlagItemCloned,
    kFSEventStreamEventFlagItemCreated, kFSEventStreamEventFlagItemFinderInfoMod,
    kFSEventStreamEventFlagItemInodeMetaMod, kFSEventStreamEventFlagItemIsDir,
    kFSEventStreamEventFlagItemIsFile, kFSEventStreamEventFlagItemIsHardlink,
    kFSEventStreamEventFlagItemIsLastHardlink, kFSEventStreamEventFlagItemIsSymlink,
    kFSEventStreamEventFlagItemModified, kFSEventStreamEventFlagItemRemoved,
    kFSEventStreamEventFlagItemRenamed, kFSEventStreamEventFlagItemXattrMod,
    kFSEventStreamEventFlagKernelDropped, kFSEventStreamEventFlagMount,
    kFSEventStreamEventFlagMustScanSubDirs, kFSEventStreamEventFlagNone,
    kFSEventStreamEventFlagOwnEvent, kFSEventStreamEventFlagRootChanged,
    kFSEventStreamEventFlagUnmount, kFSEventStreamEventFlagUserDropped,
    kFSEventStreamEventIdSinceNow, ConstFSEventStreamRef, FSEventStreamContext,
    FSEventStreamCreate, FSEventStreamCreateFlags, FSEventStreamEventFlags, FSEventStreamEventId,
    FSEventStreamFlushSync, FSEventStreamStart, FSEventStreamStop,
};
use std::{
    ffi::CStr,
    fmt::{Display, Formatter},
    os::raw::c_void,
    ptr::NonNull,
    slice,
    sync::mpsc::Sender,
};

// Helper to send the runloop from an observer thread.
struct CFRunLoopSendWrapper(CFRetained<CFRunLoop>);

// Safety: According to the Apple documentation, it is safe to send CFRef types across threads.
//
// https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html
unsafe impl Send for CFRunLoopSendWrapper {}

pub struct FsEvent {
    paths: Vec<String>,
    since_when: FSEventStreamEventId,
    latency: CFTimeInterval,
    flags: FSEventStreamCreateFlags,
    runloop: Option<CFRetained<CFRunLoop>>,
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
    const NONE = kFSEventStreamEventFlagNone;
    const MUST_SCAN_SUBDIRS = kFSEventStreamEventFlagMustScanSubDirs;
    const USER_DROPPED = kFSEventStreamEventFlagUserDropped;
    const KERNEL_DROPPED = kFSEventStreamEventFlagKernelDropped;
    const IDS_WRAPPED = kFSEventStreamEventFlagEventIdsWrapped;
    const HISTORY_DONE = kFSEventStreamEventFlagHistoryDone;
    const ROOT_CHANGED = kFSEventStreamEventFlagRootChanged;
    const MOUNT = kFSEventStreamEventFlagMount;
    const UNMOUNT = kFSEventStreamEventFlagUnmount;
    const ITEM_CREATED = kFSEventStreamEventFlagItemCreated;
    const ITEM_REMOVED = kFSEventStreamEventFlagItemRemoved;
    const INODE_META_MOD = kFSEventStreamEventFlagItemInodeMetaMod;
    const ITEM_RENAMED = kFSEventStreamEventFlagItemRenamed;
    const ITEM_MODIFIED = kFSEventStreamEventFlagItemModified;
    const FINDER_INFO_MOD = kFSEventStreamEventFlagItemFinderInfoMod;
    const ITEM_CHANGE_OWNER = kFSEventStreamEventFlagItemChangeOwner;
    const ITEM_XATTR_MOD = kFSEventStreamEventFlagItemXattrMod;
    const IS_FILE = kFSEventStreamEventFlagItemIsFile;
    const IS_DIR = kFSEventStreamEventFlagItemIsDir;
    const IS_SYMLINK = kFSEventStreamEventFlagItemIsSymlink;
    const OWN_EVENT = kFSEventStreamEventFlagOwnEvent;
    const IS_HARDLINK = kFSEventStreamEventFlagItemIsHardlink;
    const IS_LAST_HARDLINK = kFSEventStreamEventFlagItemIsLastHardlink;
    const ITEM_CLONED = kFSEventStreamEventFlagItemCloned;
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

fn default_stream_context(event_sender: *const Sender<Event>) -> FSEventStreamContext {
    let ptr = event_sender as *mut c_void;
    FSEventStreamContext {
        version: 0,
        info: ptr,
        retain: None,
        release: None,
        copyDescription: None,
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

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

impl FsEvent {
    pub fn new(paths: Vec<String>) -> Self {
        Self {
            paths,
            since_when: kFSEventStreamEventIdSinceNow,
            latency: 0.0,
            flags: kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagNoDefer,
            runloop: None,
        }
    }

    // https://github.com/thibaudgg/rb-fsevent/blob/master/ext/fsevent_watch/main.c
    pub fn append_path(&mut self, source: &str) -> Result<()> {
        self.paths.push(source.to_string());
        Ok(())
    }

    fn build_native_paths(&self) -> CFRetained<CFArray<CFString>> {
        let paths: Vec<_> = self.paths.iter().map(|x| CFString::from_str(x)).collect();
        CFArray::from_retained_objects(&paths)
    }

    fn internal_observe(
        since_when: FSEventStreamEventId,
        latency: CFTimeInterval,
        flags: FSEventStreamCreateFlags,
        paths: &CFArray<CFString>,
        event_sender: Sender<Event>,
        runloop_sender: Option<Sender<CFRunLoopSendWrapper>>,
    ) -> Result<()> {
        let stream_context = default_stream_context(&event_sender);

        unsafe {
            let stream = FSEventStreamCreate(
                kCFAllocatorDefault,
                Some(callback),
                &stream_context as *const _ as *mut _,
                paths.as_opaque(),
                since_when,
                latency,
                flags,
            );

            if let Some(ret_tx) = runloop_sender {
                let runloop = CFRunLoopSendWrapper(CFRunLoop::current().unwrap());
                ret_tx.send(runloop).expect("unabe to send CFRunLoopRef");
            }

            #[allow(deprecated)]
            FSEventStreamScheduleWithRunLoop(
                stream,
                &CFRunLoop::current().unwrap(),
                kCFRunLoopDefaultMode.unwrap(),
            );

            FSEventStreamStart(stream);
            CFRunLoop::run();

            FSEventStreamFlushSync(stream);
            FSEventStreamStop(stream);
        }

        Ok(())
    }

    pub fn observe(&self, event_sender: Sender<Event>) {
        let native_paths = self.build_native_paths();
        Self::internal_observe(
            self.since_when,
            self.latency,
            self.flags,
            &native_paths,
            event_sender,
            None,
        )
        .unwrap();
    }

    pub fn observe_async(&mut self, event_sender: Sender<Event>) -> Result<()> {
        let (ret_tx, ret_rx) = std::sync::mpsc::channel();
        let native_paths = self.build_native_paths();

        struct CFMutableArraySendWrapper(CFRetained<CFArray<CFString>>);

        // Safety
        // - See comment on `CFRunLoopSendWrapper
        unsafe impl Send for CFMutableArraySendWrapper {}

        let paths = CFMutableArraySendWrapper(native_paths);
        let since_when = self.since_when;
        let latency = self.latency;
        let flags = self.flags;

        std::thread::spawn(move || {
            Self::internal_observe(
                since_when,
                latency,
                flags,
                &paths.0,
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
            runloop.stop();
        }
    }
}

unsafe extern "C-unwind" fn callback(
    _stream_ref: ConstFSEventStreamRef,
    info: *mut c_void,
    num_events: usize,                             // size_t numEvents
    event_paths: NonNull<c_void>,                  // void *eventPaths
    event_flags: NonNull<FSEventStreamEventFlags>, // const FSEventStreamEventFlags eventFlags[]
    event_ids: NonNull<FSEventStreamEventId>,      // const FSEventStreamEventId eventIds[]
) {
    let event_paths =
        unsafe { slice::from_raw_parts(event_paths.as_ptr() as *const *const i8, num_events) };
    let event_flags = unsafe { slice::from_raw_parts(event_flags.as_ptr(), num_events) };
    let event_ids = unsafe { slice::from_raw_parts(event_ids.as_ptr(), num_events) };
    let sender = unsafe {
        (info as *mut Sender<Event>)
            .as_mut()
            .expect("Invalid Sender<Event>.")
    };
    for event in
        event_paths
            .iter()
            .zip(event_flags)
            .zip(event_ids)
            .map(|((&path, &flag), &id)| unsafe {
                let path = CStr::from_ptr(path).to_str().expect("Invalid UTF8 string.");
                Event {
                    event_id: id,
                    flag: StreamFlags::from_bits(flag).unwrap_or_else(|| {
                        panic!("Unable to decode StreamFlags: {} for {}", flag, path)
                    }),
                    path: path.to_string(),
                }
            })
    {
        let _s = sender.send(event);
    }
}
