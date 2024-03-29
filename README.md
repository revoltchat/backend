# Revolt Backend

This is a monorepo for the Revolt backend.

| Crate              | Path                                               | Description                       |
| ------------------ | -------------------------------------------------- | --------------------------------- |
| `core/config`      | [crates/core/config](crates/core/config)           | Core: Configuration               |
| `core/database`    | [crates/core/database](crates/core/database)       | Core: Database Implementation     |
| `core/models`      | [crates/core/models](crates/core/models)           | Core: API Models                  |
| `core/permissions` | [crates/core/permissions](crates/core/permissions) | Core: Permission Logic            |
| `core/presence`    | [crates/core/presence](crates/core/presence)       | Core: User Presence               |
| `core/result`      | [crates/core/result](crates/core/result)           | Core: Result and Error types      |
| `delta`            | [crates/delta](crates/delta)                       | REST API server                   |
| `bonfire`          | [crates/bonfire](crates/bonfire)                   | WebSocket events server           |
| `quark`            | [crates/quark](crates/quark)                       | Models and logic (**DEPRECATED**) |

Note: `january`, `autumn`, and `vortex` are yet to be moved into this monorepo.

## Minimum Supported Rust Version

Rust 1.70 or higher.

## Development Guide

Before getting started, you'll want to install:

- Rust toolchain (rustup recommended)
- Docker
- Git
- mold (optional, faster compilation)

> A **default.nix** is available for Nix users!
> Just run `nix-shell` and continue.

Now you can clone and build the project:

```bash
git clone https://github.com/revoltchat/backend revolt-backend
cd revolt-backend
cargo build
```

If you want to run the API and event servers:

```bash
# create environment file (will be deprecated in future)
cp .env.example .env

# (optionally) copy the default configuration file
cp crates/core/config/Revolt.toml Revolt.toml
# configure as necessary...

# start other necessary services
docker compose up -d

# run the API server
cargo run --bin revolt-delta
# run the events server
cargo run --bin revolt-bonfire

# hint:
# mold -run <cargo build, cargo run, etc...>
```

You can start a web client by doing the following:

```bash
# if you do not have yarn yet and have a modern Node.js:
corepack enable

# clone the web client and run it:
git clone --recursive https://github.com/revoltchat/revite
cd revite
yarn
yarn build:deps
yarn dev --port 3001
```

Then go to https://local.revolt.chat:3001

## Deployment Guide

### Cutting new crate releases

Begin by bumping crate versions:

```bash
just patch # 0.0.X
just minor # 0.X.0
just major # X.0.0
```

Then commit the changes to package files.

Proceed to publish all the new crates:

```bash
just publish
```

### Cutting new binary releases

Tag and push a new release by running:

```bash
just release
```

## License

The Revolt backend is generally licensed under the [GNU Affero General Public License v3.0](https://github.com/revoltchat/backend/blob/master/LICENSE).

**Individual crates may supply their own licenses!**
