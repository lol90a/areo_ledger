# Running the Solana escrow tests

Two suites, with very different costs:

| Suite | What it covers | Toolchain needed |
|---|---|---|
| `cargo test --lib` | 14 pure unit tests: the Ed25519 quote-attestation parser, account sizing, refund-window bounds | A normal Rust toolchain. Works on Windows. |
| `anchor test` | 38 integration tests against a local validator: governance, signed-quote enforcement, settlement, refunds, pause, account substitution | Solana + Anchor. **Does not work natively on Windows** — `solana-test-validator` is not supported there. |

## Unit tests

```bash
cd contracts/solana
cargo test --lib
```

Expected: `14 passed`.

## Integration tests

The pinned combination below is the one these tests are verified against. It is
not arbitrary — see *Why these versions* at the end.

| Component | Version |
|---|---|
| Solana CLI | 1.18.26 |
| Anchor CLI | 0.29.0 |
| Node | >= 20.18 |
| platform-tools | v1.41 (rustc 1.75) |

### The quickest reproducible route: Docker

The `backpackapp/build:v0.29.0` image carries Anchor 0.29 and a Solana CLI, but
it ships Node 18 and Solana 1.17, both of which are too old here — so the two
get replaced inside the container.

```bash
docker run -d --name aeroledger-anchor backpackapp/build:v0.29.0 sleep infinity

# Node 20 (the image's Node 18 cannot install @solana/codecs-*)
docker exec aeroledger-anchor bash -lc '
  curl -fsSL -o /tmp/node.tar.xz https://nodejs.org/dist/v20.18.1/node-v20.18.1-linux-x64.tar.xz
  mkdir -p /opt/node20 && tar -xJf /tmp/node.tar.xz -C /opt/node20 --strip-components=1'

# Solana 1.18.26 (the image's 1.17 builds with rustc 1.68, too old for the deps)
docker exec aeroledger-anchor bash -lc 'solana-install init 1.18.26'

# Copy the project in, excluding build output
tar -cf - --exclude=target --exclude=node_modules --exclude=.anchor \
  -C contracts/solana . |
  docker exec -i aeroledger-anchor bash -c 'mkdir -p /build && tar -xf - -C /build'

docker exec aeroledger-anchor bash -lc '
  export PATH=/opt/node20/bin:$PATH
  mkdir -p ~/.config/solana
  [ -f ~/.config/solana/id.json ] ||
    solana-keygen new --no-bip39-passphrase --silent -o ~/.config/solana/id.json
  cd /build
  yarn install --frozen-lockfile
  anchor build
  PID=$(solana-keygen pubkey target/deploy/aeroledger_escrow-keypair.json)
  sed -i "s/Fg6PaFpoGXkYsidMpWxTWqK6W2BeZ7FEfcYkgqPLF6j/$PID/g" \
    Anchor.toml programs/aeroledger-escrow/src/lib.rs
  anchor build
  anchor test'
```

Expected: `38 passing`.

### Native (Linux / macOS / WSL)

```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"
cargo install --git https://github.com/coral-xyz/anchor --tag v0.29.0 anchor-cli --locked
cd contracts/solana
yarn install --frozen-lockfile
anchor build
PID=$(solana-keygen pubkey target/deploy/aeroledger_escrow-keypair.json)
sed -i "s/Fg6PaFpoGXkYsidMpWxTWqK6W2BeZ7FEfcYkgqPLF6j/$PID/g" \
  Anchor.toml programs/aeroledger-escrow/src/lib.rs
anchor build
anchor test
```

## Program id

`declare_id!` and `Anchor.toml` both carry the Anchor template placeholder
`Fg6PaFpoGXkYsidMpWxTWqK6W2BeZ7FEfcYkgqPLF6j`. That is deliberate: it means the
program cannot be deployed by accident, because the declared id will never match
a real deploy keypair.

