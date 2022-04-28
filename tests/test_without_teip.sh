#!/bin/bash
set -u
# smart version
# grep -n -C 3 admin < test_secure | awk -F '[-:]' '{print $1}' | awk NF | awk '{ do{ for(s=e=$1; (r=getline)>0 && $1<=e+1; e=$1); print s==e ? s : s","e }while(r>0) }'| awk '{print $0"s/./@/g"}' | sed -f- test_secure > tmp
# inefficient version
grep -n -C 3 admin < test_secure | awk -F '[-:]' '{print $1}' | awk NF | awk '{print $0"s/./@/g"}' | sed -f- test_secure > tmp
echo "sha1sum = $(sha1sum tmp)"
