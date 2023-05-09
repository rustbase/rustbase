<div align="center">

<img src="https://github.com/rustbase.png?size=115">
    
<h1>
Rustbase
</h1>

### A noSQL key-value database cross-platform program written in [Rust](https://www.rust-lang.org/)

<font size="2" >Rustbase is open source, lightweight, modern and fast NoSQL database, using DustData as the storage engine.</font>

<br />

[![](https://img.shields.io/badge/Donate-Stripe-%23635afe?style=for-the-badge)](https://donate.stripe.com/4gw8xx3wc1Uyb96288)

</div>


<br />

# ⚠️ Warnings

-   This is a work in progress, so it is not recommended to use it in production.

# Supported platforms

|         | Windows | Linux | MacOS |
| ------- | ------- | ----- | ----- |
| i686    | ✅       | ✅     | -     |
| x64     | ✅       | ✅     | -     |
| aarch64 | -       | -     | -     |
| arm     | -       | -     | -     |

# Installation

## Windows (WSL) & Linux
You can install the latest version of Rustbase using the following command:
```bash
curl -L https://www.rustbase.app/install | bash
```

### Manual Installation
You can download the latest version of Rustbase [here](https://github.com/rustbase/rustbase/releases).


## MacOS
**Currently, Rustbase is not available for MacOS.**

## Docker
You can use the official Docker image of Rustbase [here](https://github.com/rustbase/rustbase/pkgs/container/rustbase).

```bash
docker pull ghcr.io/rustbase/rustbase:latest
```

# External Components
## Rustbase CLI
Use the [Rustbase CLI](https://github.com/rustbase/rustbase-cli) to manage your Rustbase Server.

## DustData
[DustData](https://github.com/rustbase/dustdata) is a data concurrency control key-value storage engine to Rustbase

# 🔗 Contribute

[Click here](./CONTRIBUTING.md) to see how to Contribute

Join our [Discord server](https://discord.gg/m5ZzWPumbd) to get help and discuss features.

# Internal Components

-   [Config](./src/config/)
-   [Query](./src/query/)
-   [Server](./src/server/)
    -   [Cache](./src/server/cache/)
    -   [Engine](./src/server/engine/)
    -   [Route](./src/server/route/)
    -   [Wirewave](./src/server/wirewave/)
-   [Utils](./src/utils/)

# Authors

<div align="center">

| [<img src="https://github.com/peeeuzin.png?size=115" width=115><br><sub>@peeeuzin</sub>](https://github.com/peeeuzin) |
| :-------------------------------------------------------------------------------------------------------------------: |

</div>

# License

[MIT License](./LICENSE)
