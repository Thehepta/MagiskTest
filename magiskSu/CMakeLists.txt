cmake_minimum_required(VERSION 3.10.2)
project(su)

set(CMAKE_CXX_STANDARD 14)

set(CMAKE_EXE_LINKER_FLAGS "-static")

add_executable(su main.cpp su_client.cpp su_service.cpp tool.h pts.cpp pts.h)

