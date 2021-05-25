#!/bin/bash
export version=0.4.1-alpha.13
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
