#!/bin/bash
export version=0.4.1-alpha.6
echo "pub const VERSION: String = \"${version}\";" > src/version.rs
