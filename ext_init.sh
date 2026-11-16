#!/system/bin/sh

#!/system/bin/sh

LOG_TAG=magiskShll
log() { /system/bin/log -t "$LOG_TAG" -p i "$*"; }

log "running builtin ext_init.sh"

LOGFILE=/data/local/tmp/log.log
/system/bin/chcon u:object_r:system_file:s0 $LOGFILE
chmod 777 $LOGFILE
exec >> "$LOGFILE" 2>&1
echo "start"
chmod 777 /dev/su
#system/bin/cp /dev/su /data/local/tmp/su
/system/bin/chcon u:object_r:magisk_exec:s0 /dev/su
/dev/su --daemon &

log "-----end-----"
log "end"

exit 0
