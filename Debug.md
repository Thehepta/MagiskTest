## 真机日志查看
这个可以使用
adb shell cat /sys/fs/pstore/console-ramoops-0 > last_init_crash.log
这个我手机上没有
adb shell cat /sys/fs/pstore/pmsg-ramoops-0 > last_init_pmsg.log