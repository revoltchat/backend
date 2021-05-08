#!/bin/bash
export version=0.4.1-alpha.9
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
