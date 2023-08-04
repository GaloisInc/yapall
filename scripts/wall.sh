#!/usr/bin/env bash

start=${SECONDS}
"${@}"
echo "$(( SECONDS - start ))s"
