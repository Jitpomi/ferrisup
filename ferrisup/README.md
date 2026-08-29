# FerrisUp CLI guide

FerrisUp creates and restructures Rust projects. It combines local templates, framework-specific setup, Cargo workspace operations, and dependency utilities behind one CLI.

Website: [ferrisup.jitpomi.com](https://ferrisup.jitpomi.com)

FerrisUp is most useful as a structural layer. It makes a project's initial conventions explicit and gives a growing project a supported route toward multiple components. In AI-assisted workflows, that consistency gives coding agents clearer package boundaries and repeatable build instructions. FerrisUp does not replace engineering judgment, code review, tests, or deployment hardening.

## Installation

```bash
cargo install ferrisup
```

From a source checkout:

```bash
cargo install --path ferrisup
```

## Command map

| Command | Current role |
| --- | --- |
| `new` | Create a project or component from a selected template or framework flow. |
| `preview` | Summarize a selection without creating project files. |
| `list` | Show the top-level component types advertised by the CLI. |
| `transform` | Interactively restructure a Rust project or add related components. |
| `component` | Add, remove, or list project components. |
| `workspace` | Initialize, inspect, modify, or optimize a Cargo workspace. |
| `dependency` | Add, remove, update, or analyze Cargo dependencies. |
| `config` | Export or import FerrisUp configuration. |
| `unused-features` | Inspect dependency feature usage for a project. |

Use `ferrisup <command> --help` for the authoritative option list.

## Usage

### Create projects

The `new` command prompts for values that are omitted:

```text
ferrisup new [OPTIONS] [NAME]
```

Supported options are:

- `-c, --component-type <TYPE>`;
- `--framework <FRAMEWORK>`;
- `--provider <PROVIDER>`;
- `--application-type <APPLICATION_TYPE>`;
- `-p, --project-type <PROJECT_TYPE>`;
- `-g, --git`;
- `-b, --build`;
- `--no-interactive`.

Examples:

```bash
ferrisup new starter --component-type minimal --no-interactive
ferrisup new service --component-type server --framework axum --git
ferrisup new frontend --component-type client --framework leptos
ferrisup new function --component-type serverless --provider aws
```

Project names must be safe Cargo package names. FerrisUp rejects path-like names and existing destinations instead of overwriting them. When `--build` is supplied, FerrisUp builds the generated project after creation.

Client flows differ by framework. Leptos uses bundled templates. Dioxus delegates creation to the Dioxus CLI, and Tauri delegates to the official project creator through `npx`; those flows may install or require external tooling and network access.

## Components and template scope

`ferrisup list` currently advertises `minimal`, `library`, `embedded`, `server`, `client`, `serverless`, `data-science`, and `edge`. The argument parser also accepts `shared` for workspace-oriented reusable code.

The bundled template tree mirrors the supported component surface. Treat `ferrisup list`, interactive choices, and command help as authoritative.

Templates are starting points, not finished applications. Review generated dependencies, secrets handling, deployment settings, target support, licensing, and threat models before production use.

## Preview

```text
ferrisup preview [OPTIONS]
```

Preview accepts `--component-type`, `--framework`, `--provider`, and `--application-type`. It does not write a project. It is useful for inspecting the intended structure, but framework-specific generation and complex template substitutions may not be represented exactly.

```bash
ferrisup preview --component-type server --framework poem
ferrisup preview --component-type edge --provider cloudflare
```

## Transform projects

```text
ferrisup transform [--project <PATH>] [--template <TEMPLATE>]
```

Transformation is interactive. Depending on the project state, it can:

- convert a single Cargo package into a workspace;
- add a component to a workspace;
- create a related component without workspace conversion;
- update `.ferrisup/metadata.toml`;
- display project-specific next steps.

Structural changes move files and rewrite Cargo metadata. Run the command from a clean version-controlled project, inspect the proposed choices carefully, and review the complete diff afterward. The `--template` value is accepted by the CLI but the transformation workflow is still primarily interactive.

## Component management

```text
ferrisup component [--action <ACTION>] [--component-type <TYPE>] [--project <PATH>]
```

Actions are `add`, `remove`, and `list`. Omitted values are selected interactively. `--project` defaults to the current directory.

## Workspace management

```text
ferrisup workspace [--action <ACTION>] [--path <PATH>]
```

Actions are `init`, `add`, `remove`, `list`, and `optimize`. The path defaults to the current directory. Workspace operations update Cargo manifests, so review changes before committing them.

## Dependency management

```bash
ferrisup dependency add serde tokio --path ./service
ferrisup dependency add tokio --features macros,rt-multi-thread --no-interactive
ferrisup dependency remove serde --path ./service
ferrisup dependency update tokio --path ./service
ferrisup dependency analyze --path ./service
```

`dependency add` accepts `--dev`, `--features`, `--version`, `--path`, and `--no-interactive`. Without an explicit version, FerrisUp currently records an unconstrained version and lets Cargo resolve it. Network-backed crate lookup can fail independently of local manifest updates.

## Configuration and feature inspection

```bash
ferrisup config --export --path ./ferrisup-config.toml
ferrisup config --import ./ferrisup-config.toml
ferrisup unused-features --path ./service
```

Configuration import and export concern FerrisUp's own configuration. `unused-features` is an analysis aid; verify its findings before removing features.

## AI-assisted development

A reliable agent workflow with FerrisUp is straightforward:

1. Select the component and framework explicitly so generation is reproducible.
2. Generate into a new destination and commit the baseline.
3. Give the agent the generated README, Cargo manifests, and acceptance criteria.
4. Require `cargo fmt`, `cargo clippy`, and `cargo test` as completion checks.
5. Review dependency, security, and deployment changes as normal engineering work.

This is FerrisUp's value in an AI-heavy environment: it turns architectural intent into ordinary, inspectable Rust files. It does not hide the project behind a proprietary runtime or make generated code authoritative.

## Template development

Templates live under `templates/` and are embedded into the released binary. Start with the [template authoring guide](./docs/TEMPLATE_AUTHORING_GUIDE.md). Changes should preserve the shared template schema, exercise the relevant generation path, and compile on the target platform where practical.

## Verification

From the repository root:

```bash
cargo fmt --all --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets
```

See [TESTING.md](./TESTING.md) and [CONTRIBUTING.md](./CONTRIBUTING.md) for project-specific guidance.

## License

FerrisUp is available under the [MIT License](../LICENSE).
