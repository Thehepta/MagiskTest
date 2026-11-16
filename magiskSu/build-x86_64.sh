#!/bin/bash



cmake -DCMAKE_BUILD_TYPE=Debug -DCMAKE_MAKE_PROGRAM=ninja -G Ninja -S . -B cmake-build-x86_64-debug  -DCMAKE_TOOLCHAIN_FILE=~/NDK/build/cmake/android.toolchain.cmake -DANDROID_ABI=x86_64 -DANDROID_PLATFORM=23
cmake --build  cmake-build-x86_64-debug  --target su
