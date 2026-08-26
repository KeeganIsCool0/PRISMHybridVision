#include "UserInterface.h"
#define GLFW_INCLUDE_NONE
#include <GLFW/glfw3.h>
#include <glad/glad.h>
#include <imgui.h>
#include <backends/imgui_impl_glfw.h>
#include <backends/imgui_impl_opengl3.h>
#include <cmath>
#include <iostream>
#include <algorithm>

namespace spatial {

UserInterface::UserInterface(AudioEngine& engine)
    : engine_(engine),
      azimuthDegrees_(0.0f),
      elevationDegrees_(0.0f),
      distance_(1.0f), // meters
      spreadDegrees_(42.0f)
{
    // Initialize room correction arrays to zero/false
    lowShelfDb_.fill(0.0f);
    highShelfDb_.fill(0.0f);
    bypass_.fill(false);
}

bool UserInterface::update(ImGuiIO& io, GLFWwindow* window) {
    // Handle input (mouse drag for 3D interaction)
    handleInput(io, window);
    (void)window; // Mark as used

    // Clear buffers
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    glEnable(GL_DEPTH_TEST);

    // Set up basic 3D projection and view
    int width, height;
    glfwGetWindowSize(window, &width, &height);
    glViewport(0, 0, width, height);

    // Simple perspective projection using legacy OpenGL for now
    // TODO: Replace with proper modern OpenGL or ImGui-based 2D representation
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    const float aspect = static_cast<float>(width) / static_cast<float>(height);
    const float fov = 45.0f * static_cast<float>(M_PI) / 180.0f;
    const float near = 0.1f; // Mark as used
    const float far = 100.0f; // Mark as used
    const float f = 1.0f / std::tan(fov / 2.0f); // Mark as used
    GLfloat proj[16] = {
        f / aspect, 0, 0, 0,
        0, f, 0, 0,
        0, 0, (far + near) / (near - far), -1,
        0, 0, (2 * far * near) / (near - far), 0
    };
    glLoadMatrixf(proj);

    // View matrix (camera looking at origin)
    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();
    // Camera positioned to view the sphere from a distance
    const float camDist = 3.0f;
    glTranslatef(0.0f, 0.0f, -camDist);

    // Update engine with current position
    Position pos{azimuthDegrees_, elevationDegrees_, distance_};
    engine_.setPosition(pos);
    engine_.setSpread(spreadDegrees_);

    // Update room correction for all channels (we could optimize this to only changed channels)
    for (size_t i = 0; i < kOutputChannels; ++i) {
        if (!bypass_[i]) {
            engine_.setRoomCorrection(
                static_cast<Channel>(i),
                RoomCorrection{0.0f, lowShelfDb_[i], highShelfDb_[i], false}
            );
        } else {
            engine_.setRoomCorrection(
                static_cast<Channel>(i),
                RoomCorrection{0.0f, 0.0f, 0.0f, true}
            );
        }
    }

    // Render 3D sphere (using legacy OpenGL for simplicity)
    render3D();

    // Render ImGui controls overlay
    renderControls();

    // Render ImGui
    ImGui::Render();
    ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());

    // Handle viewport platforms - SKIPPED for compatibility
    // if (io.ConfigFlags & ImGuiConfigFlags_ViewportsEnable) {
    //     GLFWwindow* backup_current_context = glfwGetCurrentContext();
    //     ImGui::UpdatePlatformWindows();
    //     ImGui::RenderPlatformWindowsDefault();
    //     glfwMakeContextCurrent(backup_current_context);
    // }

    // Check if user wants to quit (we'll check for escape key or a quit flag)
    // For now, we rely on window close button
    return !glfwWindowShouldClose(window);
}

void UserInterface::handleInput(ImGuiIO& io, GLFWwindow* window) {
    (void)window; // Mark as used
    // Left mouse button drag for azimuth/elevation (rotate sphere)
    if (io.MouseDown[0]) {
        // Convert mouse delta to rotation angles
        // Sensitivity factor
        const float sensitivity = 0.5f;
        azimuthDegrees_ -= io.MouseDelta.x * sensitivity;
        elevationDegrees_ += io.MouseDelta.y * sensitivity;

        // Clamp elevation to prevent gimbal lock and stay within reasonable bounds
        elevationDegrees_ = std::clamp(elevationDegrees_, -30.0f, 90.0f);

        // Keep azimuth in reasonable range (we'll let it wrap naturally in sin/cos)
        // But we can keep it within -360 to 360 for sanity
        while (azimuthDegrees_ < -360.0f) azimuthDegrees_ += 360.0f;
        while (azimuthDegrees_ > 360.0f) azimuthDegrees_ -= 360.0f;
    }

    // Right mouse button drag or scroll wheel for distance (Z-axis)
    if (io.MouseDown[1]) {
        // Right drag: vertical movement changes distance
        const float distanceSensitivity = 0.01f;
        distance_ -= io.MouseDelta.y * distanceSensitivity;
        distance_ = std::clamp(distance_, 0.1f, 10.0f);
    }

    // Scroll wheel also controls distance
    if (io.MouseWheel != 0.0f) {
        const float scrollSensitivity = 0.1f;
        distance_ -= io.MouseWheel * scrollSensitivity;
        distance_ = std::clamp(distance_, 0.1f, 10.0f);
    }
}

