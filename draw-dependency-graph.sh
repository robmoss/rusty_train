#!/bin/bash
#
# Draw the dependency graph for the crates in this workspace.
#
# This image is used in the developer guide.
#

# Support saving the dependency graph in multiple formats.
out_base="./book/src/dev_guide/dependencies"
declare -A output
output[png]="${out_base}.png"



# Save the dependency graph in one or more image formats.
save_graph_images () {
    for format in "${!output[@]}"; do
        out_file="${output[$format]}"
        echo "${out_file} ..."
        print_graph ./crates/* | dot "-T${format}" > "${out_file}"
    done
}



# Print the dependency graph as a directed graph.
print_graph () {
    echo 'strict digraph {'
    echo '  rankdir = "LR";'
    echo '  graph [splines = true, overlap = prism, ranksep = 1, nodesep = 0.125];'
    echo '  edge [tailport = e, headport = w];'
    echo '  node [fontname = "Bitstream Vera Sans Mono", shape = box];'

    for crate_path in "${@}"; do
        crate_name=$(basename "${crate_path}")
        if [ "${crate_name}" = "navig18xx" ]; then
            # Ignore navig18xx, it depends on all of the other crates.
            continue
        fi
        print_crate_deps "${crate_path}" "${crate_name}"
    done

    echo '}'
}



# Print the crate dependencies as directed edges.
print_crate_deps () {
    crate_toml="${1}/Cargo.toml"
    for dep_crate in \
        $(sed -n '/^\[dependencies\]/,/^\[/p' "${crate_toml}" \
              | grep '^n18' \
              | awk '{print $1}'); do
        echo "  ${2} -> ${dep_crate};"
    done
}



# Save the dependency graph images.
save_graph_images
