#!/bin/bash


cmake -DCMAKE_BUILD_TYPE=Debug -DCMAKE_MAKE_PROGRAM=ninja -DCMAKE_TOOLCHAIN_FILE=~/NDK/build/cmake/android.toolchain.cmake -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=23 -G Ninja -S  sepol -B sepol/cmake-build-arm64debug
cmake --build sepol/cmake-build-arm64debug --target sepolicy
cmake --build sepol/cmake-build-arm64debug --target preload

cmake -DCMAKE_BUILD_TYPE=Debug -DCMAKE_MAKE_PROGRAM=ninja -DCMAKE_TOOLCHAIN_FILE=~/NDK/build/cmake/android.toolchain.cmake -DANDROID_ABI=x86_64 -DANDROID_PLATFORM=23 -G Ninja -S  sepol -B sepol/cmake-build-x64debug
cmake --build sepol/cmake-build-x64debug --target sepolicy
cmake --build sepol/cmake-build-x64debug --target preload