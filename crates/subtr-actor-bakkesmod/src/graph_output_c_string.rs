use super::*;

pub(crate) unsafe fn c_string_arg(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    CStr::from_ptr(value).to_str().ok().map(str::to_owned)
}
