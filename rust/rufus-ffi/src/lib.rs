//! Narrow C ABI for incrementally integrating Rust into Rufus.

#![no_std]

use core::ffi::c_char;
use core::slice;

use rufus_core::{
    DestructiveWritePreflight, IMAGE_FOOTER_MARGIN, MIN_TARGET_SIZE, PHYSICAL_DRIVE_PATH_CAPACITY,
    PhysicalDiskNumber, UiDriveIndex, image_fits_target, is_drive_large_enough,
    preflight_destructive_write,
};

pub const INVALID_PHYSICAL_DRIVE: i32 = -1;
pub const INVALID_UI_DRIVE_INDEX: i32 = -1;
pub const PHYSICAL_DRIVE_PATH_CAPACITY_C: usize = PHYSICAL_DRIVE_PATH_CAPACITY;
pub const MIN_TARGET_SIZE_C: u64 = MIN_TARGET_SIZE;
pub const IMAGE_FOOTER_MARGIN_C: u64 = IMAGE_FOOTER_MARGIN;
pub const PREFLIGHT_OK: i32 = 0;
pub const PREFLIGHT_INVALID_UI_INDEX: i32 = 1;
pub const PREFLIGHT_TARGET_TOO_SMALL: i32 = 2;
pub const PREFLIGHT_IMAGE_TOO_LARGE: i32 = 3;

#[allow(
    unsafe_code,
    reason = "the symbol must have a stable name for the C linker"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rufus_decode_ui_drive_index(ui_drive_index: u32) -> i32 {
    match UiDriveIndex::try_from(ui_drive_index) {
        Ok(index) => PhysicalDiskNumber::from(index).get() as i32,
        Err(_) => INVALID_PHYSICAL_DRIVE,
    }
}

/// Encode a Windows physical disk number into the existing Rufus UI index space.
///
/// Returns the UI index on success, or `-1` when the physical disk is outside
/// the supported `[0, 64)` table used by the C UI.
#[allow(
    unsafe_code,
    reason = "the symbol must have a stable name for the C linker"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rufus_encode_ui_drive_index(physical_disk_number: u32) -> i32 {
    match UiDriveIndex::try_from(PhysicalDiskNumber::new(physical_disk_number)) {
        Ok(index) => index.get() as i32,
        Err(_) => INVALID_UI_DRIVE_INDEX,
    }
}

/// Return `1` when `ui_drive_index` is inside the existing Rufus UI table.
#[allow(
    unsafe_code,
    reason = "the symbol must have a stable name for the C linker"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rufus_is_valid_ui_drive_index(ui_drive_index: u32) -> i32 {
    i32::from(UiDriveIndex::try_from(ui_drive_index).is_ok())
}

/// Format `\\.\PhysicalDriveN` into a caller-provided buffer.
///
/// Returns the number of path bytes written (excluding the trailing NUL), or
/// `-1` when `buffer` is null or shorter than needed.
#[allow(
    unsafe_code,
    reason = "C callers pass a writable path buffer that must be filled in place"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rufus_format_physical_drive_path(
    physical_disk_number: u32,
    buffer: *mut c_char,
    buffer_len: usize,
) -> i32 {
    if buffer.is_null() || buffer_len == 0 {
        return -1;
    }

    // SAFETY: the C caller provides a writable buffer of `buffer_len` bytes.
    let bytes = unsafe { slice::from_raw_parts_mut(buffer.cast::<u8>(), buffer_len) };
    match PhysicalDiskNumber::new(physical_disk_number)
        .device_path()
        .write_cstr(bytes)
    {
        Some(path_len) => path_len as i32,
        None => -1,
    }
}

/// Return `1` when `disk_size` meets Rufus' existing 8 MiB listing threshold.
#[allow(
    unsafe_code,
    reason = "the symbol must have a stable name for the C linker"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rufus_is_drive_large_enough(disk_size: u64) -> i32 {
    i32::from(is_drive_large_enough(disk_size))
}

/// Return `1` when `projected_size` fits `disk_size` plus the VHD footer margin.
#[allow(
    unsafe_code,
    reason = "the symbol must have a stable name for the C linker"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rufus_image_fits_target(projected_size: u64, disk_size: u64) -> i32 {
    i32::from(image_fits_target(projected_size, disk_size))
}

/// Final safety preflight immediately before destructive disk I/O.
///
/// When `check_image` is non-zero, `projected_size` is validated against
/// `disk_size` using the existing VHD footer margin.
#[allow(
    unsafe_code,
    reason = "the symbol must have a stable name for the C linker"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rufus_preflight_destructive_write(
    ui_drive_index: u32,
    disk_size: u64,
    check_image: i32,
    projected_size: u64,
) -> i32 {
    let projected = if check_image != 0 {
        Some(projected_size)
    } else {
        None
    };
    match preflight_destructive_write(ui_drive_index, disk_size, projected) {
        DestructiveWritePreflight::Ok => PREFLIGHT_OK,
        DestructiveWritePreflight::InvalidUiIndex => PREFLIGHT_INVALID_UI_INDEX,
        DestructiveWritePreflight::TargetTooSmall => PREFLIGHT_TARGET_TOO_SMALL,
        DestructiveWritePreflight::ImageTooLarge => PREFLIGHT_IMAGE_TOO_LARGE,
    }
}

