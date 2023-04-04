#!/bin/bash

only_target=$1

if [ "$only_target" == "" ]; then
    echo "Building for all targets"
else
    echo "Building only for $only_target"
fi

targets=("x86_64-unknown-linux-gnu" "i686-unknown-linux-gnu" "x86_64-pc-windows-gnu" "i686-pc-windows-gnu" "x86_64-apple-darwin" "i686-apple-darwin" "aarch64-apple-darwin")
short_targets=("linux-x64" "linux-x86" "windows-x64" "windows-x86" "macos-x64" "macos-x86" "macos-arm64")

for t in "${!targets[@]}"; do
    short=${short_targets[$t]}
    target=${targets[$t]}

    if [ "$only_target" != "" ] && [[ "$short" != *"$only_target"* ]]; then
        continue
    fi

    echo "[+] Building for $target"
    # ignore informational messages
    rustup target add $target >/dev/null 2>&1

    cargo build -q --target $target --release

    echo "[Done] Building for $target"
done

bin_name=rustbase

for t in "${!targets[@]}"; do
    target=${targets[$t]}
    short=${short_targets[$t]}

    if [ "$only_target" != "" ] && [[ "$short" != *"$only_target"* ]]; then
        continue
    fi

    echo "[+] Packaging for $short"

    if [[ "$short" == *"windows"* ]]; then
        target_bin=rustbase.exe
    else
        target_bin=rustbase
    fi

    mkdir -p release
    cp target/$target/release/$target_bin release/
    zip -jq release/$bin_name.zip release/$target_bin
    mv release/$bin_name.zip release/$bin_name-$short.zip

    rm -rf release/$target_bin

    echo "[Done] Packaged for $short"
done

echo "[+] Done"
