#!/bin/bash
export version=0.4.1-alpha.11
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
