#!/bin/bash
set -eu

# This script executes  various versions of teip command.
# And print the result of benchmark.

versions=(1.2.1 1.2.2 2.0.0 2.1.0 2.2.0)
testfile="$HOME/repos/greymd/test_files/xsv/nasa_19950801.tsv"
script="$HOME/repos/pythonspeed/cachegrind-benchmarking/cachegrind.py"
tmpdir="$(mktemp -d)"
trap 'rm -rf $tmpdir' EXIT

run_benchmark() {
  local cmd="$1"
  python3 "$script" "$cmd" -f 1 -- cat < "$testfile" > /dev/null
}

{
  cd "$tmpdir"
  for ver in "${versions[@]}"; do
    echo "## teip ${ver}"
    echo
    ## Download and extract
    curl -q -L -o teip.tar.gz \
      "https://github.com/greymd/teip/releases/download/v${ver}/teip-${ver}.x86_64-unknown-linux-musl.tar.gz"
    tar xzf teip.tar.gz
    rm teip.tar.gz
    mv bin/teip "teip-${ver}"
    rm -rf completion man bin doc
    ## Run benchmark
    run_benchmark "./teip-${ver}"
    echo
  done
}
