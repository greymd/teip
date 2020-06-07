#!/bin/bash
set -eu

is_registerable() {
  local _code="$1"
  local _res_code
  _res_code="$(curl --retry 3 -s -o /dev/null -w "%{http_code}" "https://git.io/${_code}")"
  if [[ "$_res_code" == 302 ]];then
    echo "code ${_code} is NOT registerable" >&2
    return 1
  else
    echo "code ${_code} is registerable" >&2
    return 0
  fi
}

shorten() {
  local _url="$1"
  local _code="$2"
  local _res_code
  _res_code="$(curl --retry 3 -s -o /dev/null -w "%{http_code}" -F "url=${_url}" -F "code=${_code}" "https://git.io")"
  if [[ "$_res_code" == 201 ]];then
    echo "Succeed to register ${_code}, yay" >&2
    return 0
  else
    echo "Failed to register ${_code} .." >&2
    return 1
  fi
}

main() {
  local _url="$1"
  local _code="$2"
  is_registerable "$_code" \
    && shorten "$_url" "$_code" \
    && echo "https://git.io/$_code"
}

main ${1+"$@"}
