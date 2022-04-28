#!/bin/bash
set -u
readonly THIS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
grep -n -C 3 admin < "$THIS_DIR"/test_secure | awk -F '[-:]' '{print $1}' | awk 'NF{print NR"s/./@/g"}' | sed -f- test_secure > tmp
echo "sha1sum = $(sha1sum tmp)"
