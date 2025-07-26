#!/bin/bash

echo "start build Fuseisk"


#cargo ndk  --platform 30 --target x86_64-linux-android  build  --release
#cp target/x86_64-linux-android/release/Fuseisk ./

cargo ndk  --platform 30 --target x86_64-linux-android  build
cp target/x86_64-linux-android/debug/Fuseisk ./


magiskboot unpack init_boot.img

magiskboot cpio  ramdisk.cpio  "extract init  init_back"

magiskboot cpio ramdisk.cpio \
"add 0750 init Fuseisk" \
"add 0750 init_back init_back" \

magiskboot repack init_boot.img