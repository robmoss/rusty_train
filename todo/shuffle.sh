#!/bin/bash
#
# Rename files so that a new file can be inserted.
#
# USAGE:
#
# Create a gap for a new file with prefix NUM:
#
#     ./shuffle.sh NUM
#
# Remove a gap at prefix NUM:
#
#     ./shuffle.sh --down NUM
#

# Fail if any command fails (-e) or an undefined variable is used (-u).
# Also prevent errors in a pipeline from being masked by the return code of
# the final command in the pipeline (-o pipefail).
set -e -u -o pipefail

usage () {
    echo
    echo "USAGE: $(basename "$0") [-d | --down] NUM"
    echo
    exit 2
}

SHUFFLE_DIRECTION=up
if [ "$#" -gt 1 ]; then
    if [ "$1" = "-d" ] || [ "$1" = "--down" ]; then
        SHUFFLE_DIRECTION=down
        shift
    else
        usage
    fi
fi

if [ "$#" -eq 1 ] && [[ "$1" =~ ^[0-9]+$ ]]; then
    NUM="$1"
else
    usage
fi

# Loop over all files matching DIGITS-*.md.
find . -maxdepth 1 -regex "./[0-9]+-.*\.md" -print0 | sort -z -n -r | while read -r -d $'\0' curr_file; do

    # Determine whether this file starts with a number >= NUM.
    curr_num=$(echo "${curr_file}" | sed 's/^\.\/0*//' | sed 's/-.*//')
    if [ "${curr_num}" -lt "${NUM}" ]; then
        continue
    fi

    # Replace the leading number with the next integer.
    if [ "${SHUFFLE_DIRECTION}" = "down" ]; then
        prefix=$(printf %03d $(( curr_num - 1 )))
    else
        prefix=$(printf %03d $(( curr_num + 1 )))
    fi
    suffix=$(echo "${curr_file}" | sed 's/^\.\/[0-9]\+//')
    dest_file="${prefix}${suffix}"

    # Rename the file.
    git mv "${curr_file}" "${dest_file}"
done
