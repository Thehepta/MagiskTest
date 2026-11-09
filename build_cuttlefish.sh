#!/bin/bash

echo "start build Fuseisk"

cargo ndk  --platform 30 --target x86_64-linux-android  build --release
cp target/x86_64-linux-android/release/MinMagisk ./Fuseisk

#cargo ndk  --platform 30 --target x86_64-linux-android  build
#cp target/x86_64-linux-android/debug/MinMagisk ./


magiskboot unpack init_boot.img

magiskboot cpio  ramdisk.cpio  "extract init  init_back"

magiskboot cpio ramdisk.cpio \
"add 0750 init Fuseisk" \
"add 0750 init_back init_back" \
"add 0750 libpreload.so libpreload.so" \
"add 0750 libsepolicy.so libsepolicy.so" \
"add 0750 su su" \
"add 0750 init.sh init.sh" \
"add 0750 magisk magisk" \

magiskboot repack init_boot.img

cp new-boot.img /media/chic/nvme0n1p31/cuttelefish/new-boot.img