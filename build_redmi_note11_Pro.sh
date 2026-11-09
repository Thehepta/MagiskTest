#!/bin/bash

echo "start build Fuseisk"


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