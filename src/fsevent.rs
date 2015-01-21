#![allow(unstable)]

extern crate "fsevent-sys" as fsevent;
extern crate libc;

use fsevent::core_foundation as cf;
use fsevent::fsevent as fs;

use std::fmt::{Error, Show, Formatter};
use std::result::Result;
use std::ffi::CString;
use std::mem::transmute;
use std::slice::from_raw_mut_buf;
use std::raw::Slice;
use std::str::from_utf8;
use std::ffi::c_str_to_bytes;


pub const NULL: cf::CFRef = cf::NULL;

pub struct FsEvent<'a> {
  paths: cf::CFMutableArrayRef,
  since_when: fs::FSEventStreamEventId,
  latency: cf::CFTimeInterval,
  flags: fs::FSEventStreamCreateFlags,
  pub callback: FsEventCallback,
}

#[derive(Show)]
pub struct Event {
  event_id: u64,
  flag: StreamFlags,
  path: String,
}

pub type FsEventCallback = fn(Vec<Event>);

bitflags! {
  flags StreamFlags: u32 {
    const NONE = 0x00000000,
    const MUST_SCAN_SUBDIRS = 0x00000001,
    const USER_DROPPED = 0x00000002,
    const KERNEL_DROPPED = 0x00000004,
    const IDS_WRAPPED = 0x00000008,
    const HISTORY_DONE = 0x00000010,
    const ROOT_CHANGED = 0x00000020,
    const MOUNT = 0x00000040,
    const UNMOUNT = 0x00000080,
    const ITEM_CREATED = 0x00000100,
    const ITEM_REMOVED = 0x00000200,
    const INOTE_META_MOD = 0x00000400,
    const ITEM_RENAMED = 0x00000800,
    const ITEM_MODIFIED = 0x00001000,
    const FINDER_INFO_MOD = 0x00002000,
    const ITEM_CHANGE_OWNER = 0x00004000,
    const ITEM_XATTR_MOD = 0x00008000,
    const IS_FILE = 0x00010000,
    const IS_DIR = 0x00020000,
    const IS_SYMLIMK = 0x00040000,
  }
}

impl Show for StreamFlags {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
      write!(f, "{}", self.bits())
    }
}

pub fn is_api_available() -> (bool, String) {
  let ma = cf::system_version_major();
  let mi = cf::system_version_minor();

  if ma == 10 && mi < 5 {
    return (false, "This version of OSX does not support the FSEvent library, cannot proceed".to_string());
  }
  return (true, "ok".to_string());
}


fn default_stream_context(info: *const FsEvent) -> fs::FSEventStreamContext {
  let ptr = info as *mut libc::c_void;
  let stream_context = fs::FSEventStreamContext{
    version: 0,
    info: ptr,
    retain: cf::NULL,
    copy_description: cf::NULL };

  stream_context
}

