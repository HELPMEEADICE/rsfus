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
#define RUFUS_MIN_TARGET_SIZE (8ULL * 1024ULL * 1024ULL)
#define RUFUS_IMAGE_FOOTER_MARGIN (4ULL * 1024ULL)

int32_t rufus_decode_ui_drive_index(uint32_t ui_drive_index);
int32_t rufus_encode_ui_drive_index(uint32_t physical_disk_number);
int32_t rufus_is_valid_ui_drive_index(uint32_t ui_drive_index);
int32_t rufus_format_physical_drive_path(uint32_t physical_disk_number,
	char* buffer, size_t buffer_len);
int32_t rufus_is_drive_large_enough(uint64_t disk_size);
int32_t rufus_image_fits_target(uint64_t projected_size, uint64_t disk_size);

#ifdef __cplusplus
}
#endif

#endif
