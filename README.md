# <h1 align="center"> NIL-SLIP44 </h1>

**Mapping between [SLIP-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md) coin types and the
associated metadata**

SLIP-0044 is a standard that defines coin type values for use in hierarchical deterministic wallets. This crate provides type-safe access to coin types and their metadata.

![Github Actions](https://github.com/newinternetlabs/nil-slip44/workflows/Tests/badge.svg)

## Usage
Add the following dependency to your Cargo manifest...
```toml
[dependencies]
nil-slip44 = "0.3.2"
```
...and see the [docs](https://docs.rs/nil-slip44) or [What can I do?](#what-can-i-do) section below for how to use it.

## What can I do?

```rust
use std::{convert::TryFrom, str::FromStr};
use nil_slip44::{Coin, Symbol};

const BITCOIN_ID: u32 = Coin::Bitcoin.id(); // Coin ID is constant

fn main() {
    assert_eq!(BITCOIN_ID, 0);
    assert_eq!(Coin::Bitcoin.id(), 0);
    assert_eq!(Coin::Bitcoin.ids(), vec![0]); // Coin may have multiple IDs (e.g. Credits)
    assert_eq!(Coin::Bitcoin.name(), "Bitcoin");
    assert_eq!(Coin::Bitcoin.to_string(), "Bitcoin");

    assert_eq!(Coin::try_from(0), Ok(Coin::Bitcoin)); // Try to get Coin from its ID
    assert_eq!(Coin::from_str("Bitcoin"), Ok(Coin::Bitcoin));
    assert_eq!(Coin::from(Symbol::BTC), Coin::Bitcoin); // Get Coin from its Symbol (can't fail, all symbols have associated coins)

    assert_eq!(Symbol::BTC.to_string(), "BTC");

    assert_eq!(Symbol::try_from(0), Ok(Symbol::BTC)); // Try to get coin Symbol from its ID
    assert_eq!(Symbol::try_from(Coin::Bitcoin), Ok(Symbol::BTC)); // Try to convert Coin to Symbol (can fail if no Symbol for Coin is specified)
    assert_eq!(Symbol::from_str("BTC"), Ok(Symbol::BTC));
}
```
