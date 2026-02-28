# arazzo2mermaid

![Build](https://github.com/hainet50b/arazzo2mermaid/actions/workflows/build.yml/badge.svg)

A lightweight Rust CLI tool that converts [Arazzo](https://spec.openapis.org/arazzo/latest.html) workflows into [Mermaid](https://mermaid.js.org/) diagrams.

## Motivation

The Arazzo ecosystem is still evolving, and dedicated visualization tools are limited. This tool is intended as a bridge to connect Arazzo to the existing Mermaid ecosystem until more dedicated tooling matures.

## Features

- Convert Arazzo workflows into Mermaid flowchart output
- Support both YAML and JSON input formats
- Write to standard output or save to a file
- Open diagrams directly in [mermaid.live](https://mermaid.live/)
- Lightweight single-binary CLI, also Docker-friendly

## Quick Start

### Run with Docker

```sh
docker container run --rm \
  -v "$PWD":/spec:ro \
  hainet50b/arazzo2mermaid arazzo.yml
```

Note: The `--live` option is not available when running in Docker.

### Run with Binary

Download the prebuilt binary from [GitHub Releases](https://github.com/hainet50b/arazzo2mermaid/releases) (Linux-only).

## Commands

By default, it reads YAML format and writes Mermaid text to standard output.

```sh
arazzo2mermaid arazzo.yml
```

### Input Options

Read from standard input instead of a file:

```sh
cat arazzo.yml | arazzo2mermaid
```

Or use `-` to explicitly specify stdin:

```sh
arazzo2mermaid -
```

Convert from JSON format:

```sh
arazzo2mermaid --format json arazzo.json
```

### Output Options

Save to a file:

```sh
arazzo2mermaid arazzo.yml -o docs/flowchart.mmd
```

Open in [mermaid.live](https://mermaid.live/) (overrides `-o` and standard output, binary only):

```sh
arazzo2mermaid arazzo.yml --live
```

## Conversion Rules

The following rules are based on the [Arazzo Specification v1.0.1](https://spec.openapis.org/arazzo/v1.0.1.html).

### Node Shapes

| Shape                   | Meaning                                                   |
|-------------------------|-----------------------------------------------------------|
| Rectangle (`[label]`)   | A workflow step                                           |
| Rhombus (`{condition}`) | A decision point based on `successCriteria` or `criteria` |
| Circle (`((End))`)      | End of the workflow                                       |

### Step Connections

Steps are connected sequentially by default. When `onSuccess` or `onFailure` actions are defined, they override the default sequential flow.

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

When `onSuccess` is omitted, the next sequential step is executed. When `onFailure` is omitted, the workflow breaks and returns (treated as End in the diagram). These defaults follow the Arazzo specification.

### Criteria

- When `successCriteria` or `criteria` contains multiple criteria, their conditions are joined with `&&` and displayed as a single rhombus label.
- When an action (such as `onSuccess`) defines `criteria`, an additional rhombus node is inserted in the flow. The `true` edge proceeds to the action target, and the `false` edge goes to End. This behavior when not all criteria are met is not explicitly defined in the Arazzo specification.
- When multiple actions are defined without `criteria`, only the first action is meaningful because an action without `criteria` matches unconditionally. This is valid per the Arazzo specification but semantically ambiguous, and this tool does not guarantee correct rendering for such cases.

### Cross-Workflow Connections

When an action (such as `onSuccess`) specifies `workflowId` instead of `stepId`, the edge goes to the referenced workflow's subgraph node. `workflowId` and `stepId` are mutually exclusive per the Arazzo specification. If both are defined, `workflowId` takes precedence over `stepId`.
