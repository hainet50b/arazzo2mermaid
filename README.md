# arazzo2mermaid

> ⚠️ This project is under active development.  
> The CLI interface is subject to change until the first release.

A lightweight Rust CLI tool that converts Arazzo workflows into Mermaid diagrams.

## Overview

`arazzo2mermaid` converts Arazzo workflows into Mermaid diagrams for documentation and visualization.

The Arazzo ecosystem is still evolving, and dedicated visualization tools are limited. This tool is intended as a bridge
to connect Arazzo to the existing Mermaid ecosystem until more dedicated tooling matures.

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

## Conversion Rules

### Step connections

Steps are connected sequentially by default. When `onSuccess` or `onFailure` actions are defined, they override the
default sequential flow.

| successCriteria | onSuccess | onFailure | Rendering                                                                                                |
|-----------------|-----------|-----------|----------------------------------------------------------------------------------------------------------|
| Defined         | Defined   | Defined   | Rhombus node with condition label. `true` and `false` edges follow the specified actions.                |
| Defined         | Defined   | Omitted   | Rhombus node with condition label. `true` edge follows onSuccess. `false` edge goes to End.              |
| Defined         | Omitted   | Defined   | Rhombus node with condition label. `true` edge goes to the next step. `false` edge follows onFailure.    |
| Defined         | Omitted   | Omitted   | Rhombus node with condition label. `true` edge goes to the next step. `false` edge goes to End.          |
| Omitted         | Defined   | Defined   | Rhombus node without condition label. `true` and `false` edges follow the specified actions.             |
| Omitted         | Defined   | Omitted   | Rhombus node without condition label. `true` edge follows onSuccess. `false` edge goes to End.           |
| Omitted         | Omitted   | Defined   | Rhombus node without condition label. `true` edge goes to the next step. `false` edge follows onFailure. |
| Omitted         | Omitted   | Omitted   | Rectangle node connected to the next step, or End if it is the last step.                                |

### Node shapes

| Shape                   | Meaning                                   |
|-------------------------|-------------------------------------------|
| Rectangle (`[label]`)   | A workflow step                           |
| Rhombus (`{condition}`) | A decision point based on successCriteria |
| Circle (`((End))`)      | End of the workflow                       |

### Defaults from the Arazzo specification

When `onSuccess` is omitted, the next sequential step is executed. When `onFailure` is omitted, the workflow breaks and returns (treated as End in the diagram). These defaults follow the [Arazzo Specification v1.0.1](https://spec.openapis.org/arazzo/latest.html).
