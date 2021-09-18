#!/bin/bash
export version=0.5.3-alpha.4-patch.0
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
