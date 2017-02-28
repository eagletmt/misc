project(gnome-keyring-unlock)
cmake_minimum_required(VERSION 2.8)

include(FindPkgConfig)
pkg_check_modules(GNOME_KEYRING_UNLOCK REQUIRED gnome-keyring-1)
include_directories(${GNOME_KEYRING_UNLOCK_INCLUDE_DIRS})
link_directories(${GNOME_KEYRING_UNLOCK_LIBRARY_DIRS})
link_libraries(${GNOME_KEYRING_UNLOCK_LIBRARIES})

add_definitions(-std=c99 -Wall -W)
set(CMAKE_CXX_FLAGS_RELEASE "-O2")
set(CMAKE_CXX_FLAGS_DEBUG "-Og -g")

add_executable(gnome-keyring-unlock gnome-keyring-unlock.c)

install(PROGRAMS gnome-keyring-unlock DESTINATION bin)
