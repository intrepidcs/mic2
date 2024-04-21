#pragma once

/* Generated with cbindgen:0.26.0 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

enum NeoVIMICErrType
#ifdef __cplusplus
  : uint32_t
#endif // __cplusplus
 {
  NeoVIMICErrTypeSuccess,
  NeoVIMICErrTypeFailure,
  NeoVIMICErrTypeInvalidParameter,
};
#ifndef __cplusplus
typedef uint32_t NeoVIMICErrType;
#endif // __cplusplus

typedef struct Arc_Mutex_NeoVIMIC Arc_Mutex_NeoVIMIC;

typedef struct Arc_Mutex_NeoVIMIC NeoVIMIC;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Find all neovi MIC2s.
 *
 * @param devices    Pointer to an array of NeoVIMIC structs. Initialize to nullptr. Must call mic2_free() when done.
 * @param length     Length of devices. Must point to valid memory
 *
 * @return           NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
 */
NeoVIMICErrType mic2_find(NeoVIMIC **devices,
                          uint32_t *length);

/**
 * Open the IO interface on the device.
 *
 * @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
 *
 * @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
 */
NeoVIMICErrType mic2_io_open(NeoVIMIC *device);

/**
 * Close the IO interface on the device.
 *
 * @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
 *
 * @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
 */
NeoVIMICErrType mic2_io_close(NeoVIMIC *device);

/**
 * Check if the IO interface on the device is open.
 *
 * @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
 * @param is_open   Pointer to a bool. Set to true if open, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
 *
 * @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
 */
NeoVIMICErrType mic2_io_is_open(NeoVIMIC *device,
                                bool *is_open);

/**
 * Enable the IO Buzzer on the device.
 *
 * @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
 * @param enable   Set to true to enable, false if not.
 *
 * @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
 */
NeoVIMICErrType mic2_io_buzzer_enable(NeoVIMIC *device, bool enable);

/**
 * Check if the IO Buzzer on the device is enabled.
 *
 * @param device    Pointer to aNeoVIMIC structs. Returns NeoVIMICErrTypeInvalidParameter if nullptr
 * @param is_enabled   Pointer to a bool. Set to true if enabled, false if not. Returns NeoVIMICErrTypeInvalidParameter if nullptr
 *
 * @return          NeoVIMICErrTypeSuccess if successful, NeoVIMICErrTypeFailure if not
 */
NeoVIMICErrType mic2_io_buzzer_is_enabled(NeoVIMIC *device,
                                          bool *is_enabled);

/**
 * Free the NeoVIMIC object. This must be called when finished otherwise a memory leak will occur.
 *
 * @param device    Pointer to aNeoVIMIC structs. Okay to pass a nullptr.
 *
 * @return          None
 */
void mic2_free(NeoVIMIC *device);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
