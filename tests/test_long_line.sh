#!/bin/bash
set -u
readonly THIS_DIR="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
readonly TEST_INPUT_FILE="${THIS_DIR}/test.txt"
readonly TEST_RESULT_FILE="${THIS_DIR}/test_result.txt"
trap 'rm -f "${TEST_INPUT_FILE}" "${TEST_RESULT_FILE}"' EXIT

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
seq 10000 10100 | xargs -I@ sh -c 'printf "%@s\n" | sed "s/ /あ/g"' > "$TEST_INPUT_FILE"
"$TEIP_EXEC" -c 10000- -- sed 's/./い/g' < "$TEST_INPUT_FILE" | sed 's/あ//g' > "$TEST_RESULT_FILE"
for i in {1..100}; do
  line=$(grep -a -m 1 -n -x -E "い{$i}" "$TEST_RESULT_FILE" | cut -d: -f1)
  [[ "$line" != "$i" ]] && {
    echo "test failed" >&2
    exit 1
  }
done

echo "test succeeded" >&2
exit 0
