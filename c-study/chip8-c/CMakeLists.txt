cmake_minimum_required(VERSION 3.28)
project(chip8 LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

include(FetchContent)
FetchContent_Declare(
    raylib
    GIT_REPOSITORY https://github.com/raysan5/raylib.git
    GIT_TAG 6.0
    GIT_SHALLOW ON
)
FetchContent_MakeAvailable(raylib)

add_executable(chip8 src/main.c)
target_link_libraries(chip8 PRIVATE raylib)
