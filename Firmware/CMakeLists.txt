cmake_minimum_required(VERSION 3.5)
# Set policy to allow subprojects with older cmake_minimum_required
set(CMAKE_POLICY_VERSION_MINIMUM 3.5 CACHE STRING "Minimum policy version" FORCE)
project(AtmosPipewirePanner VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Find PipeWire
find_package(PkgConfig REQUIRED)
pkg_check_modules(PIPEWIRE REQUIRED IMPORTED_TARGET libpipewire-0.3)

# FetchContent for dependencies
include(FetchContent)

# GLFW for window and input
FetchContent_Declare(
  glfw
  GIT_REPOSITORY https://github.com/glfw/glfw.git
  GIT_TAG        3.3.8
)

# Glad for OpenGL loading
FetchContent_Declare(
  glad
  GIT_REPOSITORY https://github.com/Dav1dde/glad.git
  GIT_TAG        v0.1.34
)

# Dear ImGui for immediate mode GUI
FetchContent_Declare(
  imgui
  GIT_REPOSITORY https://github.com/ocornut/imgui.git
  GIT_TAG        v1.89
)

# Make the dependencies available
FetchContent_MakeAvailable(glfw glad imgui)

# Now explicitly create the imgui library target since FetchContent doesn't do it for imgui
# Use the source and binary dirs set by FetchContent
# ImGui sources
set(IMGUI_SOURCES
    ${imgui_SOURCE_DIR}/imgui.cpp
    ${imgui_SOURCE_DIR}/imgui_draw.cpp
    ${imgui_SOURCE_DIR}/imgui_tables.cpp
    ${imgui_SOURCE_DIR}/imgui_widgets.cpp
)
# ImGui backends we need
set(IMGUI_BACKENDS
    ${imgui_SOURCE_DIR}/backends/imgui_impl_glfw.cpp
    ${imgui_SOURCE_DIR}/backends/imgui_impl_opengl3.cpp
)

# Create imgui library from source files
add_library(imgui
    ${IMGUI_SOURCES}
    ${IMGUI_BACKENDS}
)

# Include directories for imgui itself
target_include_directories(imgui
    PUBLIC
        ${imgui_SOURCE_DIR}
        ${imgui_SOURCE_DIR}/backends
)

# The imgui backends need GLFW headers, so link imgui to glfw (to get its include directories)
target_link_libraries(imgui glfw)

# Main executable
add_executable(atmos-panner
    main.cpp
    AudioEngine.cpp
    PipeWireOutput.cpp
    UserInterface.cpp
)

target_include_directories(atmos-panner PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

# Link PipeWire
target_link_libraries(atmos-panner PRIVATE PkgConfig::PIPEWIRE)

# Link our dependencies
target_link_libraries(atmos-panner PRIVATE
    glad
    imgui
    # Note: glfw is already linked via imgui, but we also need it for the main.cpp (for window creation)
    glfw
)

# Enable testing
include(CTest)
if(BUILD_TESTING)
    add_executable(audio-engine-test AudioEngine.cpp tests/AudioEngineTest.cpp)
    target_include_directories(audio-engine-test PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
    target_compile_options(audio-engine-test PRIVATE -Wall -Wextra -Wpedantic)
    add_test(NAME audio-engine-test COMMAND audio-engine-test)
endif()