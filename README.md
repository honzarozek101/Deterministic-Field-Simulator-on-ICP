# Deterministic Field Simulator on ICP (Rust Canister)

Minimal, deterministic **2D field engine** running as an Internet Computer (ICP) canister.
The canister stores a persistent field state, evolves it via a simple operator, and exposes
a **state hash** for audit / replay style workflows.

This repo is the “A0” implementation: **field + deterministic evolution + hash**.

---

## What this is

- A **2D scalar field** `dim × dim` stored in canister memory (`Vec<f64>`).
- Deterministic initialization from `(dim, seed, alpha)`.
- Deterministic evolution using a 4-neighbor Laplacian (periodic boundary).
- `get_hash()` returns SHA-256 over `(dim, step, alpha, field-bytes)`.

This is intentionally minimal. No UI, no multichain, no oracles.

---

## API (Candid)

- `init_engine(dim: nat32, seed: nat64, alpha: float64) -> ()`
- `tick(n: nat32) -> ()`
- `get_step() -> (nat64) query`
- `get_dim() -> (nat32) query`
- `get_hash() -> (vec nat8) query`
- `get_field_slice(x0: nat32, y0: nat32, w: nat32, h: nat32) -> (vec float64) query`

See: `field_engine.did`

---

## Requirements (Windows)

This setup uses **WSL (Ubuntu)** because `dfx` is installed inside Linux.

Install / have:
- Windows 10/11 + WSL2
- Ubuntu distro in WSL
- `dfx` installed inside Ubuntu
- Rust toolchain inside Ubuntu (`rustup`)
- build tools inside Ubuntu (`build-essential`)

---

## Setup (WSL / Ubuntu)

Open **Ubuntu** terminal (WSL) and install dependencies:

```bash
# 1) DFX
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# restart terminal (or open a new one), then verify:
dfx --version

# 2) Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
cargo --version

# 3) WASM target for Rust canisters
rustup target add wasm32-unknown-unknown

# 4) Build tools (fixes: "linker cc not found")
sudo apt update && sudo apt install -y build-essential pkg-config
