publish:
    cargo publish --package revolt-config --allow-dirty
    cargo publish --package revolt-result --allow-dirty
    cargo publish --package revolt-permissions --allow-dirty
    cargo publish --package revolt-models --allow-dirty
    cargo publish --package revolt-presence --allow-dirty
    cargo publish --package revolt-database --allow-dirty

patch:
    cargo release version patch --execute

minor:
    cargo release version minor --execute

major:
    cargo release version major --execute

release:
    scripts/try-tag-and-release.sh
