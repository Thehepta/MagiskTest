#!/system/bin/sh

cmd="$1"
SELF="$0"
LOG_TAG=magiskShll
log() { /system/bin/log -t "$LOG_TAG" -p i "$*"; }

log "running  $SELF : $cmd"

do_nonencrypted(){

    log "running builtin init.sh:do_nonencrypted"
      # ... 你的业务逻辑 ...
    log "builtin done"

    # 只有「从 /dev 入口执行」且「外部脚本存在」时，才转交给外部脚本
    if [ "$SELF" = "/dev/init.sh" ] && [ -f "$EXT" ]; then
        log "external script found, exec $EXT"
        exec /system/bin/sh "$EXT" "$@"
    fi
}
do_boot_completed(){

    log "running builtin init.sh:do_boot_completed"
    # ... 你的业务逻辑 ...
    log "builtin done"

    # 只有「从 /dev 入口执行」且「外部脚本存在」时，才转交给外部脚本
    if [ "$SELF" = "/dev/init.sh" ] && [ -f "$EXT" ]; then
        log "external script found, exec $EXT"
        exec /system/bin/sh "$EXT" "$@"
    fi
}

current_context() {
    id -Z 2>/dev/null || cat /proc/self/attr/current 2>/dev/null || echo "unknown"
}

do_post_fs_data(){

  # ===== 以下为内置逻辑（外部脚本不存在时执行）=====
  log "running builtin init.sh:do_post_fs_data"
  # ... 你的业务逻辑 ...
  log "builtin done"

  EXT=/data/local/tmp/ext_init.sh

  # 只有「从 /dev 入口执行」且「外部脚本存在」时，才转交给外部脚本
  if [ "$SELF" = "/dev/init.sh" ] && [ -f "$EXT" ]; then
      log "external script found, exec $EXT"
      exec /system/bin/sh "$EXT" "$@"
  fi

  selinux_context=$(current_context)
  new_selinux_context=$(echo "$selinux_context" | awk -F: '{OFS=":"; $3=$3"_exec"; print}')
  chmod 777 /dev/su
  /system/bin/chcon $new_selinux_context /dev/su
  /dev/su --daemon &
  exit 0

}

case "$cmd" in
    post-fs-data)   do_post_fs_data   "$@" ;;
    nonencrypted)   do_nonencrypted   "$@" ;;
    boot_completed) do_boot_completed "$@" ;;
esac              # esac 绝对不能省
