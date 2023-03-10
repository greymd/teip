#!/usr/bin/env bash
set -eu
THIS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cargo run -- --completion bash > "$THIS_DIR"/bash/teip
cargo run -- --completion fish > "$THIS_DIR"/fish/teip.fish
cargo run -- --completion zsh > "$THIS_DIR"/zsh/_teip
cargo run -- --completion powershell > "$THIS_DIR"/powershell/teip.ps1
