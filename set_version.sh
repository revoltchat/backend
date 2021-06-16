#!/bin/bash
export version=0.5.0-alpha.2
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
