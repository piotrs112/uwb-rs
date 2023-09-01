#!/bin/bash

set -euo pipefail
IFS=$'\n\t'


for ((i=1; i<=30; i++))
do
    label="living-room-p2-$i"
    echo $label
    LABEL=$label TIMEOUT=60 PROBE=/dev/ttyACM1 python3 collect_data.py
done
