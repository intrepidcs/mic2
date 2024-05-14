#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

#include <mic2.h>
#include <time.h>

#define DEVICE_COUNT (10)
#define BUF_SIZE (1024)

// Function prototypes here
int print_error(const NeoVIMICErrType *err);
bool is_buzzer_enabled(const NeoVIMIC *device);
bool is_gpsled_enabled(const NeoVIMIC *device);

bool exercise_all_io(const NeoVIMIC *device);
bool exercise_io_buzzer(const NeoVIMIC *device);
bool exercise_io_button(const NeoVIMIC *device);
bool exercise_io_gpsled(const NeoVIMIC *device);
bool exercise_audio(const NeoVIMIC *device);
bool exercise_gps(const NeoVIMIC *device);

int main(int argc, char *argv[]) {
  (void)argc;
  (void)argv;

  printf("Finding neovi MIC2 devices...\n");
  NeoVIMIC devices[DEVICE_COUNT] = {0};
  uint32_t length = (uint32_t)DEVICE_COUNT;
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  if ((err = mic2_find(devices, &length, MIC2_API_VERSION, sizeof(NeoVIMIC))) !=
      NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  printf("Found %d neoVI MIC2 devices!\n", length);
  // Loop through all the devices found
  for (uint32_t i = 0; i < length; i++) {
    bool has_gps = false;
    if ((err = mic2_has_gps(&devices[i], &has_gps)) != NeoVIMICErrTypeSuccess) {
      return print_error(&err);
    }
    printf("Device %s has GPS: %s\n", devices[i].serial_number,
           has_gps ? "yes" : "no");

    printf("Opening IO device %s...\n", devices[i].serial_number);
    if ((err = mic2_io_open(&devices[i])) != NeoVIMICErrTypeSuccess) {
      mic2_free(&devices[i]);
      return print_error(&err);
    }

    bool success = exercise_all_io(&devices[i]);
    printf("Exercised all IO on device %s %s\n", devices[i].serial_number,
           success ? "succeeded" : "failed");

    printf("Closing IO device %s...\n", devices[i].serial_number);
    if ((err = mic2_io_close(&devices[i])) != NeoVIMICErrTypeSuccess) {
      mic2_free(&devices[i]);
      return print_error(&err);
    }

    bool gps_success = exercise_gps(&devices[i]);
    printf("Exercised GPS on device %s %s\n", devices[i].serial_number,
           gps_success ? "successfully" : "failed");

    bool audio_success = exercise_audio(&devices[i]);
    printf("Exercised audio on device %s %s\n", devices[i].serial_number,
           audio_success ? "successfully" : "failed");

    mic2_free(&devices[i]);
  }
  return 0;
}

/**
 * Prints the error message corresponding to the error code provided
 *
 * @param err a pointer to the error code to print
 * @return the error code as an int
 */
int print_error(const NeoVIMICErrType *err) {
  // Check for invalid parameter
  if (!err) {
    printf("print_error(): Invalid parameter\n");
    return NeoVIMICErrTypeInvalidParameter;
  }

  // Get error string and print it
  char buffer[BUF_SIZE];
  memset(buffer, 0, BUF_SIZE);
  uint32_t length = (uint32_t)BUF_SIZE;
  if (mic2_error_string(*err, buffer, &length) == NeoVIMICErrTypeSuccess) {
    printf("%s\n", buffer);
  } else {
    printf("Failed to get error string: %d\n", *err);
  }
  return (int)*err;
}

/**
 * This function is a helper function that checks if the buzzer is enabled or
 * disabled on the given device. It takes a pointer to a NeoVIMIC object and
 * returns a boolean value indicating whether the buzzer is enabled or disabled.
 *
 * @param device a pointer to a NeoVIMIC object representing the device to check
 * @return a boolean value indicating whether the buzzer is enabled or disabled
 */
bool is_buzzer_enabled(const NeoVIMIC *device) {
  bool is_enabled = false;
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  if ((err = mic2_io_buzzer_is_enabled((NeoVIMIC *)device, &is_enabled)) !=
      NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  printf("Buzzer is %s...\n", is_enabled ? "enabled" : "disabled");
  return is_enabled;
}

/**
 * This function is a helper function that checks if the buzzer is enabled or
 * disabled on the given device. It takes a pointer to a NeoVIMIC object and
 * returns a boolean value indicating whether the buzzer is enabled or disabled.
 *
 * @param device a pointer to a NeoVIMIC object representing the device to check
 * @return a boolean value indicating whether the buzzer is enabled or disabled
 */
bool is_gpsled_enabled(const NeoVIMIC *device) {
  bool is_enabled = false;
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  if ((err = mic2_io_gpsled_is_enabled((NeoVIMIC *)device, &is_enabled)) !=
      NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  printf("GPS LED is %s...\n", is_enabled ? "enabled" : "disabled");
  return is_enabled;
}

bool exercise_all_io(const NeoVIMIC *device) {
  const bool result = exercise_io_buzzer(device) &&
                      exercise_io_button(device) && exercise_io_gpsled(device);
  return result;
}

bool exercise_io_buzzer(const NeoVIMIC *device) {
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  // Toggle the buzzer
  bool success = !is_buzzer_enabled(device);
  if ((err = mic2_io_buzzer_enable(device, true)) != NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  bool success2 = is_buzzer_enabled(device);
  // Wait 1 second so we can hear the buzzer
  sleep(1);
  if ((err = mic2_io_buzzer_enable(device, false)) != NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  bool success3 = !is_buzzer_enabled(device);
  return success && success2 && success3;
}

bool exercise_io_button(const NeoVIMIC *device) {
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  // Read the button
  printf("Reading the button state...\n");
  for (uint32_t i = 0; i < 6; i++) {
    sleep(1);
    bool pressed = false;
    if ((err = mic2_io_button_is_pressed(device, &pressed)) !=
        NeoVIMICErrTypeSuccess) {
      return print_error(&err);
    }
    printf("Button %d is %s...\n", i, pressed ? "pressed" : "not pressed");
  }
  return err == NeoVIMICErrTypeSuccess;
}

/**
 * This function is a helper function that exercises the GPS LED of the given
 * NeoVIMIC device. It toggles the GPS LED on and off, waits for one second, and
 * then toggles it off again. It returns true if the function was successful in
 * toggling the GPS LED, and false otherwise.
 *
 * @param device a pointer to a NeoVIMIC object representing the device to check
 * @return a boolean value indicating whether the function was successful in
 *         toggling the GPS LED
 */
bool exercise_io_gpsled(const NeoVIMIC *device) {
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  // Toggle the GPS LED
  bool success = !is_gpsled_enabled(device);
  if ((err = mic2_io_gpsled_enable(device, true)) != NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  bool success2 = is_gpsled_enabled(device);
  // Wait 1 second so we can hear the gpsled
  sleep(1);
  if ((err = mic2_io_gpsled_enable(device, false)) != NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  bool success3 = !is_gpsled_enabled(device);
  return success && success2 && success3;
}

bool exercise_audio(const NeoVIMIC *device) {
  const uint32_t RECORDING_TIME = 6;
  printf("Recording %d seconds of audio...\n", RECORDING_TIME);
  bool success1 = mic2_audio_start(device, 44100) == NeoVIMICErrTypeSuccess;
  sleep(RECORDING_TIME);
  bool success2 = mic2_audio_stop(device) == NeoVIMICErrTypeSuccess;
  bool success3 = mic2_audio_save(device, "main.wav") == NeoVIMICErrTypeSuccess;

  return success1 && success2 && success3;
}

bool exercise_gps(const NeoVIMIC *device) {
  bool has_gps = false;
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  if ((err = mic2_has_gps(device, &has_gps)) != NeoVIMICErrTypeSuccess) {
    return print_error(&err);
  }
  if (!has_gps) {
    printf("This device does not have GPS.\n");
    return false;
  }

  if ((err = mic2_gps_open(device)) != NeoVIMICErrTypeSuccess) {
    return print_error(&err) == NeoVIMICErrTypeSuccess;
  }
  bool has_lock = false;
  if ((err = mic2_gps_has_lock(device, &has_lock)) != NeoVIMICErrTypeSuccess) {
    mic2_gps_close(device);
    return print_error(&err) == NeoVIMICErrTypeSuccess;
  }
  printf("GPS has lock: %s\n", has_lock ? "true" : "false");

  struct CGPSInfo info = {0};

  if ((err = mic2_gps_info(device, &info, sizeof(info))) !=
      NeoVIMICErrTypeSuccess) {
    mic2_gps_close(device);
    return print_error(&err) == NeoVIMICErrTypeSuccess;
  }
  time_t current_time = info.current_time;
  printf("Timestamp: %s\n", asctime(gmtime(&current_time)));
  printf("Longitude: %d°%c %d' %d\"  (Valid: %d)\n", info.latitude.degrees,
         info.latitude.minutes, info.latitude.seconds, info.latitude_direction,
         info.latitude_valid);
  printf("Longitude: %d°%c %d' %d\"  (Valid: %d)\n", info.longitude.degrees,
         info.longitude.minutes, info.longitude.seconds,
         info.longitude_direction, info.longitude_valid);
  printf("Altitude: %f\n", info.altitude);
  printf("NavStat: %d\n", info.nav_stat);
  printf("h_acc: %f\n", info.h_acc);
  printf("v_acc: %f\n", info.v_acc);
  printf("sog_kmh: %f\n", info.sog_kmh);
  printf("cog: %f\n", info.cog);
  printf("vvel: %f\n", info.vvel);
  printf("age_c: %f\n", info.age_c);
  printf("hdop: %f\n", info.hdop);
  printf("vdop: %f\n", info.vdop);
  printf("tdop: %f\n", info.tdop);
  printf("Satellite count: %d\n", info.satellites_count);
  for (uint8_t i = 0; i < info.satellites_count; i++) {
    printf("\t%d. Satellite PRN: %d: SNR: %d SNR valid: %d\n", i,
           info.satellites[i].prn, info.satellites[i].snr,
           info.satellites[i].snr_valid);
  }
  printf("Clock Bias: %f\n", info.clock_bias);
  printf("Clock Drift: %f\n", info.clock_drift);
  printf("Timepulse granularity: %f\n", info.timepulse_granularity);

  if ((err = mic2_gps_close(device)) != NeoVIMICErrTypeSuccess) {
    return print_error(&err) == NeoVIMICErrTypeSuccess;
  }

  return err == NeoVIMICErrTypeSuccess;
}
