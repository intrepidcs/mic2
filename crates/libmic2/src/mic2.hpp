#pragma once

#include <cstdint>
#include <string>
#include <expected>
#include <vector>

#include <mic2.h>

namespace mic2 {

class CNeoVIMIC {
public:
  CNeoVIMIC(const NeoVIMIC &device);
  ~CNeoVIMIC();

  auto has_gps() const -> std::expected<bool, NeoVIMICErrType>;
  auto get_serial_number() const -> std::string;

  auto audio_save(std::string path) const -> std::expected<void, NeoVIMICErrType>;
  auto audio_start(uint32_t sample_rate) const -> std::expected<void, NeoVIMICErrType>;
  auto audio_stop() const -> std::expected<void, NeoVIMICErrType>;

  auto gps_close() const -> std::expected<void, NeoVIMICErrType>;
  auto gps_has_lock() const -> std::expected<bool, NeoVIMICErrType>;
  auto gps_info() const -> std::expected<CGPSInfo, NeoVIMICErrType>;
  auto gps_is_open() const -> std::expected<bool, NeoVIMICErrType>;
  auto gps_open() const -> std::expected<void, NeoVIMICErrType>;

  auto io_button_is_pressed() const -> std::expected<bool, NeoVIMICErrType>;
  auto io_buzzer_enable(bool enable) const -> std::expected<void, NeoVIMICErrType>;
  auto io_buzzer_is_enabled() const -> std::expected<bool, NeoVIMICErrType>;
  auto io_close() const -> std::expected<void, NeoVIMICErrType>;
  auto io_gpsled_enable(bool enable) const -> std::expected<void, NeoVIMICErrType>;
  auto io_gpsled_is_enabled() const -> std::expected<bool, NeoVIMICErrType>;
  auto io_is_open() const -> std::expected<bool, NeoVIMICErrType>;
  auto io_open() const -> std::expected<void, NeoVIMICErrType>;

  // std::variant<bool, NeoVIMICErrType> mic2_find() const;
  // std::variant<bool, NeoVIMICErrType> mic2_free() const;
  // std::variant<bool, NeoVIMICErrType> mic2_error_string() const;

private:
  const NeoVIMIC device;
};

auto find() -> std::expected<std::vector<CNeoVIMIC>, NeoVIMICErrType>;

}; // namespace mic2