Before `anchor test` can deploy, both must match the keypair Anchor generated in
`target/deploy/`. **Do not commit that change** — the placeholder is the safe
default in version control. `git checkout -- Anchor.toml
programs/aeroledger-escrow/src/lib.rs` afterwards, or run the tests in a
container as above, where the edit stays throwaway.

**Use the `sed` above rather than `anchor keys sync`.** Three `[programs.*]`
sections declare the same program, and `anchor keys sync` updates only one of
them per invocation — so a single run leaves `[programs.localnet]` on the
placeholder while `declare_id!` has moved, and `anchor test` then deploys to one
address and runs code that declares another:

```
Error Code: DeclaredProgramIdMismatch. Error Number: 4100.
```

`anchor keys sync` also rewrites `Anchor.toml` through a TOML round-trip, which
drops every comment in the file — including the note at the top explaining why
the placeholder id is there at all.

## Why these versions

Each pin below exists because something concrete broke without it. They are
recorded so a future upgrade knows what it is re-opening.

- **Solana 1.18.26, not 1.17.** `Cargo.lock` pins `solana-program 1.18.26`, and
  parts of that dependency set need rustc >= 1.71. Solana 1.17's platform-tools
  ship rustc 1.68, which fails on `zmij` (a `serde_json` dependency). 1.18.26's
  platform-tools v1.41 ship rustc 1.75.

- **`Cargo.lock` is lockfile format v3, not v4.** The Solana toolchain builds
  with cargo 1.73, which cannot parse v4 at all and aborts before compiling
  anything. A modern host cargo writes v4 by default, so if you ever regenerate
  the lock, convert the `version = 4` line on line 3 back to `version = 3`.

- **Several crates are pinned below their latest release** — `blake3 1.5.5`,
  `proc-macro-crate 3.1.0`, `indexmap 2.2.6`, `jobserver 0.1.31`, `borsh 1.5.1`,
  `rayon 1.10.0`, `unicode-segmentation 1.11.0`, `wasm-bindgen 0.2.92`,
  `js-sys`/`web-sys 0.3.69`. Each newer version requires either edition 2024 or
  a rustc above what platform-tools provides. Re-resolving the lock with a
  modern cargo will quietly pull them forward again and break `anchor build`.

- **Node >= 20.18.** `@solana/spl-token` pulls `@solana/codecs-numbers`, whose
  `engines` field refuses Node 18.

- **`rpc-websockets` pinned to 7.10.0.** 7.11.0 restructured `dist/`, turning
  `dist/lib/client` from a file into a directory with no `index.js`, which
  breaks `@solana/web3.js@1.91.8` at require time.

- **`@solana/web3.js` resolved to a single 1.91.8.** Anchor 0.29 otherwise
  resolves its own nested copy, and the two copies disagree about which
  `rpc-websockets` major they need.

`yarn.lock` is committed precisely so this set does not drift. Use
`yarn install --frozen-lockfile`.

## What the integration suite asserts

Every test named `rejects …` asserts a *specific* error code, not merely that
something threw. `expectError` resolves an error name to its numeric code
through the program IDL and Anchor's `LangErrorCode`, because a rejection raised
during preflight simulation arrives as a bare `custom program error: 0x1781`
with no logs for Anchor to translate. Matching on names alone would report a
correct rejection as a failure.

Three cases assert a mechanism that is worth knowing about:

- *rejects a forged escrow vault* — the substituted account is never consulted.
  Anchor derives the vault address from `(order, mint)` and creates it by CPI,
  so the CPI references the genuine ATA, which the attacker did not include in
  the transaction. The runtime rejects it as a missing account.
- *rejects a substituted payout token account* and *rejects refunding to an
  account that is not the buyer's ATA* — `associated_token::authority` checks
  the token owner before the ATA address, so these fail with
  `ConstraintTokenOwner`.
- *rejects a duplicate order id* — the Order PDA already exists, so `init` fails
  in the System Program with `AccountAlreadyInUse` (error 0).
