#!/bin/bash
export version=0.5.3-2
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
