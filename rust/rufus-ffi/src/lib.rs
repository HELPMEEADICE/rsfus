//! Narrow C ABI for incrementally integrating Rust into Rufus.

#![no_std]

use rufus_core::{PhysicalDiskNumber, UiDriveIndex};

pub const INVALID_PHYSICAL_DRIVE: i32 = -1;

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
}
