#!/bin/bash
set -eo pipefail

git add -u
git commit --amend --no-edit
make pkg/installer
make pkg/mkimage-raw-efi      
make pkg/eve
make installer-raw