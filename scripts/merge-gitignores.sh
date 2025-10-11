#! /usr/bin/env bash
# this scripts will merge the first level .gitirnores (not recursive) in the current directory into a single .gitignore files from your projects unique sort and remove comments
# to find the root: if ${PROJECT_ROOT} is set, we want it's parent (the root for all projects), otherwise use cwd
PROJECT_ROOT=${PROJECT_ROOT/..:-$(pwd)/}
find "${PROJECT_ROOT}" -maxdepth 2 -name '.gitignore' -exec cat {} \; | grep -v '^#\|^$' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' | sort -u
