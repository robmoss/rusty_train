#!/bin/bash
#
# Updates each file in the to-do list book chapter, and rebuilds the book.
#
# Usage:
#
#     ./build_book.sh
#

# Fail if any command fails (-e) or an undefined variable is used (-u).
# Also prevent errors in a pipeline from being masked by the return code of
# the final command in the pipeline (-o pipefail).
set -e -u -o pipefail

# This array maps filename tags to sub-chapter files.
# Note that a single file can have multiple tags, and its contents will appear
# in each of the matching output files.
declare -A tag
tag[DOCS]="documentation.md"
tag[FEAT]="features.md"
tag[IMPL]="implementation.md"
tag[RUST]="rust.md"
tag[MISC]="miscellaneous.md"

# The input and output directories.
dir_todo="$(realpath --relative-to="$(dirname "$0")/.." "$(dirname "$0")/../todo")"
dir_book_root="$(dirname "$0")"
dir_book_out="${dir_book_root}/src/todo"

check_dir_exists () {
    if [ ! -d "$1" ]; then
        echo "ERROR: no directory $1"
        exit 2
    fi
}

check_dir_exists "${dir_todo}"
check_dir_exists "${dir_book_root}"
check_dir_exists "${dir_book_out}"

for tag_name in "${!tag[@]}"; do
    out_file="${dir_book_out}/${tag[${tag_name}]}"
    echo "Writing ${out_file} ..."

    # Start by writing the header content, if any.
    header_file="${dir_todo}/${tag_name}.md"
    if [ -f "${header_file}" ]; then
        echo "    Adding ${header_file} ..."
        cat "${header_file}" > "${out_file}"
    else
        cat /dev/null > "${out_file}"
    fi

    # Append each matching file, in sorted order.
    find "${dir_todo}" -maxdepth 1 -regex ".*/[0-9]+\(-.*\)?-${tag_name}-.*\.md" -print0 | sort -z -n | while read -r -d $'\0' todo_file; do
        echo "    Adding ${todo_file} ..."
        printf "\n" >> "${out_file}"
        cat "${todo_file}" >> "${out_file}"
    done
done

# Update the crate dependency graph.
"${dir_book_root}/draw-dependency-graph.sh"

# Rebuild the book.
(cd "${dir_book_root}" && mdbook build)
