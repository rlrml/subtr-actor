set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_PROCESSOR x86_64)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
set(CMAKE_TRY_COMPILE_PLATFORM_VARIABLES
  XWIN_SYSROOT
  XWIN_VC_LIB_DIR
  XWIN_UCRT_LIB_DIR
  XWIN_UM_LIB_DIR
)

if(NOT DEFINED XWIN_SYSROOT AND DEFINED ENV{XWIN_SYSROOT})
  set(XWIN_SYSROOT "$ENV{XWIN_SYSROOT}" CACHE PATH "xwin MSVC/Windows SDK sysroot")
endif()

if(NOT DEFINED XWIN_SYSROOT)
  message(FATAL_ERROR "Set XWIN_SYSROOT to an xwin --use-winsysroot-style splat directory")
endif()

if(NOT EXISTS "${XWIN_SYSROOT}/VC/Tools/MSVC" OR NOT EXISTS "${XWIN_SYSROOT}/Windows Kits/10")
  message(FATAL_ERROR "XWIN_SYSROOT is not an xwin MSVC sysroot: ${XWIN_SYSROOT}")
endif()

if(NOT DEFINED XWIN_VC_LIB_DIR)
  file(GLOB _xwin_vc_tools_dirs LIST_DIRECTORIES TRUE "${XWIN_SYSROOT}/VC/Tools/MSVC/*")
  if(NOT _xwin_vc_tools_dirs)
    message(FATAL_ERROR "XWIN_SYSROOT is missing MSVC tool versions: ${XWIN_SYSROOT}")
  endif()
  list(SORT _xwin_vc_tools_dirs COMPARE NATURAL)
  list(GET _xwin_vc_tools_dirs -1 _xwin_vc_tools_dir)
  set(XWIN_VC_LIB_DIR "${_xwin_vc_tools_dir}/lib/x64")
endif()
if(NOT DEFINED XWIN_UCRT_LIB_DIR OR NOT DEFINED XWIN_UM_LIB_DIR)
  file(GLOB _xwin_windows_sdk_lib_dirs LIST_DIRECTORIES TRUE "${XWIN_SYSROOT}/Windows Kits/10/Lib/*")
  if(NOT _xwin_windows_sdk_lib_dirs)
    message(FATAL_ERROR "XWIN_SYSROOT is missing Windows SDK library versions: ${XWIN_SYSROOT}")
  endif()
  list(SORT _xwin_windows_sdk_lib_dirs COMPARE NATURAL)
  list(GET _xwin_windows_sdk_lib_dirs -1 _xwin_windows_sdk_lib_dir)
  set(XWIN_UCRT_LIB_DIR "${_xwin_windows_sdk_lib_dir}/ucrt/x64")
  set(XWIN_UM_LIB_DIR "${_xwin_windows_sdk_lib_dir}/um/x64")
endif()

if(NOT EXISTS "${XWIN_VC_LIB_DIR}" OR NOT EXISTS "${XWIN_UCRT_LIB_DIR}" OR NOT EXISTS "${XWIN_UM_LIB_DIR}")
  message(FATAL_ERROR "XWIN_SYSROOT is missing expected MSVC/SDK x64 library directories")
endif()

find_program(CLANG_CL clang-cl REQUIRED)
find_program(LLD_LINK lld-link REQUIRED)
find_program(LLVM_LIB llvm-lib REQUIRED)
find_program(LLVM_MT llvm-mt)
find_program(LLVM_RC llvm-rc)

set(CMAKE_CXX_COMPILER "${CLANG_CL}" CACHE FILEPATH "clang-cl")
set(CMAKE_LINKER "${LLD_LINK}" CACHE FILEPATH "lld-link")
set(CMAKE_AR "${LLVM_LIB}" CACHE FILEPATH "llvm-lib")
set(CMAKE_RANLIB ":" CACHE FILEPATH "no ranlib for MSVC archives")
if(LLVM_MT)
  set(CMAKE_MT "${LLVM_MT}" CACHE FILEPATH "llvm-mt")
endif()
if(LLVM_RC)
  set(CMAKE_RC_COMPILER "${LLVM_RC}" CACHE FILEPATH "llvm-rc")
endif()
set(CMAKE_CXX_COMPILER_TARGET x86_64-pc-windows-msvc)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")

set(CMAKE_CXX_FLAGS_INIT "--target=x86_64-pc-windows-msvc -fuse-ld=lld /winsysroot \"${XWIN_SYSROOT}\"")

set(_xwin_link_flags
  "/NOLOGO"
  "/LIBPATH:\"${XWIN_VC_LIB_DIR}\""
  "/LIBPATH:\"${XWIN_UCRT_LIB_DIR}\""
  "/LIBPATH:\"${XWIN_UM_LIB_DIR}\""
)
string(REPLACE ";" " " _xwin_link_flags "${_xwin_link_flags}")
set(CMAKE_EXE_LINKER_FLAGS_INIT "${_xwin_link_flags}")
set(CMAKE_MODULE_LINKER_FLAGS_INIT "${_xwin_link_flags}")
set(CMAKE_SHARED_LINKER_FLAGS_INIT "${_xwin_link_flags}")
set(CMAKE_EXE_LINKER_FLAGS "${_xwin_link_flags}" CACHE STRING "xwin MSVC linker flags" FORCE)
set(CMAKE_MODULE_LINKER_FLAGS "${_xwin_link_flags}" CACHE STRING "xwin MSVC linker flags" FORCE)
set(CMAKE_SHARED_LINKER_FLAGS "${_xwin_link_flags}" CACHE STRING "xwin MSVC linker flags" FORCE)
