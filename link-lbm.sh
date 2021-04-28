#!/bin/bash

cd LBM
cargo build
cd ..
cp LBM/target/debug/lbmaf.dll Assets/Plugins/lbmaf.dll
rm Assets/Plugins/lbmaf.dll.meta