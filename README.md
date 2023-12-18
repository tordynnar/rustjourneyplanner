# Journey Planner

### About

Journey Planner is an add-on for [Tripwire](https://bitbucket.org/daimian/tripwire), an [Eve Online](https://www.eveonline.com/)
wormhole mapping tool. It uses Tripwire and [EvE-Scout](https://eve-scout.com/) wormhole information to calculate the fastest
route between two systems.

Journey Planner is designed to run entirely within the web browser. It does not have any server-side code. It uses Tripwire as
its backend. It is written entirely in Rust using the [Leptos](https://leptos.dev/) framework, and compiles to
[WASM](https://webassembly.org/) to run in the browser.

### Parsing the Eve SDE

The Eve [Static Data Export (SDE)](https://developers.eveonline.com/resource) contains information needed by Journey Planner
such as system names, classes, gates, etc. A subset of this data needs to be serialized and distributed with Journey Planner.

This step can be skipped because this repository has the serialized SDE included at `./ref/sde.json`.

```shell
# Download the Eve SDE
curl -o ./tools/eve_sde_cli/sde.zip https://eve-static-data-export.s3-eu-west-1.amazonaws.com/tranquility/sde.zip

# Parse the Eve SDE (output to ./ref/sde.json)
(cd tools/eve_sde_cli ; cargo run)
```

### Compiling

```shell
# Use nightly Rust
rustup toolchain install nightly
rustup override set nightly

# Install trunk (https://trunkrs.dev/)
cargo install trunk

# Generate the CSS (output to ./generated/leptonic)
(cd tools/build_theme ; cargo run)

# Build wasm/html/javascript (output to ./dist)
trunk build --release
```

### Test Github Workflow

```shell
# The architecture must be specified if testing from an M-series Mac
act --container-architecture linux/amd64
```