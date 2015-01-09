#![allow(non_upper_case_globals, non_camel_case_types)]
extern crate libc;

use core_foundation as cf;

pub type FSEventStreamCreateFlags = u32;
pub type FSEventStreamEventFlags = u32;
pub type FSEventStreamRef = *mut libc::c_void;
pub type FSEventStreamEventId = u64;

pub type FSEventStreamCallback = extern "C" fn(
	FSEventStreamRef,  //ConstFSEventStreamRef streamRef
	*mut libc::c_void, // void *clientCallBackInfo
	libc::size_t,      // size_t numEvents
	*mut libc::c_void, // void *eventPaths
	*mut libc::c_void, // const FSEventStreamEventFlags eventFlags[]
	*mut libc::c_void, // const FSEventStreamEventId eventIds[]
);

pub const kFSEventStreamEventIdSinceNow: FSEventStreamEventId = 0xFFFFFFFFFFFFFFFF;

pub const kFSEventStreamCreateFlagNone:FSEventStreamCreateFlags  = 0x00000000;
pub const kFSEventStreamCreateFlagUseCFTypes:FSEventStreamCreateFlags = 0x00000001;
pub const kFSEventStreamCreateFlagNoDefer:FSEventStreamCreateFlags = 0x00000002;
pub const kFSEventStreamCreateFlagWatchRoot:FSEventStreamCreateFlags = 0x00000004;
pub const kFSEventStreamCreateFlagIgnoreSelf:FSEventStreamCreateFlags = 0x00000008;
pub const kFSEventStreamCreateFlagFileEvent:FSEventStreamCreateFlags = 0x0000001;


#[repr(C)]
pub struct FSEventStreamContext {
   pub version: cf::CFIndex,
   pub info: *mut libc::c_void,
   pub retain: *mut libc::c_void,
   pub copy_description: *mut libc::c_void,
}
impl Copy for FSEventStreamContext { }

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

}



