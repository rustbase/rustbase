#!/bin/bash

only_target=$1

case "$only_target" in
    *-linux*)
        echo "Building only for Linux"
        only_target="linux"
        ;;

    *-windows*)
        echo "Building only for Windows"
        only_target="windows"
        ;;

    *-macos*)
        echo "Building only for Mac OS"
        only_target="macos"
        ;;

    *)
        echo "Building for all targets"
        only_target=""
        ;;
esac

targets=("x86_64-unknown-linux-gnu" "i686-unknown-linux-gnu" "x86_64-pc-windows-gnu" "i686-pc-windows-gnu")
short_targets=("linux-x64" "linux-x86" "windows-x64" "windows-x86")

for t in "${targets[@]}"; do
    if [ "$only_target" != "" ] && [[ "$t" != *"$only_target"* ]]; then
        continue
    fi

    echo "[+] Building for $t"
    # ignore informational messages
    rustup target add $t > /dev/null 2>&1

    cargo build -q --target $t --release

    echo "[Done] Building for $t"
done

bin_name=rustbase

for t in "${!targets[@]}"; do
    target=${targets[$t]}
    short=${short_targets[$t]}

    if [ "$only_target" != "" ] && [[ "$target" != *"$only_target"* ]]; then
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