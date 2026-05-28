cmake_minimum_required(VERSION 3.20)
project(SubtrActorBakkesModPlugin LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

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

add_library(SubtrActorPlugin SHARED
  SubtrActorPluginAbiLayout.cpp
  SubtrActorPlugin.cpp
  SubtrActorPluginGraphDump.cpp
  SubtrActorPluginGraphJson.cpp
  SubtrActorPluginGraphVerify.cpp
  SubtrActorPluginLive.cpp
)

target_include_directories(SubtrActorPlugin PRIVATE
  "${BAKKESMOD_SDK_DIR}/include"
  "${CMAKE_CURRENT_LIST_DIR}/../crates/subtr-actor-bakkesmod/include"
)

target_link_directories(SubtrActorPlugin PRIVATE
  "${BAKKESMOD_SDK_DIR}/lib"
)

target_link_libraries(SubtrActorPlugin PRIVATE pluginsdk.lib)

target_compile_definitions(SubtrActorPlugin PRIVATE
  WIN32_LEAN_AND_MEAN
  NOMINMAX
)
