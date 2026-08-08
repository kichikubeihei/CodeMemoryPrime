//! Path normalization and Windows Network Share (UNC / Mapped Drive) resolution utilities.

/// Normalizes file paths across macOS, Linux, and Windows.
/// On Windows, automatically resolves mapped drive letters (e.g. `Z:\`) to UNC paths (`//SERVER/Share`)
/// using Win32 `WNetGetConnectionW`, and normalizes backslashes to forward slashes.
pub fn normalize_path(path_str: &str) -> String {
    let clean = path_str.trim();
    if clean.is_empty() {
        return String::new();
    }

    #[cfg(windows)]
    {
        let bytes = clean.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' && (bytes[0] as char).is_ascii_alphabetic() {
            let drive_letter = (bytes[0] as char).to_ascii_uppercase();
            let drive_str = format!("{}:", drive_letter);

            if let Some(unc_share) = get_win32_unc_path(&drive_str) {
                let rest = &clean[2..];
                let rest_clean = rest.trim_start_matches(|c| c == '\\' || c == '/');
                let combined = if rest_clean.is_empty() {
                    unc_share
                } else {
                    format!("{}/{}", unc_share.trim_end_matches('/'), rest_clean)
                };
                return combined.replace('\\', "/");
            }
        }
    }

    clean.replace('\\', "/")
}

#[cfg(windows)]
fn get_win32_unc_path(drive_str: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Convert "Z:" to null-terminated UTF-16 wide string
    let wide_drive: Vec<u16> = OsStr::new(drive_str).encode_wide().chain(std::iter::once(0)).collect();
    let mut buffer: [u16; 512] = [0; 512];
    let mut buffer_len: u32 = 512;

    unsafe {
        // Load mpr.dll (Windows Multi-Provider Router API) dynamically
        let lib = load_mpr_dll();
        if lib.is_null() {
            return None;
        }

        let proc_addr = get_proc_addr(lib, b"WNetGetConnectionW\0");
        if proc_addr.is_null() {
            free_lib(lib);
            return None;
        }

        type WNetGetConnectionWFn = unsafe extern "system" fn(
            lpLocalName: *const u16,
            lpRemoteName: *mut u16,
            lpnLength: *mut u32,
        ) -> u32;

        let wnet_get_connection: WNetGetConnectionWFn = std::mem::transmute(proc_addr);

        let result = wnet_get_connection(wide_drive.as_ptr(), buffer.as_mut_ptr(), &mut buffer_len);
        free_lib(lib);

        if result == 0 { // NO_ERROR (0)
            let unc_len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer_len as usize);
            if let Ok(unc_str) = String::from_utf16(&buffer[..unc_len]) {
                return Some(unc_str.replace('\\', "/"));
            }
        }
    }
    None
}

#[cfg(windows)]
unsafe fn load_mpr_dll() -> *mut std::ffi::c_void {
    extern "system" {
        fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
    }
    LoadLibraryA(b"mpr.dll\0".as_ptr())
}

#[cfg(windows)]
unsafe fn free_lib(lib: *mut std::ffi::c_void) {
    extern "system" {
        fn FreeLibrary(hLibModule: *mut std::ffi::c_void) -> i32;
    }
    FreeLibrary(lib);
}

#[cfg(windows)]
unsafe fn get_proc_addr(lib: *mut std::ffi::c_void, name: &[u8]) -> *mut std::ffi::c_void {
    extern "system" {
        fn GetProcAddress(hModule: *mut std::ffi::c_void, lpProcName: *const u8) -> *mut std::ffi::c_void;
    }
    GetProcAddress(lib, name.as_ptr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_basic() {
        assert_eq!(normalize_path("C:\\Users\\dev\\project"), "C:/Users/dev/project");
        assert_eq!(normalize_path("//SERVER/share/code"), "//SERVER/share/code");
    }
}
