publish:
    cargo publish --package revolt-parser
    cargo publish --package revolt-result
    cargo publish --package revolt-config
    cargo publish --package revolt-files
    cargo publish --package revolt-permissions
    cargo publish --package revolt-models
    cargo publish --package revolt-presence
    cargo publish --package revolt-database

patch:
    cargo release version patch --execute

minor:
    cargo release version minor --execute

major:
    cargo release version major --execute

release:
    scripts/try-tag-and-release.sh
