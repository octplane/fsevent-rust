#![allow(non_upper_case_globals, non_camel_case_types)]
extern crate libc;

use core_foundation as cf;

pub type FSEventStreamRef = *mut libc::c_void;

pub type FSEventStreamCallback = extern "C" fn(
  FSEventStreamRef,  //ConstFSEventStreamRef streamRef
  *mut libc::c_void, // void *clientCallBackInfo
  libc::size_t,      // size_t numEvents
  *mut libc::c_void, // void *eventPaths
  *mut libc::c_void, // const FSEventStreamEventFlags eventFlags[]
  *mut libc::c_void, // const FSEventStreamEventId eventIds[]
);

pub type FSEventStreamEventId = u64;

pub const kFSEventStreamEventIdSinceNow: FSEventStreamEventId = 0xFFFFFFFFFFFFFFFF;

pub type FSEventStreamCreateFlags = u32;

pub const kFSEventStreamCreateFlagNone: FSEventStreamCreateFlags     = 0x00000000;
pub const kFSEventStreamCreateFlagUseCFTypes: FSEventStreamCreateFlags   = 0x00000001;
pub const kFSEventStreamCreateFlagNoDefer: FSEventStreamCreateFlags   = 0x00000002;
pub const kFSEventStreamCreateFlagWatchRoot: FSEventStreamCreateFlags   = 0x00000004;
pub const kFSEventStreamCreateFlagIgnoreSelf: FSEventStreamCreateFlags   = 0x00000008;
pub const kFSEventStreamCreateFlagFileEvents: FSEventStreamCreateFlags   = 0x00000010;

#[repr(C)]
pub struct FSEventStreamContext {
  pub version: cf::CFIndex,
  pub info: *mut libc::c_void,
  pub retain: *mut libc::c_void,
  pub copy_description: *mut libc::c_void,
}
// impl Clone for FSEventStreamContext { }
// impl Copy for FSEventStreamContext { }

#[link(name = "CoreServices", kind = "framework")]
extern "C" {

  pub fn FSEventStreamCreate(
    allocator: cf::CFRef,
    callback: *const FSEventStreamCallback,
    context: *const FSEventStreamContext,
    pathsToWatch: cf::CFMutableArrayRef,
    sinceWhen: FSEventStreamEventId,
    latency: cf::CFTimeInterval,
    flags: FSEventStreamCreateFlags ) -> FSEventStreamRef;

  pub fn FSEventStreamShow(stream_ref: FSEventStreamRef);
  pub fn FSEventStreamScheduleWithRunLoop(stream_ref: FSEventStreamRef,
    run_loop: cf::CFRunLoopRef,
    run_loop_mode: cf::CFStringRef);

  pub fn FSEventStreamUnscheduleFromRunLoop(stream_ref: FSEventStreamRef,
    run_loop: cf::CFRunLoopRef,
    run_loop_mode: cf::CFStringRef);

  pub fn FSEventStreamStart(stream_ref: FSEventStreamRef) -> bool;
  pub fn FSEventStreamFlushSync(stream_ref: FSEventStreamRef);
  pub fn FSEventStreamStop(stream_ref: FSEventStreamRef);
  pub fn FSEventStreamInvalidate(stream_ref: FSEventStreamRef);
  pub fn FSEventStreamRelease(stream_ref: FSEventStreamRef);

}
