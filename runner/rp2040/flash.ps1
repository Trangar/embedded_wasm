# See README.md on requirements
Set-PSDebug -Trace 1
$ErrorActionPreference = "Stop"

cargo build --release
elf2uf2-rs target/thumbv6m-none-eabi/release/rp2040_runner target/out.uf2
Copy-Item target/out.uf2 E:/
