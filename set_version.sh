#!/bin/bash
export version=0.5.0-alpha.0-patch.0
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
