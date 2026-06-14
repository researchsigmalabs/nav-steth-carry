# NAV methodology

How the **stETH Redemption Carry** vault's Net Asset Value (NAV) is computed and pushed on-chain. The
math lives in [`../nav`](../nav) and is fully reproducible from public on-chain state.

## The model

NAV is computed **at par** — stETH ≙ ETH ≙ WETH — as the sum of six inventory legs (`compute_nav`):

```
NAV = liquid_weth
    + native_eth
    + metamorpho_assets
    + steth
    + lido_pending_eth
    + lido_claimable_eth
```

Valuing every leg at par is deliberately conservative: the strategy only realizes a gain when stETH
acquired at a discount is redeemed at par, so the carry's spread is recognized as it clears rather
than marked up in advance.

## How each leg is sourced on-chain

The strategy Safe (Custody Safe
[`0x6ea2…508a`](https://etherscan.io/address/0x6ea2A146b9f575E2D6B539DC7524E56BF484508a)) holds the
vault's assets. Each leg is a direct on-chain read, so anyone can reproduce the NAV:

| Leg | On-chain read |
|-----|---------------|
| `liquid_weth` | `WETH.balanceOf(safe)` — WETH [`0xC02a…6Cc2`](https://etherscan.io/address/0xC02aaa39b223FE8D0A0e5C4F27eAD9083C756Cc2) |
| `native_eth` | the Safe's native ETH balance |
| `metamorpho_assets` | `sigmaETH.convertToAssets(sigmaETH.balanceOf(safe))` — sigmaETH (Morpho Vault v2) [`0x6B83…BF32`](https://etherscan.io/address/0x6B833881A9f083aD2CCACF8c10ABC0dc151cBF32) |
| `steth` | `stETH.balanceOf(safe)` — Lido stETH [`0xae7a…fE84`](https://etherscan.io/address/0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84) (rebasing, ETH-equivalent) |
| `lido_pending_eth` | sum of the **not-yet-finalized** requests in the Lido Withdrawal Queue [`0x889e…F9B1`](https://etherscan.io/address/0x889edC2eDab5f40e902b864aD4d7AdE8E412F9B1) owned by the Safe |
| `lido_claimable_eth` | sum of the **finalized (claimable)** requests in the same queue |

## The sanity guard

`nav_is_sane` runs before every on-chain submission. It rejects:

- a **zero NAV while shares exist** (`totalSupply > 0`), and
- any move larger than `max_jump_pct` versus the previous on-chain `totalAssets`.

A wrong NAV mis-prices every share, so this guard is mandatory.

## On-chain submission

This repository is **only the math**. On-chain, the vault's valuation-provider role (Admin Safe
[`0xEd93…eBc1`](https://etherscan.io/address/0xEd93F0ED8F3cF989806c1A7AFb223870D83bEbc1)) submits the
NAV via Lagoon's `updateNewTotalAssets`, and the curator role (Custody Safe) validates and settles it
before each deposit/redemption cycle (twice daily, 06:00 / 18:00 UTC).
