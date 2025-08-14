


# 概述
这是一个 magisk init 项目，用于通过替换劫持boot.img 中的init,并且可以让系统正常启动。我曾经想把它写成一个maigks类似的项目，后续有时间在考虑把
这个项目目前只在 cuttlefish 上测试过了，理论上来说可以兼容大部分手机，但只是理论。

目前的工程量完成到，抄完了magisk 的几种启动方式，然后修补通过mount 修补init.rc文件并写入数据

# magisk Boot Methods

这是magisk 中的启动方式，这个工程就是对这个逻辑的实践
Android booting can be roughly categorized into 3 major different methods. We provide a general rule of thumb to determine which method your device is most likely using, with exceptions listed separately.

Method | Initial rootdir | Final rootdir
:---: | --- | ---
**A** | `rootfs` | `rootfs`
**B** | `system` | `system`
**C** | `rootfs` | `system`

- **Method A - Legacy ramdisk**: This is how *all* Android devices used to boot (good old days). The kernel uses `initramfs` as rootdir, and exec `/init` to boot.
    - Devices that does not fall in any of Method B and C's criteria
- **Method B - Legacy SAR**: This method was first seen on Pixel 1. The kernel directly mounts the `system` partition as rootdir and exec `/init` to boot.
    - Devices with `(LV = 28)`
    - Google: Pixel 1 and 2. Pixel 3 and 3a when `(RV = 28)`.
    - OnePlus: 6 - 7
    - Maybe some `(LV < 29)` Android Go devices?
- **Method C - 2SI ramdisk SAR**: This method was first seen on Pixel 3 Android 10 developer preview. The kernel uses `initramfs` as rootdir and exec `/init` in `rootfs`. This `init` is responsible to mount the `system` partition and use it as the new rootdir, then finally exec `/system/bin/init` to boot.
    - Devices with `(LV >= 29)`
    - Devices with `(LV < 28, RV >= 29)`, excluding those that were already using Method B
    - Google: Pixel 3 and 3a with `(RV >= 29)`


# magisk init底层程序开发和测试

## avd
avd模拟器启动
```
/home/chic/Android/Sdk/emulator/emulator -avd Pixel_8_API_34-ext12 -no-window   -no-audio   -no-boot-anim   -gpu swiftshader_indirect   -read-only   -no-snapshot   -port 5682   -cores 4   -memory 8192   -show-kernel   -logcat ''   -logcat-output logcat.log   -ramdisk ramdisk_magisk.img   -feature SystemAsRoot  > kernel.log
```
指定magisk修补的 ramdisk -ramdisk ramdisk_magisk.img

### 直接修补ramdisk

如何修改ramgisik(需要先启动模拟器，build.py脚本需要通过adb 获取架构)
```
./build.py avd_patch ramdisk.img ramdisk_magisk.img
```


### 直接修补init_boot.img

如何修改init_boot(需要先启动模拟器，build.py脚本需要通过adb 获取架构)
```
./build.py avd_patch init_boot.img init_boot_magisk.img
```


## cuttlefish
cuttlefish 启动，-init_boot_image 可以指定替换的 magisk修补镜像
```
HOME=$PWD ./bin/launch_cvd --daemon -init_boot_image=init_boot_magisk.img
``` 
使用cuttlefish需要去google 下载镜像和cuttlefish 。然后需要安装一个包，才能启动cuttlefish,否则会报错。这个包可以编译，也可以去下载。

## magisk修补init_boot.img 和ramdisk
目前的cuttlefish 使用的是init_boot.img， 因为我不是手机环境，同时我想为修补的镜像高一些自己的东西进去，所以没法使用build.py 自动修补，需要使用magiskboot工具来修补。
### ramdisk修补
```
#先解压目标文件
./magiskboot decompress "$TARGET_FILE" ramdisk.cpio

#然后往ramdisk.cpio 中添加文件，这个资料比较多

#最后在用magiskboot 将ramdisk.cpio 变成ramdisk.img
./magiskboot compress=gzip ramdisk.cpio "$OUTPUT_FILE"

```

### init_boot.img 修补
这个比较麻烦，因为magiskboot 提供的命令介绍没太看懂，研究了很久才搞明白。magiskboot  提供了将init_boot.img 解包的功能 ./magiskboot unpack "$TARGET_FILE"，但是没有将解压出来的文件再次处理成init_boot.img的功能，需要使用原来的init_boot.img 的包替换里面的文件，可能是同名文件，然后生产新包
```
# 解开init_boot.img,一般会有一个ramdisk.cpio
./magiskboot unpack "$TARGET_FILE"

#然后往ramdisk.cpio 中添加文件，这个资料比较多

# ramdisk.cpio不要改名字，使用原init_boot.img包执行命令，他会将你处理的ramdisk.cpio 替换init_boot.img 这个包里的文件，生产新的img
./magiskboot repack "$TARGET_FILE" "$OUTPUT_FILE"

```