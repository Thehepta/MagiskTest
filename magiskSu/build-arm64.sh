#!/bin/bash


cmake -DCMAKE_BUILD_TYPE=Debug -DCMAKE_MAKE_PROGRAM=ninja -G Ninja -S . -B cmake-build-arm64-debug  -DCMAKE_TOOLCHAIN_FILE=~/NDK/build/cmake/android.toolchain.cmake -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=23
cmake --build  cmake-build-arm64-debug --target su
