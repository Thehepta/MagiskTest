#!/bin/bash

echo "start build Fuseisk"


cmake -DCMAKE_BUILD_TYPE=Debug -DCMAKE_MAKE_PROGRAM=ninja -DCMAKE_TOOLCHAIN_FILE=~/NDK/build/cmake/android.toolchain.cmake -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=23 -G Ninja -S  sepol -B sepol/cmake-build-arm64debug
cmake --build sepol/cmake-build-arm64debug --target sepolicy
cmake --build sepol/cmake-build-arm64debug --target preload

cp sepol/cmake-build-arm64debug/libsepolicy.so ./
cp sepol/cmake-build-arm64debug/libpreload.so  ./

RUSTFLAGS='-C target-feature=+crt-static' cargo ndk  --platform 30 --target aarch64-linux-android  build --release
cp target/aarch64-linux-android/release/MinMagisk ./Fuseisk


magiskboot unpack redmi_boot.img

magiskboot cpio  ramdisk.cpio  "extract init  init_back"

magiskboot cpio ramdisk.cpio \
"add 0750 init Fuseisk" \
"add 0750 init_back init_back" \
"add 0750 libpreload.so libpreload.so" \
"add 0750 libsepolicy.so libsepolicy.so" \
"add 0750 su su" \
"add 0750 init.sh init.sh" \

magiskboot repack redmi_boot.img