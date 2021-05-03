#!/bin/bash
export version=0.4.1-alpha.7-patch.0
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
