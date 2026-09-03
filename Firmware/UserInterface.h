#pragma once
#include "AudioEngine.h"
#include <string>
#include <vector>
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

    // Tab state
    int currentTab_ = 0; // 0 = Atmos Panner, 1 = I/O Selection

    // I/O Selection state
    struct NodeInfo {
        int id;
        std::string name;
    };
    std::vector<NodeInfo> inputNodes_;
    std::vector<NodeInfo> outputNodes_;
    int selectedInputNode_ = -1;
    int selectedOutputNode_ = -1;

    // Helper methods
    void handleInput(ImGuiIO& io, GLFWwindow* window);
    void render3D();
    void renderControls();
    void renderTabs();
    void renderAtmosPannerTab();
    void renderIoSelectionTab();
    void refreshNodeLists();
    void applyIoSelection();
};

} // namespace spatial