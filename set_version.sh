#!/bin/bash
export version=0.5.3-alpha.0-patch.1
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
