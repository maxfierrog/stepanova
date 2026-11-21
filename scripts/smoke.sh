#!/bin/sh
# End-to-end smoke test of the device binary on a running rmemu emulator.
#
# Usage: scripts/smoke.sh RMEMU_DIR BINARY [SCREENSHOT]
#
# Fails unless Stepanova takes over the panel (xochitl stops), a pen stroke
# visibly changes the screen, the power button exits it, and xochitl returns.
set -eu

emu=$1
binary=$2
shot=${3:-smoke.png}

rmemu() { "$emu/bin/rmemu" "$@"; }
fail() { echo "smoke: FAIL: $*" >&2; rmemu ssh 'cat /tmp/stepanova.log' >&2 || true; exit 1; }
running() { rmemu ssh "pidof $1" >/dev/null 2>&1; }
await() {
    i=0
    until eval "$1"; do
        i=$((i + 1))
        [ "$i" -le 30 ] || fail "timed out waiting for: $1"
        sleep 1
    done
}

rmemu push "$binary" /home/root/stepanova >/dev/null
rmemu ssh 'chmod +x /home/root/stepanova; rm -f /tmp/stepanova.log; (nohup /home/root/stepanova > /tmp/stepanova.log 2>&1 &)'

await '! running xochitl && rmemu ssh "grep -q \"stepanova: ready\" /tmp/stepanova.log"'
rmemu wait --idle 1000 >/dev/null
echo "smoke: takeover ok"

drawn=$(rmemu draw 300,500 500,400 700,600 900,450 1200,700 --pressure 0.8)
case $drawn in
    *"screen changed"*) echo "smoke: stroke drawn" ;;
    *) fail "pen stroke did not change the screen ($drawn)" ;;
esac
rmemu wait --idle 500 >/dev/null
rmemu screenshot "$shot" --scale 0.25 >/dev/null

rmemu power >/dev/null
await '! running stepanova && running xochitl'
echo "smoke: exit ok, xochitl restored"

rmemu ssh 'cat /tmp/stepanova.log' | grep -E 'stepanova: [1-9][0-9]* refreshes' \
    || fail "no latency report in log"
echo "smoke: PASS"
