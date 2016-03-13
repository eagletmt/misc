project(fc-find)
cmake_minimum_required(VERSION 2.8)

include(FindPkgConfig)
pkg_check_modules(FONTCONFIG REQUIRED fontconfig)
include_directories(${FONTCONFIG_INCLUDE_DIRS})
link_directories(${FONTCONFIG_LIBRARY_DIRS})
link_libraries(${FONTCONFIG_LIBRARIES})

add_definitions(-std=c++11 -Wall -W)
set(CMAKE_CXX_FLAGS_RELEASE "-O3")
set(CMAKE_CXX_FLAGS_DEBUG "-Og -g")

add_executable(fc-find fc-find.cc)

install(PROGRAMS fc-find DESTINATION bin)
