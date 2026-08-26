#pragma once

#include <array>
#include <cstddef>
#include <mutex>
#include <string_view>
#include <vector>

namespace spatial {

inline constexpr std::size_t kOutputChannels = 17; // 9 bed + 2 LFE + 6 height

enum class Channel : std::size_t {
    FrontLeft, FrontCenter, FrontRight, WideLeft, WideRight,
    SideLeft, SideRight, BackLeft, BackRight, Lfe1, Lfe2,
    TopFrontLeft, TopFrontRight, TopMiddleLeft, TopMiddleRight,
    TopBackLeft, TopBackRight
};

struct Position { float azimuthDegrees = 0.0F; float elevationDegrees = 0.0F; float distance = 1.0F; };
struct RoomCorrection { float gainDb = 0.0F; float lowShelfDb = 0.0F; float highShelfDb = 0.0F; bool bypass = false; };

class AudioEngine {
public:
    AudioEngine();
    void setPosition(Position position);
    [[nodiscard]] Position position() const;
    void setSpread(float degrees);
    void setRoomCorrection(Channel channel, RoomCorrection correction);
    [[nodiscard]] std::array<float, kOutputChannels> gains() const;
    void renderMono(const float* input, std::size_t frames, float* const* output);
    [[nodiscard]] static std::string_view channelName(Channel channel);

private:
    struct Biquad { float b0=1, b1=0, b2=0, a1=0, a2=0, z1=0, z2=0; float process(float); };
    struct FilterChain { Biquad low, high; RoomCorrection settings; };
    void refreshGainsLocked();
    void refreshFiltersLocked();
    static Biquad shelf(float db, float frequency, bool high);

    mutable std::mutex mutex_;
    Position position_{};
    float spreadDegrees_ = 42.0F;
    std::array<float, kOutputChannels> gains_{};
    std::array<FilterChain, kOutputChannels> filters_{};
};

} // namespace spatial