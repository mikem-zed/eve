#!/bin/sh
# shellcheck shell=dash
killall -STOP rungetty.sh
killall login
/sbin/installer </dev/console >/dev/console 2>&1
killall -CONT rungetty.sh