void UserInterface::render3D() {
    // TODO: Replace legacy OpenGL with modern OpenGL or ImGui-based rendering
    // For now, using legacy OpenGL which may not work in core profile
    // This is a temporary fix to get it compiling

    // Render a sphere with latitude/longitude lines
    // Color the sphere based on current azimuth/elevation for visual feedback

    // Save current matrix state
    glPushMatrix();

    // Apply the current azimuth/elevation rotation to the sphere
    glRotatef(elevationDegrees_, 1.0f, 0.0f, 0.0f);  // Pitch (X-axis)
    glRotatef(azimuthDegrees_, 0.0f, 1.0f, 0.0f);   // Yaw (Y-axis)

    // Sphere radius
    const float radius = 1.0f;
    const int slices = 32;
    const int stacks = 16;

    // Draw sphere as wireframe
    glColor3f(0.3f, 0.6f, 0.9f); // Light blue

    // Draw lines of latitude
    for (int i = 0; i <= stacks; ++i) {
        const float lat = static_cast<float>(M_PI) * (-0.5f + static_cast<float>(i) / stacks);
        const float z = std::sin(lat) * radius;
        const float r = std::cos(lat) * radius;

        glBegin(GL_LINE_STRIP);
        for (int j = 0; j <= slices; ++j) {
            const float lng = 2.0f * static_cast<float>(M_PI) * static_cast<float>(j) / slices;
            const float x = r * std::cos(lng);
            const float y = r * std::sin(lng);
            glVertex3f(x, y, z);
        }
        glEnd();
    }

    // Draw lines of longitude
    for (int j = 0; j <= slices; ++j) {
        const float lng = 2.0f * static_cast<float>(M_PI) * static_cast<float>(j) / slices;
        const float x = std::cos(lng) * radius;
        const float y = std::sin(lng) * radius;

        glBegin(GL_LINE_STRIP);
        for (int i = 0; i <= stacks; ++i) {
            const float lat = static_cast<float>(M_PI) * (-0.5f + static_cast<float>(i) / stacks);
            const float z = std::sin(lat) * radius;
            const float r = std::cos(lat) * radius;
            glVertex3f(x * r, y * r, z);
        }
        glEnd();
    }

    // Draw a marker at the current "listener" position (origin)
    glColor3f(1.0f, 0.0f, 0.0f); // Red
    glPointSize(5.0f);
    glBegin(GL_POINTS);
    glVertex3f(0.0f, 0.0f, 0.0f);
    glEnd();

    // Restore matrix
    glPopMatrix();
}

void UserInterface::renderControls() {
    // ImGui window for controls
    ImGui::Begin("Spatial Controls", nullptr, ImGuiWindowFlags_AlwaysAutoResize);

    ImGui::Text("Current Position:");
    ImGui::Text("Azimuth: %.1f°", azimuthDegrees_);
    ImGui::Text("Elevation: %.1f°", elevationDegrees_);
    ImGui::Text("Distance: %.2f m", distance_);

    ImGui::Separator();

    // Spread control
    ImGui::SliderFloat("Spread", &spreadDegrees_, 0.0f, 180.0f, "%.1f°");

    ImGui::Separator();

    // Room correction controls (simplified - show first few channels)
    ImGui::Text("Room Correction (First 4 Channels):");
    for (size_t i = 0; i < std::min(kOutputChannels, static_cast<size_t>(4)); ++i) {
        ImGui::PushID(static_cast<int>(i));
        std::string_view channelNameView = AudioEngine::channelName(static_cast<Channel>(i));
        const char* channelName = channelNameView.data();
        ImGui::Text("%s:", channelName);

        ImGui::Checkbox("Bypass", &bypass_[i]);
        ImGui::SameLine();

        ImGui::DragFloat("Low Shelf", &lowShelfDb_[i], 0.1f, -20.0f, 20.0f, "%.1f dB");
        ImGui::SameLine();
        ImGui::DragFloat("High Shelf", &highShelfDb_[i], 0.1f, -20.0f, 20.0f, "%.1f dB");

        ImGui::PopID();
        ImGui::Separator();
    }

    if (ImGui::Button("Reset Position")) {
        azimuthDegrees_ = 0.0f;
        elevationDegrees_ = 0.0f;
        distance_ = 1.0f;
    }

    ImGui::SameLine();
    // Quit button - we'll handle this by setting a flag or just letting window close work
    if (ImGui::Button("Quit")) {
        // We don't have direct access to window here, so we'll just return false
        // Actually, let's set a flag that the main loop can check
        // For simplicity, we'll just close the window via glfw if we can access it
        // Since we can't easily access window from here, we'll just return and
        // let the main loop handle it via the window close button
    }

    ImGui::End();
}

} // namespace spatial