/* SPDX-License-Identifier: LGPL-3.0-or-later */

use pkg_version::*;
use std::ffi::CString;
use std::mem::ManuallyDrop;
use virt::sys;

use crate::*;

fn err_set(error_ptr: *mut *mut VirtLintError, error: VirtLintError) {
    if !error_ptr.is_null() && unsafe { *error_ptr }.is_null() {
        unsafe { *error_ptr = Box::leak(Box::new(error)) }
    }
}

macro_rules! check_not_null {
    ($var:expr, $error:ident, $ret:expr) => {
        if $var.is_null() {
            err_set($error, VirtLintError::InvalidArgument(stringify!($var)));
            return $ret;
        }
    };
}

macro_rules! c_chars_to_string {
    ($x:expr) => {{
        ::std::ffi::CStr::from_ptr($x)
            .to_string_lossy()
            .into_owned()
    }};
}

macro_rules! string_to_mut_c_chars {
    ($x:expr) => {
        ::std::ffi::CString::new($x).unwrap().into_raw() as *mut ::std::ffi::c_char
    };
}

#[no_mangle]
pub extern "C" fn virt_lint_version() -> std::ffi::c_ulong {
    pkg_version_major!() * 1000000 + pkg_version_minor!() * 1000 + pkg_version_patch!()
}

#[no_mangle]
pub extern "C" fn virt_lint_string_free(string: *mut std::ffi::c_char) {
    if !string.is_null() {
        std::mem::drop(unsafe { CString::from_raw(string) });
    }
}

