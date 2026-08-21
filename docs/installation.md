---
layout: default
title: Installation
nav_order: 2
---

# Installation
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## From a release

Download the archive for your platform from the [latest release](https://github.com/jprybylski/lengua/releases/latest),
extract it, and put `lengua` on your `PATH`:

```bash
tar -xzf lengua_0.1.2_darwin_arm64.tar.gz
install lengua /usr/local/bin/lengua
```

Prebuilt archives are published for:

| Platform | Asset |
|---|---|
| Linux (x86_64) | `lengua_<version>_linux_amd64.tar.gz` |
| macOS (Intel) | `lengua_<version>_darwin_amd64.tar.gz` |
| macOS (Apple Silicon) | `lengua_<version>_darwin_arm64.tar.gz` |
| Windows (x86_64) | `lengua_<version>_windows_amd64.zip` |

## From source

Requires a [Rust toolchain](https://rustup.rs) (stable channel):

```bash
git clone https://github.com/jprybylski/lengua.git
cd lengua
cargo install --path crates/lengua-cli
```

This installs the `lengua` binary into `~/.cargo/bin` (make sure that's on your `PATH`).

## Verifying the install

```bash
lengua --version
lengua --help
```