#[cfg(not(test))]
#[allow(
    unsafe_code,
    reason = "Windows GNU unwind metadata requires this symbol even with panic=abort"
)]
#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[allow(
    unsafe_code,
    reason = "a no_std static library needs an OS fail-fast panic boundary"
)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn RaiseFailFastException(
        exception_record: *const core::ffi::c_void,
        context_record: *const core::ffi::c_void,
        flags: u32,
    ) -> !;
}

#[cfg(not(test))]
#[allow(
    unsafe_code,
    reason = "panics must terminate rather than unwind into the C caller"
)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { RaiseFailFastException(core::ptr::null(), core::ptr::null(), 0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_every_supported_ui_drive_index() {
        for physical_disk in 0..64 {
            assert_eq!(
                rufus_decode_ui_drive_index(0x80 + physical_disk),
                physical_disk as i32
            );
        }
    }

    #[test]
    fn rejects_values_outside_the_ui_drive_table() {
        for ui_index in [0, 0x7f, 0xc0, 0xc1, u32::MAX] {
            assert_eq!(
                rufus_decode_ui_drive_index(ui_index),
                INVALID_PHYSICAL_DRIVE
            );
        }
    }

    #[test]
    fn encodes_every_supported_physical_disk_number() {
        for physical_disk in 0..64 {
            assert_eq!(
                rufus_encode_ui_drive_index(physical_disk),
                (0x80 + physical_disk) as i32
            );
            assert_eq!(
                rufus_decode_ui_drive_index(0x80 + physical_disk),
                physical_disk as i32
            );
        }
    }

    #[test]
    fn rejects_physical_disk_numbers_outside_the_ui_table() {
        for physical_disk in [64, 65, u32::MAX] {
            assert_eq!(
                rufus_encode_ui_drive_index(physical_disk),
                INVALID_UI_DRIVE_INDEX
            );
        }
    }

    #[test]
    fn validates_ui_drive_indexes() {
        assert_eq!(rufus_is_valid_ui_drive_index(0x80), 1);
        assert_eq!(rufus_is_valid_ui_drive_index(0xbf), 1);
        assert_eq!(rufus_is_valid_ui_drive_index(0x7f), 0);
        assert_eq!(rufus_is_valid_ui_drive_index(0xc0), 0);
    }

    #[test]
    fn formats_physical_drive_paths_for_c_callers() {
        let mut buffer = [0u8; PHYSICAL_DRIVE_PATH_CAPACITY_C];
        let written = rufus_format_physical_drive_path(7, buffer.as_mut_ptr().cast(), buffer.len());
        assert_eq!(written, 18);
        assert_eq!(
            core::ffi::CStr::from_bytes_with_nul(&buffer[..19])
                .expect("path must be nul-terminated")
                .to_bytes(),
            br"\\.\PhysicalDrive7"
        );
    }

    #[test]
    fn rejects_null_or_undersized_path_buffers() {
        let mut tiny = [0u8; 4];
        assert_eq!(
            rufus_format_physical_drive_path(7, core::ptr::null_mut(), 32),
            -1
        );
        assert_eq!(
            rufus_format_physical_drive_path(7, tiny.as_mut_ptr().cast(), tiny.len()),
            -1
        );
    }

    #[test]
    fn reports_the_existing_drive_size_threshold() {
        assert_eq!(rufus_is_drive_large_enough(MIN_TARGET_SIZE_C - 1), 0);
        assert_eq!(rufus_is_drive_large_enough(MIN_TARGET_SIZE_C), 1);
    }

    #[test]
    fn reports_whether_an_image_fits_with_the_footer_margin() {
        let disk = 16 * 1024 * 1024;
        assert_eq!(
            rufus_image_fits_target(disk + IMAGE_FOOTER_MARGIN_C, disk),
            1
        );
        assert_eq!(
            rufus_image_fits_target(disk + IMAGE_FOOTER_MARGIN_C + 1, disk),
            0
        );
    }

    #[test]
    fn preflights_destructive_writes_for_c_callers() {
        let disk = 16 * 1024 * 1024;
        assert_eq!(
            rufus_preflight_destructive_write(0x87, disk, 1, disk),
            PREFLIGHT_OK
        );
        assert_eq!(
            rufus_preflight_destructive_write(0x7f, disk, 0, 0),
            PREFLIGHT_INVALID_UI_INDEX
        );
        assert_eq!(
            rufus_preflight_destructive_write(0x87, MIN_TARGET_SIZE_C - 1, 0, 0),
            PREFLIGHT_TARGET_TOO_SMALL
        );
        assert_eq!(
            rufus_preflight_destructive_write(0x87, disk, 1, disk + IMAGE_FOOTER_MARGIN_C + 1),
            PREFLIGHT_IMAGE_TOO_LARGE
        );
    }
}
