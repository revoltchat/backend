# Revolt Backend

This is a monorepo for the Revolt backend.

| Crate            | Path                                           | Description                          |
| ---------------- | ---------------------------------------------- | ------------------------------------ |
| `delta`          | [crates/delta](crates/delta)                   | REST API server                      |
| `bonfire`        | [crates/bonfire](crates/bonfire)               | WebSocket events server              |
| `quark`          | [crates/quark](crates/quark)                   | Models and logic                     |
<!--| `revcord/api`    | [crates/revcord/api](crates/revcord/api)       | Discord REST translation layer       |
| `revcord/ws`     | [crates/revcord/ws](crates/revcord/ws)         | Discord gateway translation layer    |
| `revcord/models` | [crates/revcord/models](crates/revcord/models) | Discord models and quark translation |-->

Note: `january`, `autumn`, and `vortex` are yet to be moved into this monorepo.

## Resources

### Revolt

- [Revolt Project Board](https://github.com/revoltchat/revolt/discussions) (Submit feature requests here)
- [Revolt Testers Server](https://app.revolt.chat/invite/Testers)
- [Contribution Guide](https://developers.revolt.chat/contributing)

## Contributing

The contribution guide is located at [developers.revolt.chat/contributing](https://developers.revolt.chat/contributing).
Please note that a pull request should only take care of one issue so that we can review it quickly.

## License

The Revolt backend is generally licensed under the [GNU Affero General Public License v3.0](https://github.com/revoltchat/backend/blob/master/LICENSE). Please check individual crates for further license information.
