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
tar -xzf lengua-x86_64-apple-darwin.tar.gz
install lengua /usr/local/bin/lengua
```

Prebuilt archives are published for:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

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
