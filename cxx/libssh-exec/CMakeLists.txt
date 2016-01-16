project(libssh-exec)
cmake_minimum_required(VERSION 2.8)

include(FindPkgConfig)
pkg_check_modules(LIBSSH_EXEC REQUIRED libssh)
include_directories(${LIBSSH_EXEC_INCLUDE_DIRS})
link_directories(${LIBSSH_EXEC_LIBRARY_DIRS})
link_libraries(${LIBSSH_EXEC_LIBRARIES})

add_definitions(-std=c++11 -Wall -W)
set(CMAKE_CXX_FLAGS_RELEASE "-O3")
set(CMAKE_CXX_FLAGS_DEBUG "-Og -g")

add_executable(libssh-exec libssh-exec.cc)

install(PROGRAMS libssh-exec DESTINATION bin)
