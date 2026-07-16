#ifndef RUFUS_FFI_H
#define RUFUS_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RUFUS_INVALID_PHYSICAL_DRIVE (-1)
#define RUFUS_INVALID_UI_DRIVE_INDEX (-1)
/* Enough for "\\.\PhysicalDrive" + max u32 digits + NUL */
#define RUFUS_PHYSICAL_DRIVE_PATH_CAPACITY 28

int32_t rufus_decode_ui_drive_index(uint32_t ui_drive_index);
int32_t rufus_encode_ui_drive_index(uint32_t physical_disk_number);
int32_t rufus_format_physical_drive_path(uint32_t physical_disk_number,
	char* buffer, size_t buffer_len);

#ifdef __cplusplus
}
#endif

#endif
