# nav

At-par NAV computation for the **stETH Redemption Carry** vault — `compute_nav` (sum of the six
inventory legs) + the `nav_is_sane` guard. Pure: no network, no secrets, no keys; it takes a
snapshot of on-chain balances and returns the total NAV in WETH wei.

See [`../docs/nav.md`](../docs/nav.md) for the methodology — how each leg is sourced on-chain so the
figure is independently reproducible.

```sh
cargo test
```

Apache-2.0 © σ-Labs
