#!/system/bin/sh

LOGFILE=/dev/log.log
exec >> "$LOGFILE" 2>&1
echo "start"
system/bin/cp /dev/su /tmp/su
/system/bin/chcon u:object_r:system_file:s0 /tmp/su
/tmp/su --daemon &

echo `ps -ef |grep su`
echo "----------"
echo `ls -la /sys/fs/cgroup/uid_0`
echo "end"

exit 0
