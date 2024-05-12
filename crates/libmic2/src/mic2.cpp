#include "mic2.hpp"

#include <cstdint>
#include <string>
#include <expected>
#include <vector>

using namespace mic2;

CNeoVIMIC::CNeoVIMIC(const NeoVIMIC &device) : device(device) {}
CNeoVIMIC::~CNeoVIMIC() {
  if (io_is_open()) {
    io_close();
  }
  if (gps_is_open()) {
    gps_close();
  }
}

auto CNeoVIMIC::has_gps() const -> std::expected<bool, NeoVIMICErrType> {
  bool has_gps = false;
  NeoVIMICErrType err = mic2_has_gps(&device, &has_gps);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return has_gps;
  }
}

auto CNeoVIMIC::get_serial_number() const -> std::string {
  return device.serial_number;
}

auto CNeoVIMIC::audio_save(std::string path) const
    -> std::expected<bool, NeoVIMICErrType> {
      (void)path;
  return false;
}
auto CNeoVIMIC::audio_start(uint32_t sample_rate) const
    -> std::expected<bool, NeoVIMICErrType> {
      (void)sample_rate;
  return false;
}
auto CNeoVIMIC::audio_stop() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::gps_close() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::gps_has_lock() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::gps_info() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::gps_is_open() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::gps_open() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_button_is_pressed() const
    -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_buzzer_enable() const
    -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_buzzer_is_enabled() const
    -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_close() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_gpsled_enable() const
    -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_gpsled_is_enabled() const
    -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_is_open() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}
auto CNeoVIMIC::io_open() const -> std::expected<bool, NeoVIMICErrType> {
  return false;
}

auto mic2::find() -> std::expected<std::vector<CNeoVIMIC>, NeoVIMICErrType> {
  constexpr size_t DEVICE_COUNT = 10;
  NeoVIMIC dev_buffer[DEVICE_COUNT] = {};
  uint32_t length = (uint32_t)DEVICE_COUNT;
  NeoVIMICErrType err = NeoVIMICErrTypeFailure;
  if ((err = mic2_find(dev_buffer, &length, MIC2_API_VERSION, sizeof(NeoVIMIC))) !=
      NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  }
  std::vector<CNeoVIMIC> devices;
  for (uint32_t i=0; i<length; i++) {
    devices.push_back(CNeoVIMIC(dev_buffer[i]));
  }
  return devices;
}