impl<'a> FsEvent<'a> {
  pub fn new(callback: FsEventCallback) -> FsEvent<'a> {
    let fsevent: FsEvent;
    unsafe {
      fsevent = FsEvent{
        paths: cf::CFArrayCreateMutable(cf::kCFAllocatorDefault, 0, &cf::kCFTypeArrayCallBacks),
        since_when: fs::kFSEventStreamEventIdSinceNow,
        latency: 0.1,
        flags: fs::kFSEventStreamCreateFlagFileEvents,
        callback: callback,
      };
    }
    fsevent
  }

  // https://github.com/thibaudgg/rb-fsevent/blob/master/ext/fsevent_watch/main.c
  pub fn append_path(&self,source: &str) {
    unsafe {
      let cp = CString::from_slice(source.as_bytes());
      let c_path = cp.as_slice_with_nul();
      let mut url = cf::CFURLCreateFromFileSystemRepresentation(cf::kCFAllocatorDefault, c_path.as_ptr(), c_path.len() as i64, false);
      let mut placeholder = cf::CFURLCopyAbsoluteURL(url);
      cf::CFRelease(url);

      let imaginary: cf::CFRef = cf::CFArrayCreateMutable(cf::kCFAllocatorDefault, 0, &cf::kCFTypeArrayCallBacks);

      while !cf::CFURLResourceIsReachable(placeholder, cf::kCFAllocatorDefault) {

        let child = cf::CFURLCopyLastPathComponent(placeholder);
        cf::CFArrayInsertValueAtIndex(imaginary, 0, child);
        cf::CFRelease(child);

        url = cf::CFURLCreateCopyDeletingLastPathComponent(cf::kCFAllocatorDefault, placeholder);
        cf::CFRelease(placeholder);
        placeholder = url;
      }

      url = cf::CFURLCreateFileReferenceURL(cf::kCFAllocatorDefault, placeholder, cf::kCFAllocatorDefault);
      cf::CFRelease(placeholder);
      placeholder = cf::CFURLCreateFilePathURL(cf::kCFAllocatorDefault, url, cf::kCFAllocatorDefault);
      cf::CFRelease(url);

      if imaginary != cf::kCFAllocatorDefault {
        let mut count =  0;
        while { count < cf::CFArrayGetCount(imaginary) }
        {
          let component = cf::CFArrayGetValueAtIndex(imaginary, count);
          url = cf::CFURLCreateCopyAppendingPathComponent(cf::kCFAllocatorDefault, placeholder, component, false);
          cf::CFRelease(placeholder);
          placeholder = url;
          count = count + 1;
        }
        cf::CFRelease(imaginary);
      }

      let cf_path = cf::CFURLCopyFileSystemPath(placeholder, cf::kCFURLPOSIXPathStyle);
      cf::CFArrayAppendValue(self.paths, cf_path);
      cf::CFRelease(cf_path);
      cf::CFRelease(placeholder);
    }
  }
  pub fn observe(&self) {
    let stream_context = default_stream_context(self);

    let cb = callback as *mut _;

    unsafe {
      let stream = fs::FSEventStreamCreate(cf::kCFAllocatorDefault,
       cb,
       &stream_context,
       self.paths,
       self.since_when,
       self.latency,
       self.flags);

      // fs::FSEventStreamShow(stream);

      fs::FSEventStreamScheduleWithRunLoop(stream,
        cf::CFRunLoopGetCurrent(),
        cf::kCFRunLoopDefaultMode);

      fs::FSEventStreamStart(stream);
      cf::CFRunLoopRun();
      fs::FSEventStreamFlushSync(stream);
      fs::FSEventStreamStop(stream);

    }
  }
}


#[allow(unused_variables)]
pub fn callback(
    stream_ref: fs::FSEventStreamRef,
    info: *mut libc::c_void,
    num_events: libc::size_t,      // size_t numEvents
    event_paths: *const *const libc::c_char, // void *eventPaths
    event_flags: *mut libc::c_void, // const FSEventStreamEventFlags eventFlags[]
    event_ids: *mut libc::c_void,  // const FSEventStreamEventId eventIds[]
  ) {
    let num = num_events as usize;
    let e_ptr = event_flags as *mut u32;
    let i_ptr = event_ids as *mut u64;
    let fs_event = info as *mut FsEvent;

    let mut events: Vec<Event> = Vec::new();

    unsafe {
      let paths: &[*const libc::c_char] = transmute(Slice { data: event_paths, len: num });
      let flags = from_raw_mut_buf(&e_ptr, num);
      let ids = from_raw_mut_buf(&i_ptr, num);

      for p in (0..num) {
        let i = c_str_to_bytes(&paths[p]);
        let flag: StreamFlags = StreamFlags::from_bits(flags[p] as u32)
        .expect(format!("Unable to decode StreamFlags: {}", flags[p] as u32).as_slice());

        let path = from_utf8(i).ok().expect("Invalid UTF8 string.");
        events.push(Event{event_id: ids[p], flag: flag, path: path.to_string()});
      }

      ((*fs_event).callback)(events)
    }

}