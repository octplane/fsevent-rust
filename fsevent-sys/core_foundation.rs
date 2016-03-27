#![allow(non_upper_case_globals, non_camel_case_types)]

extern crate libc;

use std::ffi::CString;

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

pub const NULL: CFRef = 0 as CFRef;
pub const NULL_REF_PTR: *mut CFRef = 0 as *mut CFRef;

pub type CFURLPathStyle = libc::c_uint;

pub const kCFAllocatorDefault: CFRef = NULL;
pub const kCFURLPOSIXPathStyle: CFURLPathStyle = 0;
pub const kCFURLHFSPathStyle: CFURLPathStyle = 1;
pub const kCFURLWindowsPathStyle: CFURLPathStyle = 2;

pub const kCFStringEncodingUTF8: u32 = 0x08000100;
pub type CFStringEncoding = u32;

pub const kCFCompareEqualTo: i32 = 0;
pub type CFComparisonResult = i32;

// MacOS uses Case Insensitive path
pub const kCFCompareCaseInsensitive: u32 = 1;
pub type CFStringCompareFlags = u32;

// CFStringEncoding
pub type kCFStringEncoding = u32;
pub const UTF8: kCFStringEncoding = 0x08000100;



#[repr(C)]
pub struct CFArrayCallBacks {
  version: CFIndex,
  retain: CFRef,
  release: CFRef,
  cp: CFRef,
  equal: CFRef,
}
//impl Clone for CFArrayCallBacks { }


#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    pub static kCFTypeArrayCallBacks: CFArrayCallBacks;
    pub static kCFRunLoopDefaultMode: CFStringRef;

    pub fn Gestalt(selector: OSType, response: *const SInt32) -> OSErr;
    pub fn CFRelease(res: CFRef);
    pub fn CFShow(res: CFRef);

    pub fn CFRunLoopRun();
    pub fn CFRunLoopStop(run_loop: CFRunLoopRef);
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

    pub fn CFURLResourceIsReachable(res: CFURLRef, err: *mut CFErrorRef) -> bool;

    pub fn CFShowStr (str: CFStringRef);
    pub fn CFStringGetCStringPtr(theString: CFStringRef, encoding: CFStringEncoding) -> *const libc::c_char;
    pub fn CFStringCreateWithCString (alloc: CFRef, source: *const libc::c_char, encoding: kCFStringEncoding) -> CFStringRef;

    pub fn CFStringCompare(theString1: CFStringRef, theString2: CFStringRef, compareOptions: CFStringCompareFlags) -> CFComparisonResult;
    pub fn CFArrayRemoveValueAtIndex(theArray: CFMutableArrayRef, idx: CFIndex);

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

pub unsafe fn str_path_to_cfstring_ref(source: &str) -> CFStringRef {
  let c_path = CString::new(source).unwrap();
  let c_len = libc::strlen(c_path.as_ptr());
  let mut url = CFURLCreateFromFileSystemRepresentation(kCFAllocatorDefault, c_path.as_ptr(), c_len as CFIndex, false);
  let mut placeholder = CFURLCopyAbsoluteURL(url);
  CFRelease(url);

  let imaginary: CFRef = CFArrayCreateMutable(kCFAllocatorDefault, 0, &kCFTypeArrayCallBacks);

  while !CFURLResourceIsReachable(placeholder, NULL_REF_PTR) {
    let child = CFURLCopyLastPathComponent(placeholder);
    CFArrayInsertValueAtIndex(imaginary, 0, child);
    CFRelease(child);

    url = CFURLCreateCopyDeletingLastPathComponent(kCFAllocatorDefault, placeholder);
    CFRelease(placeholder);
    placeholder = url;
  }

  url = CFURLCreateFileReferenceURL(kCFAllocatorDefault, placeholder, kCFAllocatorDefault);
  CFRelease(placeholder);
  placeholder = CFURLCreateFilePathURL(kCFAllocatorDefault, url, kCFAllocatorDefault);
  CFRelease(url);

  if imaginary != kCFAllocatorDefault {
    let mut count =  0;
    while count < CFArrayGetCount(imaginary) {
      let component = CFArrayGetValueAtIndex(imaginary, count);
      url = CFURLCreateCopyAppendingPathComponent(kCFAllocatorDefault, placeholder, component, false);
      CFRelease(placeholder);
      placeholder = url;
      count = count + 1;
    }
    CFRelease(imaginary);
  }

  let cf_path = CFURLCopyFileSystemPath(placeholder, kCFURLPOSIXPathStyle);
  CFRelease(placeholder);
  cf_path
}
