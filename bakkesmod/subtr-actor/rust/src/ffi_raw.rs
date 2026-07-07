use std::ffi::CStr;
use std::os::raw::c_char;
use std::{ptr, slice};

pub(crate) unsafe fn raw_ref<'a, T>(value: *const T) -> Option<&'a T> {
    // SAFETY: The caller guarantees that any non-null pointer is valid for
    // shared access for the returned lifetime.
    unsafe { value.as_ref() }
}

pub(crate) unsafe fn raw_mut<'a, T>(value: *mut T) -> Option<&'a mut T> {
    // SAFETY: The caller guarantees that any non-null pointer is valid for
    // unique mutable access for the returned lifetime.
    unsafe { value.as_mut() }
}

pub(crate) unsafe fn raw_c_str<'a>(value: *const c_char) -> Option<&'a CStr> {
    if value.is_null() {
        return None;
    }
    // SAFETY: The caller guarantees `value` points to a valid null-terminated
    // C string.
    Some(unsafe { CStr::from_ptr(value) })
}

pub(crate) unsafe fn raw_c_string(value: *const c_char) -> Option<String> {
    // SAFETY: Forwarding the caller's C-string validity guarantee.
    unsafe { raw_c_str(value) }?
        .to_str()
        .ok()
        .map(str::to_owned)
}

pub(crate) unsafe fn raw_slice<'a, T>(items: *const T, count: usize) -> Result<&'a [T], ()> {
    if items.is_null() && count != 0 {
        return Err(());
    }
    if count == 0 {
        Ok(&[])
    } else {
        // SAFETY: The caller guarantees `items` points to at least `count`
        // initialized elements.
        Ok(unsafe { slice::from_raw_parts(items, count) })
    }
}

pub(crate) unsafe fn write_one<T>(out: *mut T, value: T) -> bool {
    if out.is_null() {
        return false;
    }
    // SAFETY: The caller guarantees `out` is valid writable storage for one T.
    unsafe {
        out.write(value);
    }
    true
}

pub(crate) unsafe fn copy_to_raw<T: Copy>(items: &[T], out: *mut T, max_items: usize) -> usize {
    if out.is_null() || max_items == 0 {
        return 0;
    }
    let count = items.len().min(max_items);
    // SAFETY: The caller guarantees `out` points to writable storage for at
    // least `max_items` elements. `count` is bounded by `max_items`.
    unsafe {
        ptr::copy_nonoverlapping(items.as_ptr(), out, count);
    }
    count
}

pub(crate) unsafe fn drop_raw_box<T>(value: *mut T) {
    if !value.is_null() {
        // SAFETY: The caller guarantees `value` came from Box::into_raw and has
        // not already been freed.
        drop(unsafe { Box::from_raw(value) });
    }
}
