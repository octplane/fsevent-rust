#![allow(unstable)]

extern crate "fsevent-sys" as fsevent;
extern crate libc;

use fsevent::core_foundation as cf;
use fsevent::fsevent as fs;
use std::ffi::CString;

pub const NULL: cf::CFRef = cf::NULL;

pub struct FsEvent {
  paths: cf::CFMutableArrayRef,
  since_when: fs::FSEventStreamEventId,
  latency: cf::CFTimeInterval,
  flags: fs::FSEventStreamCreateFlags,
}
impl Copy for FsEvent { }

pub fn is_api_available() -> (bool, String) {
  let ma = cf::system_version_major();
  let mi = cf::system_version_minor();

  if ma == 10 && mi < 5 {
    return (false, "This version of OSX does not support the FSEvent library, cannot proceed".to_string());
  }
  return (true, "ok".to_string());
}

fn default_stream_context() -> fs::FSEventStreamContext {
  let stream_context = fs::FSEventStreamContext{
    version: 0,
    info: cf::NULL,
    retain: cf::NULL,
    copy_description: cf::NULL };

  stream_context
}


impl FsEvent {
  pub fn new() -> FsEvent {
    let fsevent: FsEvent;
    unsafe {
      fsevent = FsEvent{
        paths: cf::CFArrayCreateMutable(cf::kCFAllocatorDefault, 0, &cf::kCFTypeArrayCallBacks),
        since_when: fs::kFSEventStreamEventIdSinceNow,
        latency: 0.1,
        flags: fs::kFSEventStreamCreateFlagNone,
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
    let stream_context = default_stream_context();

    let cb = callback as *mut _;

    unsafe {
      let stream = fs::FSEventStreamCreate(cf::kCFAllocatorDefault,
       cb,
       &stream_context,
       self.paths,
       self.since_when,
       self.latency,
       self.flags);

      fs::FSEventStreamShow(stream);

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

pub fn callback(
    stream_ref: fs::FSEventStreamRef,
    client_callback_info: *mut libc::c_void,
    num_events: libc::size_t,      // size_t numEvents
    event_paths: *mut libc::c_void, // void *eventPaths
    event_flags: *mut libc::c_void, // const FSEventStreamEventFlags eventFlags[]
    event_ids: *mut libc::c_void,  // const FSEventStreamEventId eventIds[]
  ) {
    println!("{}", num_events);
}