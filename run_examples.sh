#!/usr/bin/env bash

# Execute command on all files in a directory
# https://stackoverflow.com/questions/10523415/execute-command-on-all-files-in-a-directory
# for i in *; do cmd "$i"; done

# ^([^;]*;[^;]*){30}[^;]+$

# https://stackoverflow.com/questions/12152626/how-can-i-remove-the-extension-of-a-filename-in-a-shell-script

clear

cd ~/.cargo/registry/src/index.crates.io-*/rust_xlsxwriter-0.57.0/examples

for filename in *.rs; do

	# Get the basename without external command
	# by stripping out longest leading match of anything followed by /
	# basename=${filename##*/}

	#example=$(echo "$filename" | cut -f 1 -d '.')
	#example=$(basename "$filename" .rs)
	example=$(echo "${filename%.*}")

	printf "$example";
	cargo run --features=serde,chrono,polars,zlib --example="$example"
	printf "\n";
done
