#pragma once
#include "AudioEngine.h"
// Forward declarations for GLFW and ImGui types we use by reference/pointer
struct GLFWwindow;
struct ImGuiIO;

namespace spatial {

class UserInterface {
public:
    explicit UserInterface(AudioEngine& engine);
    bool update(ImGuiIO& io, GLFWwindow* window);

private:
    AudioEngine& engine_;
    // State for 3D interaction
    float azimuthDegrees_ = 0.0f;
    float elevationDegrees_ = 0.0f;
    float distance_ = 1.0f; // meters
    // State for controls
    float spreadDegrees_ = 42.0f;
    // Per-channel room correction state (mirroring what we send to engine)
    std::array<float, kOutputChannels> lowShelfDb_{};
    std::array<float, kOutputChannels> highShelfDb_{};
    std::array<bool, kOutputChannels> bypass_{};

    // Helper methods
    void handleInput(ImGuiIO& io, GLFWwindow* window);
    void render3D();
    void renderControls();
};

} // namespace spatial