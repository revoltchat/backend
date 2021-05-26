#!/bin/bash
export version=0.4.2-alpha.1
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
