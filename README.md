# Mosaic

<div align="center">
  <img src="./assets/logo.png" alt="Mosaic Logo" width="300">
</div>

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)](https://www.rust-lang.org/)
[![Status: In Development](https://img.shields.io/badge/Status-In%20Development-blue)](https://github.com/yourusername/mosaic)
[![Discord](https://img.shields.io/discord/DISCORD_ID?label=Polytoria&logo=discord)](https://polytoria.com)

**A package manager for Polytoria game development.**

Mosaic simplifies how developers share and manage reusable Lua libraries. Install packages, manage versions, and build games faster.

---

## Quick Start

```bash
# Initialize your project
mosaic init

# Install a package
mosaic install logger@1.0.0

# Use it in your game
```

In Polytoria Creator:

```lua
local Logger = require(game["ScriptService"]["logger"])
Logger:info("Hello, Polytoria!")
```

---

## Features

- **Simple Installation** — One command to add libraries to your game
- **Version Management** — Specify exact package versions
- **Automatic Injection** — Packages are injected directly into your .poly files
- **No Friction** — Works with ModuleScripts, no build steps required
- **Community Driven** — Built by the Polytoria community, for the Polytoria community

---

## Why Mosaic?

Polytoria developers currently share code through the Asset Store or manual copy-paste. This works, but lacks version control and dependency management.

Mosaic brings modern package management to Polytoria:

| Feature               | Before                             | With Mosaic                   |
| --------------------- | ---------------------------------- | ----------------------------- |
| Installing a library  | Download model, copy code manually | `mosaic install`              |
| Version control       | None                               | Automatic, with `mosaic.toml` |
| Updating packages     | Re-download and replace            | `mosaic update`               |
| Managing dependencies | Manual tracking                    | Declarative, version-locked   |

---

## Installation

**Coming soon.** Mosaic is currently in active development.

Follow this repository or join the [Polytoria Discord](https://polytoria.com) for updates.

---

## How It Works

1. **Initialize a project** — `mosaic init` creates a `mosaic.toml` file
2. **Declare dependencies** — Add packages to your config
3. **Install packages** — `mosaic install` downloads and injects ModuleScripts
4. **Use in your game** — Require packages like any Polytoria module

Your `mosaic.toml`:

```toml
[package]
name = "my-game"
version = "1.0.0"

[dependencies]
logger = "github:username/polytoria-logger@1.0.0"
events = "github:username/polytoria-events@2.1.0"
```

---

## Documentation

- [Getting Started](./docs/GETTING_STARTED.md)
- [CLI Reference](./docs/CLI_REFERENCE.md)
- [Architecture](./docs/ARCHITECTURE.md)
- [Publishing Your Package](#) (Coming soon)

---

## Contributing

We welcome contributions from the Polytoria community. See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
git clone https://github.com/yourusername/mosaic.git
cd mosaic
cargo build
cargo run -- --help
```

---

## Roadmap

- [x] Project setup and branding
- [ ] CLI package manager (MVP)
- [ ] Registry API and database
- [ ] Website and discovery UI
- [ ] Publishing system
- [ ] Advanced features (version resolution, updates, etc.)

---

## License

MIT License — see [LICENSE](./LICENSE) for details.

---

## Community

Questions? Ideas? Found a bug?

- Open an [issue](https://github.com/yourusername/mosaic/issues)
- Join the [Polytoria Discord](https://polytoria.com)
- Contribute code via [pull request](https://github.com/yourusername/mosaic/pulls)

**Made with ❤️ for the Polytoria community**
