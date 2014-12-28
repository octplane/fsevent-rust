extern crate libc;

pub type UInt32 = libc::c_uint;
pub type SInt16 = libc::c_short;
pub type SInt32 = libc::c_int;

pub type FourCharCode = UInt32;
pub type OSType = FourCharCode;
pub type OSErr = SInt16;

#[repr(C)]
pub const gestaltSystemVersion: libc::c_uint = 1937339254;
#[repr(C)]
pub const gestaltSystemVersionMajor: libc::c_uint = 1937339185;
#[repr(C)]
pub const gestaltSystemVersionMinor: libc::c_uint = 1937339186;
#[repr(C)]
pub const gestaltSystemVersionBugFix: libc::c_uint = 1937339187;

#[repr(C)]
pub const kFSEventStreamCreateFlagNoDefer: libc::c_long = 0x00000002;
#[repr(C)]
pub const kFSEventStreamCreateFlagWatchRoot: libc::c_long = 0x00000004;


#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    pub fn Gestalt(selector: OSType, response: *const SInt32) -> OSErr;
    pub static kCFTypeArrayCallBacks: *mut libc::c_void;
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