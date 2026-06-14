# stETH Redemption Carry

Open reference material for the σ-Labs / Ellen Capital **stETH Redemption Carry** Lagoon vault.

The strategy buys stETH at a Curve discount and redeems it at par through the Lido withdrawal queue,
capturing the spread; capital not currently deployed in the carry is parked in a Morpho idle-yield
engine (sigmaETH). This repository publishes the parts of the keeper we open-source so the vault is
independently verifiable — starting with the **NAV computation**.

- **Live dashboard:** https://www.sigmalabs.fi/vault/0x2746f31096f23670caf4043f8b30d8d02405a257
- **Lagoon vault:** [`0x2746…a257`](https://app.lagoon.finance/vault/1/0x2746f31096f23670Caf4043f8b30D8D02405a257)
- **Governance write-up:** https://www.sigmalabs.fi/blog/lagoon-governance

## Layout

| Path | What |
|------|------|
| [`nav/`](./nav) | The at-par NAV computation crate — the exact valuation math the keeper submits on-chain (`compute_nav` + the `nav_is_sane` guard, with tests). Pure: no network, no secrets, no keys. |
| [`docs/`](./docs) | Methodology. [`docs/nav.md`](./docs/nav.md) documents how every NAV leg is sourced on-chain so the figure is reproducible. |

## License

[Apache-2.0](./LICENSE) © σ-Labs
