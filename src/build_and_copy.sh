#!/bin/bash

# Check if the number of arguments is correct
if [ "$#" -ne 2 ]; then
  echo "Error: Invalid number of arguments. Please provide the program name and server folder."
  exit 1
fi

# Set the variables for program name and server folder
program_name="$1"
server_folder="$2"

# Compile the Rust program
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
export OPENSSL_DIR="/opt/homebrew/opt/openssl@3"
export OPENSSL_LIB_DIR="/opt/homebrew/opt/openssl@3/lib"
cargo build --release --target=x86_64-unknown-linux-gnu

# Check if the compilation was successful
if [ "$?" -ne 0 ]; then
  echo "Error: Compilation failed."
  exit 1
fi

# Copy the compiled program to the server using SCP
scp "$program_name" user@server:"$server_folder"

# Check if the file transfer was successful
if [ "$?" -ne 0 ]; then
  echo "Error: Failed to copy the program to the server."
  exit 1
fi

echo "Program successfully compiled and copied to the server."
exit 0
