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
    -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_audio_save(&device, path.c_str());
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::audio_start(uint32_t sample_rate) const
    -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_audio_start(&device, sample_rate);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::audio_stop() const -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_audio_stop(&device);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::gps_close() const -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_gps_close(&device);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::gps_has_lock() const -> std::expected<bool, NeoVIMICErrType> {
  bool gps_has_lock = false;
  NeoVIMICErrType err = mic2_gps_has_lock(&device, &gps_has_lock);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return gps_has_lock;
  }
}
auto CNeoVIMIC::gps_info() const -> std::expected<CGPSInfo, NeoVIMICErrType> {
  CGPSInfo info = {};
  NeoVIMICErrType err = mic2_gps_info(&device, &info, sizeof(info));
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return info;
  }
}
auto CNeoVIMIC::gps_is_open() const -> std::expected<bool, NeoVIMICErrType> {
  bool gps_is_open = false;
  NeoVIMICErrType err = mic2_gps_is_open(&device, &gps_is_open);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return gps_is_open;
  }
}
auto CNeoVIMIC::gps_open() const -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_gps_open(&device);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::io_button_is_pressed() const
    -> std::expected<bool, NeoVIMICErrType> {
  bool io_button_is_pressed = false;
  NeoVIMICErrType err = mic2_io_button_is_pressed(&device, &io_button_is_pressed);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return io_button_is_pressed;
  }
}
auto CNeoVIMIC::io_buzzer_enable(bool enable) const
    -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_io_buzzer_enable(&device, enable);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::io_buzzer_is_enabled() const
    -> std::expected<bool, NeoVIMICErrType> {
  bool buzzer_is_enabled = false;
  NeoVIMICErrType err = mic2_io_buzzer_is_enabled(&device, &buzzer_is_enabled);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return buzzer_is_enabled;
  }
}
auto CNeoVIMIC::io_close() const -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_io_close(&device);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::io_gpsled_enable(bool enable) const
    -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_io_gpsled_enable(&device, enable);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
}
auto CNeoVIMIC::io_gpsled_is_enabled() const
    -> std::expected<bool, NeoVIMICErrType> {
  bool gpsled_is_enabled = false;
  NeoVIMICErrType err = mic2_io_gpsled_is_enabled(&device, &gpsled_is_enabled);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return gpsled_is_enabled;
  }
}
auto CNeoVIMIC::io_is_open() const -> std::expected<bool, NeoVIMICErrType> {
  bool io_is_open = false;
  NeoVIMICErrType err = mic2_io_is_open(&device, &io_is_open);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return io_is_open;
  }
}
auto CNeoVIMIC::io_open() const -> std::expected<void, NeoVIMICErrType> {
  NeoVIMICErrType err = mic2_io_open(&device);
  if (err != NeoVIMICErrTypeSuccess) {
    return std::unexpected(err);
  } else {
    return {};
  }
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
