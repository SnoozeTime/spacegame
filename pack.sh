#!/bin/bash
set -e

# Remove existing packed stuff...
# Linus and windows version will be moved in that folder.
DIR_NAME=game_to_release
rm -rf $DIR_NAME
mkdir $DIR_NAME
mkdir ${DIR_NAME}/linux
mkdir ${DIR_NAME}/windows

# 1. First pack all the commercial assets
echo "Build the assets pack"
cargo run --release --bin pack_assets
FILE=./packed.bin
if [  ! -f "$FILE" ]; then
    echo "$FILE was not created..."
    exit 1
fi

# 2. Compile for linux and for windows.
echo "Compile for linux and windows"
cargo build --release --features packed
cross build --release --target x86_64-pc-windows-gnu --features packed

# 3. Move the executables in the correct folders.
echo "Move files"
cp ./target/release/spacegame ${DIR_NAME}/linux
cp ./target/x86_64-pc-windows-gnu/release/spacegame.exe ${DIR_NAME}/windows

# 4. move assets and remove bought assets
cp -r assets ${DIR_NAME}/linux
cp -r assets ${DIR_NAME}/windows
rm -r ${DIR_NAME}/linux/assets/sprites/spaceships
rm -r ${DIR_NAME}/windows/assets/sprites/spaceships
rm -r ${DIR_NAME}/linux/assets/scifi_effects
rm -r ${DIR_NAME}/windows/assets/scifi_effects
if [ -f "${DIR_NAME}/linux/assets/data.bin" ]; then
    rm ${DIR_NAME}/linux/assets/data.bin
fi
if [ -f "${DIR_NAME}/windows/assets/data.bin" ]; then
    rm ${DIR_NAME}/windows/assets/data.bin
fi

cp game_readme.md ${DIR_NAME}/windows/readme.md
cp game_readme.md ${DIR_NAME}/linux/readme.md
cp attributions.txt ${DIR_NAME}/linux
cp attributions.txt ${DIR_NAME}/windows