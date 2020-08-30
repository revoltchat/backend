![](assets/banner.png)

Delta is a **blazing fast API server** built with Rust for the REVOLT platform.

Features:
- Robust and efficient API routes for running a chat platform.
- Distributed notification system, allowing any node to be seamlessly connected.
- Simple deployment, based mostly on pure Rust code and libraries.
- Hooks up to a MongoDB deployment, provide URI and no extra work needed.

## Docker Helper Scripts

If you have Docker installed, you can use the helper scripts to deploy Revolt in your development environment.

```bash
./build.sh   # build Docker image
./run.sh     # run Docker container
./monitor.sh # view container logs
./remove.sh  # kill and remove container
```
