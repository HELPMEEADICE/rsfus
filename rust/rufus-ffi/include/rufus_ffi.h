#ifndef RUFUS_FFI_H
#define RUFUS_FFI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RUFUS_INVALID_PHYSICAL_DRIVE (-1)

int32_t rufus_decode_ui_drive_index(uint32_t ui_drive_index);

#ifdef __cplusplus
}
#endif

#endif
