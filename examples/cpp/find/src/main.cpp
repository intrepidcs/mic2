#include <chrono>
#include <iostream>
#include <mic2.hpp>
#include <thread>
#include <vector>

int main(int argc, char* argv[]) {
  (void)argc;
  (void)argv;

  auto devices = mic2::find();
  if (!devices) {
    std::cout << "Failed to find devices: "
              << static_cast<uint32_t>(devices.error()) << "\n";
    return 1;
  }
  std::cout << "Found " << devices.value().size() << " device(s)\n";
  for (int i=0; i<50; ++i) {
    for (auto& device : devices.value()) {
      // Open IO
      if (auto result = device.io_open(); !result.has_value()) {
        std::cerr << "Failed to open " << device.get_serial_number() << ": "
                  << result.error() << "\n";
        continue;
      } else {
        std::cout << "Opened " << device.get_serial_number() << "\n";
      }
      // Enable Buzzer
      if (auto result = device.io_buzzer_enable(true); !result.has_value()) {
        std::cerr << "Failed to enable buzzer on " << device.get_serial_number()
                  << ": " << result.error() << "\n";
        continue;
      } else {
        std::cout << "Enabled buzzer on " << device.get_serial_number() << "\n";
      }
      // Wait for buzzer to make some noise
      std::this_thread::sleep_for(std::chrono::milliseconds(50));
      if (auto result = device.io_buzzer_enable(false); !result.has_value()) {
        std::cerr << "Failed to disable buzzer on " << device.get_serial_number()
                  << ": " << result.error() << "\n";
        continue;
      } else {
        std::cout << "Disabled buzzer on " << device.get_serial_number() << "\n";
      }
      // Close IO
      if (auto result = device.io_close(); !result.has_value()) {
        std::cerr << "Failed to close " << device.get_serial_number() << ": "
                  << result.error() << "\n";
        continue;
      } else {
        std::cout << "Closed " << device.get_serial_number() << "\n";
      }
    }
  }

  return 0;
}

//   if ((err = mic2_gps_info(device, &info, sizeof(info))) !=
//       NeoVIMICErrTypeSuccess) {
//     mic2_gps_close(device);
//     return print_error(&err) == NeoVIMICErrTypeSuccess;
//   }
//   time_t current_time = info.current_time;
//   printf("Timestamp: %s\n", asctime(gmtime(&current_time)));
//   printf("Longitude: %d°%c %d' %d\"  (Valid: %d)\n", info.latitude.degrees,
//          info.latitude.minutes, info.latitude.seconds,
//          info.latitude_direction, info.latitude_valid);
//   printf("Longitude: %d°%c %d' %d\"  (Valid: %d)\n", info.longitude.degrees,
//          info.longitude.minutes, info.longitude.seconds,
//          info.longitude_direction, info.longitude_valid);
//   printf("Altitude: %f\n", info.altitude);
//   printf("NavStat: %d\n", info.nav_stat);
//   printf("h_acc: %f\n", info.h_acc);
//   printf("v_acc: %f\n", info.v_acc);
//   printf("sog_kmh: %f\n", info.sog_kmh);
//   printf("cog: %f\n", info.cog);
//   printf("vvel: %f\n", info.vvel);
//   printf("age_c: %f\n", info.age_c);
//   printf("hdop: %f\n", info.hdop);
//   printf("vdop: %f\n", info.vdop);
//   printf("tdop: %f\n", info.tdop);
//   printf("Satellite count: %d\n", info.satellites_count);
//   for (uint8_t i = 0; i < info.satellites_count; i++) {
//     printf("\t%d. Satellite PRN: %d: SNR: %d SNR valid: %d\n", i,
//            info.satellites[i].prn, info.satellites[i].snr,
//            info.satellites[i].snr_valid);
//   }
//   printf("Clock Bias: %f\n", info.clock_bias);
//   printf("Clock Drift: %f\n", info.clock_drift);
//   printf("Timepulse granularity: %f\n", info.timepulse_granularity);