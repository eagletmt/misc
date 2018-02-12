project(pdf-unlock)
cmake_minimum_required(VERSION 2.8)

include(FindPkgConfig)
pkg_check_modules(PDF_UNLOCK REQUIRED poppler poppler-glib cairo gio-2.0)
include_directories(${PDF_UNLOCK_INCLUDE_DIRS})
link_directories(${PDF_UNLOCK_LIBRARY_DIRS})
link_libraries(${PDF_UNLOCK_LIBRARIES})

add_definitions(-std=c++11 -Wall -W)
set(CMAKE_CXX_FLAGS_RELEASE "-O3")
set(CMAKE_CXX_FLAGS_DEBUG "-Og -g")

add_executable(pdf-unlock pdf-unlock.cc)

install(PROGRAMS pdf-unlock DESTINATION bin)
