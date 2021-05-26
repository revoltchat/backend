#!/bin/bash
export version=0.4.2-alpha.2
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
