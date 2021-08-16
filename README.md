# Delta

## Description

Delta is a blazing fast API server built with Rust for REVOLT.

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

### Docker Helper Scripts

If you have Docker installed, you can use the helper scripts to deploy Revolt in your development environment.

| Command        | Description                |
| -------------- | -------------------------- |
| `./build.sh`   | Build Docker Image.        |
| `./run.sh`     | Run Docker container.      |
| `./monitor.sh` | View container logs.       |
| `./remove.sh`  | Kill and remove container. |

## License

Delta is licensed under the [GNU Affero General Public License v3.0](https://github.com/revoltchat/delta/blob/master/LICENSE).
