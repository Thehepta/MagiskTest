#!/system/bin/sh

#!/system/bin/sh
cmd="$1"
SELF="$0"
LOG_TAG=magiskShllEXT
log() { /system/bin/log -t "$LOG_TAG" -p i "$*"; }

current_context() {
    id -Z 2>/dev/null || cat /proc/self/attr/current 2>/dev/null || echo "unknown"
}

log "running builtin ext_init.sh, context=$(current_context)"

do_nonencrypted(){
    log "running builtin init.sh:do_nonencrypted"
    # ... 你的业务逻辑 ...
    log "builtin done"
}
do_boot_completed(){
    log "running builtin init.sh:do_boot_completed"
    # ... 你的业务逻辑 ...
    log "builtin done"
}

do_post_fs_data(){

    log "running builtin init.sh:do_post_fs_data"
    chmod 777 /dev/su
    /system/bin/chcon u:object_r:magi1sk_exec:s0 /dev/su
    /dev/su --daemon &
    log "-----end-----"

    exit 0
}

case "$cmd" in
    post-fs-data)   do_post_fs_data   "$@" ;;
    nonencrypted)   do_nonencrypted   "$@" ;;
    boot_completed) do_boot_completed "$@" ;;
esac              # esac 绝对不能省


