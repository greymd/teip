#!/bin/bash
set -u
readonly THIS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
# HASH_ANSWER can be obtained by this way
# ===========================
# zcat test_secure.gz | grep -n -C 3 admin | awk -F '[-:]' '{print $1}' | awk NF > grep_n_C_3_admin.txt
# sed 's|.*|&s/./@/g|' < grep_n_C_3_admin.txt | sed -f- test_secure > grep_n_answer
# sha256sum grep_n_answer
# ===========================
readonly HASH_ANSWER="bbae7a4e85b4b2100c64e6adf443a4d650fef8e1f0831ae9b593e15e8ca5afe6"

teip_exec () {
  local res
  res="$THIS_DIR"/../target/release/teip
  [[ -f "$res" ]] && {
    echo "$res"
    return
  }
  res="$THIS_DIR"/../target/debug/teip
  [[ -f "$res" ]] && {
    echo "$res"
    return
  }
  echo "cargo run --"
  return
}

readonly TEIP_EXEC=$(teip_exec)
rm -f test_secure.gz
curl --retry 3 -OL "https://github.com/greymd/test_files/raw/v1.0.0/logs/test_secure.gz"
zcat test_secure.gz | $TEIP_EXEC -e 'grep -n -C 3 admin' -- sed 's/./@/g' > grep_n_result
if [[ "$( sha256sum grep_n_result | awk '{print $1}' )" == "$HASH_ANSWER" ]];then
  echo "[SUCCEEDED] TEST WITH LARGE FILE: PATH = $TEIP_EXEC"
  exit 0
else
  echo "[FAILED] TEST WITH LARGE FILE: PATH = $TEIP_EXEC"
  exit 1
fi
