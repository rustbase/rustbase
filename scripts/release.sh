# Only works on x86_64 Linux (for while)
cargo build --target x86_64-unknown-linux-gnu --release

bin_name=rustbase

# Copy the binary to the release directory
mkdir -p release
cp target/x86_64-unknown-linux-gnu/release/$bin_name release/

zip -jq release/$bin_name.zip release/$bin_name

mv release/$bin_name.zip release/$bin_name-linux-x64.zip

rm -rf release/$bin_name