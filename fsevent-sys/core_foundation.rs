#![allow(non_upper_case_globals, non_camel_case_types, unstable)]
extern crate libc;

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

pub type CFRef = *mut libc::c_void;

pub type CFIndex = libc::c_long;
pub type CFTimeInterval = f64;

pub type CFMutableArrayRef = CFRef;
pub type CFURLRef = CFRef;
pub type CFErrorRef = CFRef;
pub type CFStringRef = CFRef;
pub type CFRunLoopRef = CFRef;

pub const NULL: CFRef = 0 as *mut libc::c_void;

pub type CFURLPathStyle = libc::c_uint;

pub const kCFAllocatorDefault: CFRef = NULL;
pub const kCFURLPOSIXPathStyle: CFURLPathStyle = 0;
pub const kCFURLHFSPathStyle: CFURLPathStyle = 1;
pub const kCFURLWindowsPathStyle: CFURLPathStyle = 2;


#[repr(C)]
pub struct CFArrayCallBacks {
   version: CFIndex,
   retain: CFRef,
   release: CFRef,
   cp: CFRef,
   equal: CFRef,
}
impl Copy for CFArrayCallBacks { }


#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    pub static kCFTypeArrayCallBacks: CFArrayCallBacks;
    pub static kCFRunLoopDefaultMode: CFStringRef;

    pub fn Gestalt(selector: OSType, response: *const SInt32) -> OSErr;
    pub fn CFRelease(res: CFRef);
    pub fn CFShow(res: CFRef);

    pub fn CFRunLoopRun();
    pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;

    pub fn CFArrayCreateMutable(allocator: CFRef, capacity: CFIndex, callbacks: *const CFArrayCallBacks) -> CFMutableArrayRef;
    pub fn CFArrayInsertValueAtIndex(arr: CFMutableArrayRef, position: CFIndex, element: CFRef);
    pub fn CFArrayAppendValue(aff: CFMutableArrayRef, element: CFRef);
    pub fn CFArrayGetCount(arr: CFMutableArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(arr: CFMutableArrayRef, index: CFIndex) -> CFRef;

    pub fn CFURLCreateFileReferenceURL(allocator: CFRef, url: CFURLRef, err: CFRef) -> CFURLRef;
    pub fn CFURLCreateFilePathURL(allocator: CFRef, url: CFURLRef, err: CFRef) ->CFURLRef;
    pub fn CFURLCreateFromFileSystemRepresentation(allocator: CFRef, path: *const libc::c_char, len: CFIndex, is_directory: bool) -> CFURLRef;
    pub fn CFURLCopyAbsoluteURL(res: CFURLRef) -> CFURLRef;
    pub fn CFURLCopyLastPathComponent(res: CFURLRef) -> CFStringRef;
    pub fn CFURLCreateCopyDeletingLastPathComponent(allocator: CFRef, url: CFURLRef) -> CFURLRef;
    pub fn CFURLCreateCopyAppendingPathComponent(allocation: CFRef, url: CFURLRef, path: CFStringRef, is_directory: bool) -> CFURLRef;
    pub fn CFURLCopyFileSystemPath(anUrl: CFURLRef, path_style: CFURLPathStyle) -> CFStringRef;

    pub fn CFURLResourceIsReachable(res: CFURLRef, err: CFErrorRef) -> bool;
}


pub fn system_version_major() -> SInt32 {
  unsafe {
    let ret: SInt32 = 0;
    let err = Gestalt(gestaltSystemVersionMajor, &ret);
    if err != 0 {
      panic!("Gestalt call failed with error {} for gestaltSystemVersionMajor", err);
    }
    return ret;
  }

}

pub fn system_version_minor() -> SInt32 {
  unsafe {
    let ret: SInt32 = 0;
    let err = Gestalt(gestaltSystemVersionMinor, &ret);
    if err != 0 {
      panic!("Gestalt call failed with error {} for gestaltSystemVersionMinor", err);
    }
    return ret;
  }

}

pub fn system_version_bugfix() -> SInt32 {
  unsafe {
    let ret: SInt32 = 0;
    let err = Gestalt(gestaltSystemVersionBugFix, &ret);
    if err != 0 {
      panic!("Gestalt call failed with error {} for gestaltSystemVersionBugFix", err);
    }
    return ret;
  }

}

