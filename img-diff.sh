#!/bin/sh
#
# Visually compare an image against the most recently committed version.
#
# USAGE:
#
#     img-diff.sh path/to/image.png
#

set -u

if [ "$#" -ne 1 ]; then
    echo "USAGE: $(basename "$0") path/to/image.png"
    exit 2
fi

VIEWER=ristretto

for cmd in git compare "${VIEWER}"; do
    command -v ${cmd} > /dev/null || {
    echo "Could not find command '${cmd}'"
    exit 2
    }
done

ORIG_FILE=$(mktemp -t cmp-original-XXXX.png)
DIFF_FILE=$(mktemp -t cmp-difference-XXXX.png)

git cat-file -p "HEAD:$1" > "${ORIG_FILE}" || {
    echo "Could not find original version of $1"
    exit 2
}
# NOTE: returns "a value between 0 and 1" if the images differ.
compare "${ORIG_FILE}" "$1" -compose src "${DIFF_FILE}"
"${VIEWER}" "${ORIG_FILE}" "$1" "${DIFF_FILE}"

rm "${ORIG_FILE}" "${DIFF_FILE}"
