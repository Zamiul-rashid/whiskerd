#!/bin/sh
# deb passes "upgrade", rpm passes "1" on upgrade: keep the service enabled then.
case "$1" in
    upgrade|1) exit 0 ;;
esac
systemctl --global disable whiskerd.service >/dev/null 2>&1 || true
