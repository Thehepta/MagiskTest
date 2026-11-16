#!/system/bin/sh

LOG_TAG=magiskShll
log() { /system/bin/log -t "$LOG_TAG" -p i "$*"; }

SELF="$0"
EXT=/data/local/tmp/ext_init.sh

# 只有「从 /dev 入口执行」且「外部脚本存在」时，才转交给外部脚本
if [ "$SELF" = "/dev/init.sh" ] && [ -f "$EXT" ]; then
    log "external script found, exec $EXT"
    exec /system/bin/sh "$EXT" "$@"
fi

# ===== 以下为内置逻辑（外部脚本不存在时执行）=====
log "running builtin init.sh"
# ... 你的业务逻辑 ...
log "builtin done"



LOGFILE=/data/local/tmp/log.log
/system/bin/chcon u:object_r:system_file:s0 $LOGFILE
chmod 777 $LOGFILE
exec >> "$LOGFILE" 2>&1
echo "start"
system/bin/cp /dev/su /data/local/tmp/su
/system/bin/chcon u:object_r:magisk_file:s0 /data/local/tmp/su
/data/local/tmp/su --daemon &

echo `ps -ef |grep su`
echo "----------"
echo `ls -la /sys/fs/cgroup/uid_0`
echo "end"

exit 0
