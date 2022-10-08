# 86_64 Linux
cargo build --target x86_64-unknown-linux-gnu --release
# x86_64 Windows
cargo build --target x86_64-pc-windows-gnu --release

mkdir -p release
bin_name=rustbase

# 86_64 Linux
cp target/x86_64-unknown-linux-gnu/release/$bin_name release/

zip -jq release/$bin_name.zip release/$bin_name

mv release/$bin_name.zip release/$bin_name-linux-x64.zip

rm -rf release/$bin_name

# x86_64 Windows
cp target/x86_64-pc-windows-gnu/release/$bin_name.exe release/

zip -jq release/$bin_name.zip release/$bin_name.exe

mv release/$bin_name.zip release/$bin_name-windows-x64.zip

rm -rf release/$bin_name.exe