#!/bin/bash
export version=0.5.1-alpha.8-patch.0
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
