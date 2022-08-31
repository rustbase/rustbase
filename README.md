<div align="center">
    <img src="https://github.com/rustbase.png?size=115">
</div>

# Rustbase
A noSQL key-value database cross-platform program written in [Rust](https://www.rust-lang.org/)

Join our [Discord server](https://discord.gg/m5ZzWPumbd) to get help and discuss features.

# ⚠️ Warning
This is a work in progress. *Current available only for Linux*.

# Development Dependencies
Because of `tonic` dependency, we need some extra dependencies to compile the program.

## Ubuntu
```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y protobuf-compiler libprotobuf-dev
```

## Alpine Linux
```bash
sudo apk add protoc protobuf-dev
```

## MacOS
```bash
brew install protobuf
```

[Reference](https://github.com/hyperium/tonic#dependencies)


# 🔗 Contribute
[Click here](./CONTRIBUTING.md) to see how to Contribute

# Authors
<div align="center">

| [<img src="https://github.com/pedrinfx.png?size=115" width=115><br><sub>@pedrinfx</sub>](https://github.com/pedrinfx) |
| :-------------------------------------------------------------------------------------------------------------------: |


</div>

# License
[MIT License](./LICENSE)
