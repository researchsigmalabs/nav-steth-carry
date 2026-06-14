//! At-par NAV computation for the σ-Labs / Ellen Capital stETH Redemption Carry vault.
//!
//! This is the exact, self-contained valuation math the off-chain keeper runs before it submits a
//! new NAV on-chain (Lagoon `updateNewTotalAssets`). It is **pure**: no network, no secrets, no
//! keys — it takes an [`InventorySnapshot`] of on-chain balances and returns the total NAV in WETH
//! wei. Every leg is valued at par (stETH ≙ ETH ≙ WETH).
//!
//! See `docs/nav.md` for how each leg is sourced on-chain so the figure is independently reproducible.

use alloy_primitives::U256;

/// All balances are WETH-wei equivalents (everything valued at par: stETH ≙ ETH ≙ WETH).
#[derive(Debug, Clone, Default)]
pub struct InventorySnapshot {
    /// WETH held by the strategy Safe — `WETH.balanceOf(safe)`.
    pub liquid_weth: U256,
    /// Native ETH held by the strategy Safe — its account balance.
    pub native_eth: U256,
    /// sigmaETH (Morpho Vault v2) position, in assets:
    /// `sigmaETH.convertToAssets(sigmaETH.balanceOf(safe))`.
    pub metamorpho_assets: U256,
    /// stETH held by the strategy Safe — `stETH.balanceOf(safe)` (rebasing, ETH-equivalent).
    pub steth: U256,
    /// ETH still pending in the Lido withdrawal queue (requested, not yet finalized).
    pub lido_pending_eth: U256,
    /// ETH finalized in the Lido withdrawal queue and claimable.
    pub lido_claimable_eth: U256,
}

/// Full at-par NAV in WETH wei: the sum of every inventory leg.
pub fn compute_nav(s: &InventorySnapshot) -> U256 {
    s.liquid_weth + s.native_eth + s.metamorpho_assets + s.steth
        + s.lido_pending_eth + s.lido_claimable_eth
}

/// Mandatory guard before pushing a NAV. A wrong NAV mis-prices every share, so we reject a zero
/// NAV while shares exist, and any move larger than `max_jump_pct` versus the previous on-chain
/// `totalAssets`.
pub fn nav_is_sane(
    nav: U256,
    prev_total_assets: U256,
    total_supply: U256,
    max_jump_pct: f64,
) -> eyre::Result<()> {
    if nav.is_zero() && !total_supply.is_zero() {
        eyre::bail!("NAV is zero while totalSupply>0");
    }
    if !prev_total_assets.is_zero() {
        // |nav - prev| / prev * 100 <= max_jump_pct
        let (hi, lo) = if nav >= prev_total_assets { (nav, prev_total_assets) } else { (prev_total_assets, nav) };
        let diff = hi - lo;
        // compare diff/prev to max_jump_pct without floats losing precision: diff*100 vs prev*max
        let lhs = diff * U256::from(10_000u64); // diff in bps*100
        let rhs = prev_total_assets * U256::from((max_jump_pct * 100.0) as u64);
        if lhs > rhs {
            eyre::bail!("NAV jump exceeds {}%: prev={} new={}", max_jump_pct, prev_total_assets, nav);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;

    fn wei(eth: u64) -> U256 { U256::from(eth) * U256::from(10u64.pow(18)) }

    #[test]
    fn sums_all_legs_at_par() {
        let snap = InventorySnapshot {
            liquid_weth: wei(5), native_eth: wei(1),
            metamorpho_assets: wei(10), steth: wei(3),
            lido_pending_eth: wei(2), lido_claimable_eth: wei(4),
        };
        // 5 + 1 + 10 + 3 + 2 + 4 = 25
        assert_eq!(compute_nav(&snap), wei(25));
    }

    #[test]
    fn guard_rejects_zero_when_supply_positive() {
        assert!(nav_is_sane(U256::ZERO, wei(10), wei(1), 25.0).is_err());
    }

    #[test]
    fn guard_rejects_big_jump() {
        // new=2x prev, jump 100% > 25% -> reject
        assert!(nav_is_sane(wei(20), wei(10), wei(1), 25.0).is_err());
    }

    #[test]
    fn guard_accepts_small_move() {
        assert!(nav_is_sane(wei(11), wei(10), wei(1), 25.0).is_ok());
    }
}
