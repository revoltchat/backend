# Revolt Backend

This is a monorepo for the Revolt backend.

| Crate              | Path                                               | Description                   |
| ------------------ | -------------------------------------------------- | ----------------------------- |
| `core/config`      | [crates/core/config](crates/core/config)           | Core: Configuration           |
| `core/database`    | [crates/core/database](crates/core/database)       | Core: Database Implementation |
| `core/models`      | [crates/core/models](crates/core/models)           | Core: API Models              |
| `core/permissions` | [crates/core/permissions](crates/core/permissions) | Core: Permission Logic        |
| `core/presence`    | [crates/core/presence](crates/core/presence)       | Core: User Presence           |
| `core/result`      | [crates/core/result](crates/core/result)           | Core: Result and Error types  |
| `delta`            | [crates/delta](crates/delta)                       | REST API server               |
| `bonfire`          | [crates/bonfire](crates/bonfire)                   | WebSocket events server       |

Note: `january`, `autumn`, and `vortex` are yet to be moved into this monorepo.

## Minimum Supported Rust Version

Rust 1.76 or higher.

> [!CAUTION]
> The events server has a significant performance regression between Rust 1.77.2 and 1.78.0 onwards, see [issue #341](https://github.com/revoltchat/backend/issues/341).

## Development Guide

Before contributing, make yourself familiar with [our contribution guidelines](https://developers.revolt.chat/contrib.html) and the [technical documentation for this project](https://revoltchat.github.io/backend/).

Before getting started, you'll want to install:

- Rust toolchain (rustup recommended)
- Docker
- Git
- mold (optional, faster compilation)

> A **default.nix** is available for Nix users!
> Just run `nix-shell` and continue.

As a heads-up, the development environment uses the following ports:

| Service                   |      Port      |
| ------------------------- | :------------: |
| MongoDB                   |     14017      |
| Redis                     |     14079      |
| MinIO                     |     14009      |
| Maildev                   | 14025<br>14080 |
| Revolt Web App            |     14701      |
| `crates/delta`            |     14702      |
| `crates/bonfire`          |     14703      |
| `crates/services/autumn`  |     14704      |
| `crates/services/january` |     14705      |

Now you can clone and build the project:

```bash
git clone https://github.com/revoltchat/backend revolt-backend
cd revolt-backend
cargo build
```

A default configuration `Revolt.toml` is present in this project that is suited for development.

If you'd like to change anything, create a `Revolt.overrides.toml` file to overwrite it.

You may need to configure the legacy environment options:

```bash
cp .env.example .env
```

Then continue:

```bash
# start other necessary services
docker compose up -d

# run the API server
cargo run --bin revolt-delta
# run the events server
cargo run --bin revolt-bonfire
# run the file server
cargo run --bin revolt-autumn
# run th proxy server
cargo run --bin revolt-january

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
yarn dev --port 14701
```

Then go to https://local.revolt.chat:14701

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

If you have bumped the crate versions, proceed to [GitHub releases](https://github.com/revoltchat/backend/releases/new) to create a changelog.

## Testing

First, start the required services:

```sh
docker compose -f docker-compose.db.yml up -d
```

Now run tests for whichever database:

```sh
TEST_DB=REFERENCE cargo nextest run
TEST_DB=MONGODB cargo nextest run
```

## License

The Revolt backend is generally licensed under the [GNU Affero General Public License v3.0](https://github.com/revoltchat/backend/blob/master/LICENSE).

**Individual crates may supply their own licenses!**