#[no_mangle]
pub extern "C" fn virt_lint_error_free(err: *mut *mut VirtLintError) {
    if !err.is_null() && !unsafe { *err }.is_null() {
        std::mem::drop(unsafe { Box::from_raw(*err) });
        unsafe {
            *err = std::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub extern "C" fn virt_lint_error_get_message(err: *const VirtLintError) -> *mut std::ffi::c_char {
    let msg = if err.is_null() {
        "No error".to_string()
    } else {
        unsafe { &*err }.to_string()
    };

    string_to_mut_c_chars!(msg)
}

#[no_mangle]
pub extern "C" fn virt_lint_new(conn: sys::virConnectPtr) -> *mut VirtLint {
    let mut rust_conn = None;

    if !conn.is_null() {
        rust_conn = Some(unsafe { Connect::from_ptr(conn) });
    }

    Box::into_raw(Box::new(VirtLint::new(rust_conn.as_ref())))
}

#[no_mangle]
pub extern "C" fn virt_lint_free(vl: *mut VirtLint) {
    if !vl.is_null() {
        std::mem::drop(unsafe { Box::from_raw(vl) });
    }
}

#[no_mangle]
pub extern "C" fn virt_lint_capabilities_set(
    vl: *mut VirtLint,
    capsxml: *const std::ffi::c_char,
    err: *mut *mut VirtLintError,
) -> std::ffi::c_int {
    virt_lint_error_free(err);

    check_not_null!(vl, err, -1);

    let vl = unsafe { &mut *vl };
    let new_capsxml = if capsxml.is_null() {
        None
    } else {
        Some(unsafe { c_chars_to_string!(capsxml) })
    };

    if let Err(x) = vl.capabilities_set(new_capsxml) {
        err_set(err, x);
        return -1;
    }

    0
}

#[no_mangle]
pub extern "C" fn virt_lint_domain_capabilities_clear(
    vl: *mut VirtLint,
    err: *mut *mut VirtLintError,
) -> std::ffi::c_int {
    virt_lint_error_free(err);

    check_not_null!(vl, err, -1);

    let vl = unsafe { &mut *vl };

    vl.domain_capabilities_clear();
    0
}

#[no_mangle]
pub extern "C" fn virt_lint_domain_capabilities_add(
    vl: *mut VirtLint,
    domcapsxml: *const std::ffi::c_char,
    err: *mut *mut VirtLintError,
) -> std::ffi::c_int {
    virt_lint_error_free(err);

    check_not_null!(vl, err, -1);
    check_not_null!(domcapsxml, err, -1);

    let vl = unsafe { &mut *vl };
    let domcapsxml_string = unsafe { c_chars_to_string!(domcapsxml) };

    if let Err(x) = vl.domain_capabilities_add(domcapsxml_string) {
        err_set(err, x);
        return -1;
    }

    0
}

#[no_mangle]
pub extern "C" fn virt_lint_validate(
    vl: *mut VirtLint,
    domxml: *const std::ffi::c_char,
    tags: *mut *const std::ffi::c_char,
    ntags: usize,
    error_on_no_connect: bool,
    err: *mut *mut VirtLintError,
) -> std::ffi::c_int {
    virt_lint_error_free(err);

    check_not_null!(vl, err, -1);
    check_not_null!(domxml, err, -1);

    let vl = unsafe { &mut *vl };
    let domxml_string = unsafe { c_chars_to_string!(domxml) };

    let mut tags_vec = Vec::with_capacity(ntags);
    for i in 0..ntags {
        let t = unsafe { *tags.offset(i.try_into().unwrap()) };
        let t_string = unsafe { c_chars_to_string!(t) };
        tags_vec.push(t_string);
    }

    if let Err(x) = vl.validate(&domxml_string, &tags_vec, error_on_no_connect) {
        err_set(err, x);
        return -1;
    }

    0
}

#[no_mangle]
pub extern "C" fn virt_lint_list_tags(
    tags: *mut *mut *mut std::ffi::c_char,
    err: *mut *mut VirtLintError,
) -> isize {
    virt_lint_error_free(err);

    check_not_null!(tags, err, -1);

    let ret = VirtLint::list_validator_tags();
    if let Err(x) = ret {
        err_set(err, x);
        return -1;
    }

    let mut v: Vec<_> = ret
        .unwrap()
        .into_iter()
        .map(|s| string_to_mut_c_chars!(s))
        .collect();

    v.shrink_to_fit();

    let mut me = ManuallyDrop::new(v);
    unsafe {
        *tags = me.as_mut_ptr();
    }

    me.len().try_into().unwrap()
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CVirtLintWarning {
    tags: *mut *mut std::ffi::c_char,
    ntags: usize,
    domain: WarningDomain,
    level: WarningLevel,
    msg: *mut std::ffi::c_char,
}

#[no_mangle]
pub extern "C" fn virt_lint_get_warnings(
    vl: *mut VirtLint,
    warnings: *mut *mut CVirtLintWarning,
    err: *mut *mut VirtLintError,
) -> isize {
    virt_lint_error_free(err);

    check_not_null!(vl, err, -1);
    check_not_null!(warnings, err, -1);

    let vl = unsafe { &mut *vl };
    let mut c_warn: Vec<CVirtLintWarning> = Vec::new();

    let warn = vl.warnings();
    if warn.is_empty() {
        unsafe {
            *warnings = std::ptr::null_mut();
        }
        return 0;
    }

    warn.iter().for_each(|w| {
        let mut v: Vec<_> = w
            .tags
            .iter()
            .map(|s| string_to_mut_c_chars!(s.as_str()))
            .collect();
        v.shrink_to_fit();

        let mut me = ManuallyDrop::new(v);

        c_warn.push(CVirtLintWarning {
            tags: me.as_mut_ptr(),
            ntags: me.len(),
            domain: w.domain,
            level: w.level,
            msg: string_to_mut_c_chars!(w.msg.as_str()),
        })
    });

    let mut me = ManuallyDrop::new(c_warn);
    unsafe {
        *warnings = me.as_mut_ptr();
    }
    me.len().try_into().unwrap()
}

#[no_mangle]
pub extern "C" fn virt_lint_warnings_free(
    warnings: *mut *mut CVirtLintWarning,
    nwarnings: *mut isize,
) {
    if warnings.is_null() || nwarnings.is_null() || unsafe { *nwarnings <= 0 } {
        return;
    }

    let v = unsafe { Vec::from_raw_parts(*warnings, *nwarnings as usize, *nwarnings as usize) };

    v.iter().for_each(|s| {
        let tags = unsafe { Vec::from_raw_parts(s.tags, s.ntags, s.ntags) };

        tags.iter().for_each(|t| virt_lint_string_free(*t));
        virt_lint_string_free(s.msg)
    });

    unsafe {
        *warnings = std::ptr::null_mut();
        *nwarnings = 0;
    }
}
