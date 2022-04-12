#!/bin/sh
# shellcheck shell=dash
echo "stop rungetty.sh to not re-run login"
killall -STOP rungetty.sh
echo "killing login"
killall login
echo "Running RUST installer"
/sbin/installer
echo "resume rungetty.sh"
killall -CONT rungetty.sh