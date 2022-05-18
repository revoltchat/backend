#!/bin/bash
export version=0.5.3-5-patch.2
echo "pub const VERSION: &str = \"${version}\";" > src/version.rs
