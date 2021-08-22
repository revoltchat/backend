# Delta

## Description

Delta is a blazing fast API server built with Rust for Revolt.

**Features:**

- Robust and efficient API routes for running a chat platform.
- Distributed notification system, allowing any node to be seamlessly connected.
- Simple deployment, based mostly on pure Rust code and libraries.
- Hooks up to a MongoDB deployment, provide URI and no extra work needed.

## Stack

- [Rocket](https://rocket.rs/) (REST)
- [Async Tungstenite](https://github.com/sdroege/async-tungstenite) (WebSockets)
- [MongoDB](https://mongodb.com/)

## Resources

### Revolt

- [Revolt Project Board](https://github.com/revoltchat/revolt/discussions) (Submit feature requests here)
- [Revolt Testers Server](https://app.revolt.chat/invite/Testers)
- [Contribution Guide](https://developers.revolt.chat/contributing)

## CLI Commands

| Command            | Description                                                                               |
| ------------------ | ----------------------------------------------------------------------------------------- |
| `./publish.sh`     | Publish a Docker Image.                                                                   |
| `./set_version.sh` | Update the version. **Not intended for PR use.**                                          |
| `cargo build`      | Build/compile Delta.                                                                      |
| `cargo run`        | Run Delta.                                                                                |
| `cargo fmt`        | Format Delta. Not intended for PR use to avoid accidentally formatting unformatted files. |

## Contributing

The contribution guide is located at [developers.revolt.chat/contributing](https://developers.revolt.chat/contributing).
Please note that a pull request should only take care of one issue so that we can review it quickly.

## License

Delta is licensed under the [GNU Affero General Public License v3.0](https://github.com/revoltchat/delta/blob/master/LICENSE).
