cmake_minimum_required(VERSION 3.20)
project(StateExportBakkesModPlugin VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Build identification: overridable via cache or environment (the nix build
# exports these, since the sandbox has no .git), otherwise derived from git
# at configure time.
set(STATE_EXPORT_GIT_HASH "unknown" CACHE STRING "Git commit hash for this build")
set(STATE_EXPORT_GIT_DIRTY "0" CACHE STRING "Whether the source tree was dirty at build time")
set(STATE_EXPORT_COMMIT_DATE "unknown" CACHE STRING "Commit date for this build")

if(STATE_EXPORT_GIT_HASH STREQUAL "unknown" AND DEFINED ENV{STATE_EXPORT_GIT_HASH})
  set(STATE_EXPORT_GIT_HASH "$ENV{STATE_EXPORT_GIT_HASH}" CACHE STRING "Git commit hash for this build" FORCE)
endif()
if(STATE_EXPORT_GIT_DIRTY STREQUAL "0" AND DEFINED ENV{STATE_EXPORT_GIT_DIRTY})
  set(STATE_EXPORT_GIT_DIRTY "$ENV{STATE_EXPORT_GIT_DIRTY}" CACHE STRING "Whether the source tree was dirty at build time" FORCE)
endif()
if(STATE_EXPORT_COMMIT_DATE STREQUAL "unknown" AND DEFINED ENV{STATE_EXPORT_COMMIT_DATE})
  set(STATE_EXPORT_COMMIT_DATE "$ENV{STATE_EXPORT_COMMIT_DATE}" CACHE STRING "Commit date for this build" FORCE)
endif()

find_package(Git QUIET)
if(GIT_FOUND)
  if(STATE_EXPORT_GIT_HASH STREQUAL "unknown")
    execute_process(
      COMMAND "${GIT_EXECUTABLE}" rev-parse --short HEAD
      WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
      RESULT_VARIABLE STATE_EXPORT_GIT_HASH_RESULT
      OUTPUT_VARIABLE STATE_EXPORT_GIT_HASH_OUTPUT
      OUTPUT_STRIP_TRAILING_WHITESPACE
      ERROR_QUIET
    )
    if(STATE_EXPORT_GIT_HASH_RESULT EQUAL 0 AND NOT STATE_EXPORT_GIT_HASH_OUTPUT STREQUAL "")
      set(STATE_EXPORT_GIT_HASH "${STATE_EXPORT_GIT_HASH_OUTPUT}" CACHE STRING "Git commit hash for this build" FORCE)
    endif()
  endif()

  if(STATE_EXPORT_COMMIT_DATE STREQUAL "unknown")
    execute_process(
      COMMAND "${GIT_EXECUTABLE}" show -s --format=%cI HEAD
      WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
      RESULT_VARIABLE STATE_EXPORT_COMMIT_DATE_RESULT
      OUTPUT_VARIABLE STATE_EXPORT_COMMIT_DATE_OUTPUT
      OUTPUT_STRIP_TRAILING_WHITESPACE
      ERROR_QUIET
    )
    if(STATE_EXPORT_COMMIT_DATE_RESULT EQUAL 0 AND NOT STATE_EXPORT_COMMIT_DATE_OUTPUT STREQUAL "")
      set(STATE_EXPORT_COMMIT_DATE "${STATE_EXPORT_COMMIT_DATE_OUTPUT}" CACHE STRING "Commit date for this build" FORCE)
    endif()
  endif()

  if(STATE_EXPORT_GIT_DIRTY STREQUAL "0")
    execute_process(
      COMMAND "${GIT_EXECUTABLE}" diff --quiet --ignore-submodules HEAD --
      WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
      RESULT_VARIABLE STATE_EXPORT_GIT_DIRTY_RESULT
      ERROR_QUIET
    )
    if(STATE_EXPORT_GIT_DIRTY_RESULT EQUAL 1)
      set(STATE_EXPORT_GIT_DIRTY "1" CACHE STRING "Whether the source tree was dirty at build time" FORCE)
    endif()
  endif()
endif()

set(SUBTR_ACTOR_BAKKESMOD_SDK_TAG
  "479e8f571cf554b25f4eeb64d611dec4133edcaf"
  CACHE STRING
  "Pinned bakkesmodorg/BakkesModSDK revision")

if(NOT DEFINED BAKKESMOD_SDK_DIR AND DEFINED ENV{BAKKESMOD_SDK_DIR})
  set(BAKKESMOD_SDK_DIR "$ENV{BAKKESMOD_SDK_DIR}" CACHE PATH "BakkesMod SDK directory")
endif()

if(NOT DEFINED BAKKESMOD_SDK_DIR)
  include(FetchContent)
  FetchContent_Declare(
    bakkesmod_sdk
    URL "https://github.com/bakkesmodorg/BakkesModSDK/archive/${SUBTR_ACTOR_BAKKESMOD_SDK_TAG}.zip"
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
  )
  FetchContent_MakeAvailable(bakkesmod_sdk)
  set(BAKKESMOD_SDK_DIR "${bakkesmod_sdk_SOURCE_DIR}" CACHE PATH "BakkesMod SDK directory" FORCE)
endif()

if(NOT EXISTS "${BAKKESMOD_SDK_DIR}/include" OR NOT EXISTS "${BAKKESMOD_SDK_DIR}/lib/pluginsdk.lib")
  message(FATAL_ERROR "BakkesMod SDK is incomplete at ${BAKKESMOD_SDK_DIR}")
endif()

message(STATUS "BakkesMod SDK: ${BAKKESMOD_SDK_DIR}")

set(BAKKESMOD_IMGUI_INCLUDE_DIR "")
set(BAKKESMOD_IMGUI_ROOT "")
if(EXISTS "${BAKKESMOD_SDK_DIR}/imgui/imgui.h")
  set(BAKKESMOD_IMGUI_INCLUDE_DIR "${BAKKESMOD_SDK_DIR}")
  set(BAKKESMOD_IMGUI_ROOT "${BAKKESMOD_SDK_DIR}")
elseif(EXISTS "${BAKKESMOD_SDK_DIR}/imgui_bm.zip")
  set(BAKKESMOD_IMGUI_EXTRACT_DIR "${CMAKE_CURRENT_BINARY_DIR}/bakkesmod-imgui")
  file(MAKE_DIRECTORY "${BAKKESMOD_IMGUI_EXTRACT_DIR}")
  execute_process(
    COMMAND "${CMAKE_COMMAND}" -E tar xf "${BAKKESMOD_SDK_DIR}/imgui_bm.zip"
    WORKING_DIRECTORY "${BAKKESMOD_IMGUI_EXTRACT_DIR}"
    RESULT_VARIABLE BAKKESMOD_IMGUI_EXTRACT_RESULT
  )
  if(NOT BAKKESMOD_IMGUI_EXTRACT_RESULT EQUAL 0)
    message(FATAL_ERROR "Failed to extract BakkesMod ImGui headers from ${BAKKESMOD_SDK_DIR}/imgui_bm.zip")
  endif()
  set(BAKKESMOD_IMGUI_INCLUDE_DIR "${BAKKESMOD_IMGUI_EXTRACT_DIR}")
  set(BAKKESMOD_IMGUI_ROOT "${BAKKESMOD_IMGUI_EXTRACT_DIR}")
endif()

if(NOT BAKKESMOD_IMGUI_INCLUDE_DIR)
  message(FATAL_ERROR "BakkesMod ImGui headers were not found in ${BAKKESMOD_SDK_DIR}")
endif()

# Unity build: StateExportPlugin.cpp #includes every other plugin .cpp, so
# only it is compiled (mirroring the SubtrActorPlugin layout).
add_library(StateExportPlugin SHARED
  plugin/StateExportPlugin.cpp
  "${BAKKESMOD_IMGUI_ROOT}/imgui/imgui.cpp"
  "${BAKKESMOD_IMGUI_ROOT}/imgui/imgui_draw.cpp"
  "${BAKKESMOD_IMGUI_ROOT}/imgui/imgui_widgets.cpp"
)

target_include_directories(StateExportPlugin PRIVATE
  "${BAKKESMOD_SDK_DIR}/include"
  "${BAKKESMOD_IMGUI_INCLUDE_DIR}"
  "${CMAKE_CURRENT_LIST_DIR}/rust/include"
)

target_link_directories(StateExportPlugin PRIVATE
  "${BAKKESMOD_SDK_DIR}/lib"
)

target_link_libraries(StateExportPlugin PRIVATE pluginsdk.lib)

target_compile_definitions(StateExportPlugin PRIVATE
  WIN32_LEAN_AND_MEAN
  NOMINMAX
  STATE_EXPORT_PLUGIN_VERSION="${PROJECT_VERSION}"
  STATE_EXPORT_GIT_HASH="${STATE_EXPORT_GIT_HASH}"
  STATE_EXPORT_GIT_DIRTY=${STATE_EXPORT_GIT_DIRTY}
  STATE_EXPORT_COMMIT_DATE="${STATE_EXPORT_COMMIT_DATE}"
)
