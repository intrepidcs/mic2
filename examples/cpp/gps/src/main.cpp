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
  if (devices.value().size() < 1) {
    std::cerr << "Need at least one device to continue!\n";
    return 1;
  }
  auto& device = devices.value().at(0);
  // Open GPS
  if (auto result = device.gps_open(); !result.has_value()) {
    std::cerr << "Failed to open " << device.get_serial_number() << ": "
              << result.error() << "\n";
  } else {
    std::cout << "Opened " << device.get_serial_number() << "\n";
  }

  /*
  bool gps_has_lock = false;
  while (!gps_has_lock) {
    std::cout << "Waiting for GPS lock...\n";
    std::this_thread::sleep_for(std::chrono::milliseconds(500));
    if (auto result = device.gps_has_lock(); !result.has_value()) {
      std::cerr << "Failed to see if gps has lock(): " << result.error() << "\n";
      continue;
    } else {
      gps_has_lock = result.value();
    }
  };
  if (!gps_has_lock) {
    std::cerr << "Failed to get lock\n";
    return 1;
  }
  */
  while (true) {
    std::this_thread::sleep_for(std::chrono::milliseconds(500));
    if (auto result = device.gps_info(); !result.has_value()) {
      std::cerr << "Failed to get GPS info: " << result.error() << "\n";
      continue;
    } else {
      auto& info = result.value();
      time_t current_time = info.current_time;
      printf("Timestamp: %s\n", asctime(gmtime(&current_time)));
      printf("Longitude: %d°%c %d' %d\"  (Valid: %d)\n", info.latitude.degrees,
            info.latitude.minutes, info.latitude.seconds,
            info.latitude_direction, info.latitude_valid);
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
      printf("\n\n");
    }
  }


  return 0;
}
