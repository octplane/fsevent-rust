#![allow(non_upper_case_globals, non_camel_case_types)]
extern crate libc;

use std::ptr;

pub type UInt32 = libc::c_uint;
pub type SInt16 = libc::c_short;
pub type SInt32 = libc::c_int;

pub type FourCharCode = UInt32;
pub type OSType = FourCharCode;
pub type OSErr = SInt16;

pub const gestaltSystemVersion: libc::c_uint = 1937339254;
pub const gestaltSystemVersionMajor: libc::c_uint = 1937339185;
pub const gestaltSystemVersionMinor: libc::c_uint = 1937339186;
pub const gestaltSystemVersionBugFix: libc::c_uint = 1937339187;

pub const kFSEventStreamCreateFlagNoDefer: libc::c_long = 0x00000002;
pub const kFSEventStreamCreateFlagWatchRoot: libc::c_long = 0x00000004;


pub type CFRef = *mut libc::c_void;

pub type CFIndex = libc::c_long;

pub type CFMutableArrayRef = CFRef;
pub type CFURLRef = CFRef;
pub type CFErrorRef = CFRef;
pub type CFStringRef = CFRef;
pub const MNULL: CFRef = 0 as *mut libc::c_void;

//  CFURLRef url = CFURLCreateFromFileSystemRepresentation(NULL, (const UInt8*)path, (CFIndex)strlen(path), false);
// CFURLRef placeholder = CFURLCopyAbsoluteURL(url);
// CFRelease(url);


#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    pub fn Gestalt(selector: OSType, response: *const SInt32) -> OSErr;
    pub static kCFTypeArrayCallBacks: *mut libc::c_void;
    pub fn CFRelease(res: CFRef);
    pub fn CFShow(res: CFRef);

    
    pub fn CFArrayCreateMutable(allocator: *mut libc::c_void, capacity: CFIndex, callbacks: *mut libc::c_void ) -> CFMutableArrayRef;
    pub fn CFArrayInsertValueAtIndex(arr: CFMutableArrayRef, position: CFIndex, element: CFRef);
    pub fn CFArrayGetCount(arr: CFMutableArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(arr: CFMutableArrayRef, index: CFIndex) -> CFStringRef;
    
    pub fn CFURLCreateFileReferenceURL(allocator: CFRef, url: CFURLRef, err: CFRef) -> CFURLRef;
    pub fn CFURLCreateFilePathURL(allocator: CFRef, url: CFURLRef, err: CFRef) ->CFURLRef;
    pub fn CFURLCreateFromFileSystemRepresentation(allocator: CFRef, path: *const libc::c_char, len: CFIndex, is_directory: bool) -> CFURLRef;
    pub fn CFURLCopyAbsoluteURL(res: CFURLRef) -> CFURLRef;
    pub fn CFURLCopyLastPathComponent(res: CFURLRef) -> CFURLRef;
    pub fn CFURLCreateCopyDeletingLastPathComponent(allocator: CFRef, url: CFURLRef) -> CFURLRef;
    pub fn CFURLCreateCopyAppendingPathComponent(allocation: CFRef, url: CFURLRef, path: CFStringRef, is_directory: bool) -> CFURLRef;

    pub fn CFURLResourceIsReachable(res: CFURLRef, err: CFErrorRef) -> bool;
}

pub fn system_version_major() -> SInt32 {
	unsafe {
		let mut ret: SInt32 = 0;
		let err = Gestalt(gestaltSystemVersionMajor, &ret);
		if (err != 0) {
			panic!("Gestalt call failed with error {} for gestaltSystemVersionMajor", err);
		}
		return ret;
	}

}

pub fn system_version_minor() -> SInt32 {
	unsafe {
		let mut ret: SInt32 = 0;
		let err = Gestalt(gestaltSystemVersionMinor, &ret);
		if (err != 0) {
			panic!("Gestalt call failed with error {} for gestaltSystemVersionMinor", err);
		}
		return ret;
	}

}

pub fn system_version_bugfix() -> SInt32 {
	unsafe {
		let mut ret: SInt32 = 0;
		let err = Gestalt(gestaltSystemVersionBugFix, &ret);
		if (err != 0) {
			panic!("Gestalt call failed with error {} for gestaltSystemVersionBugFix", err);
		}
		return ret;
	}

}

pub fn is_api_available() -> (bool, String) {
	let ma = system_version_major();
	let mi = system_version_minor();

	if ma == 10 && mi < 5 {
		return (false, "This version of OSX does not support the FSEvent library, cannot proceed".to_string());
	}
	return (true, "ok".to_string());
}

