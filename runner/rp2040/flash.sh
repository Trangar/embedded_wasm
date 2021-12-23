#!/bin/bash
set -ex

cargo build --release
elf2uf2-rs target/thumbv6m-none-eabi/release/rp2040_runner target/out.uf2
cp target/out.uf2 /media/$USER/RPI-RP2/
