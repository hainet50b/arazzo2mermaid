# arazzo2mermaid

> ⚠️ This project is under active development.  
> The CLI interface is subject to change until the first release.

A lightweight Rust CLI tool that converts Arazzo workflows into Mermaid diagrams.

## Overview

`arazzo2mermaid` converts Arazzo workflows into Mermaid diagrams for documentation and visualization.

The Arazzo ecosystem is still evolving, and dedicated visualization tools are limited. This tool is intended as a bridge to connect Arazzo to the existing Mermaid ecosystem until more dedicated tooling matures.

## Features

- Convert Arazzo workflows into Mermaid flowchart output
- Lightweight single-binary CLI, also Docker-friendly

## Quick Start

### Run with Docker (Recommended)

Docker is the recommended way to run this tool for the following reasons:

- Infrequent usage patterns
- The benefit of keeping local environments clean

```sh
docker run --rm \
  -v $(pwd):/spec \
  arazzo2mermaid arazzo.yml
```

### Run with Binary

Prebuilt binaries may be provided in GitHub releases (initially Linux-only).

### Commands

By default, it writes Mermaid text to standard output.

```sh
arazzo2mermaid arazzo.yml
```

Read from standard input instead of a file:

```sh
cat arazzo.yml | arazzo2mermaid
```

Or use `-` to explicitly specify stdin:

```sh
arazzo2mermaid -
```

Save to a file:

```sh
arazzo2mermaid arazzo.yml -o docs/flowchart.mmd
```

