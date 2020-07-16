#!/bin/bash

 SCRIPT_DIR=""
 if which realpath > /dev/null; then
     SCRIPT_DIR="$(realpath .)"
 else
     SCRIPT_DIR="$(dirname $0 | xargs readlink -f)"
 fi

 cd ${SCRIPT_DIR}
./aalto_tg_bot > aalto_tg_bot.log 2>&1