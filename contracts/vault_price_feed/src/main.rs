#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{
    safe_mul_ratio, BASIS_POINTS_DIVISOR, MAX_ADJUSTMENT_BASIS_POINTS, MAX_ADJUSTMENT_INTERVAL,
    MAX_SPREAD_BASIS_POINTS, ONE_USD,
};
use omx_interfaces::vault_price_feed::VaultPriceFeedError;
use stylus_sdk::{block, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct VaultPriceFeed {
        bool initialized;

        address gov;
        // address pyth;
        uint256 max_strict_price_deviation;

        // TODO use pyth to get prices
        mapping (address => uint256) prices;
        // /// token -> price_feed_id
        // mapping (address => bytes32) prices;

        /// token -> spread_bps
        mapping (address => uint256) spread_basis_points;
        /// token -> is_strict_stable
        mapping (address => bool) strict_stable_tokens;

        /// token -> adjustment_bps
        mapping (address => uint256) adjustment_basis_points;
        /// token -> is_adjustment_additive
        mapping (address => bool) is_adjustment_additive;
        /// token -> last_adjustment_timing
        mapping (address => uint256) last_adjustment_timings;
    }
}

impl VaultPriceFeed {
    fn only_gov(&self) -> Result<(), VaultPriceFeedError> {
        if self.gov.get() != msg::sender() {
            return Err(VaultPriceFeedError::Forbidden);
        }

        Ok(())
    }
}

#[external]
impl VaultPriceFeed {
    pub fn init(
        &mut self,
        gov: Address,
        // pyth: Address,
        max_strict_price_deviation: U256,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(VaultPriceFeedError::AlreadyInitialized.into());
        }

        self.gov.set(gov);
        // self.pyth.set(pyth);
        self.max_strict_price_deviation
            .set(max_strict_price_deviation);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn set_adjustment(
        &mut self,
        token: Address,
        is_additive: bool,
        adjustment_bps: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        if self.last_adjustment_timings.get(token) + MAX_ADJUSTMENT_INTERVAL
            < U256::from(block::timestamp())
        {
            return Err(VaultPriceFeedError::AdjustmentFrequencyExceeded.into());
        }

        if adjustment_bps > MAX_ADJUSTMENT_BASIS_POINTS {
            return Err(VaultPriceFeedError::InvalidAdjustmentBps.into());
        }

        self.is_adjustment_additive.insert(token, is_additive);
        self.adjustment_basis_points.insert(token, adjustment_bps);
        self.last_adjustment_timings
            .insert(token, U256::from(block::timestamp()));

        Ok(())
    }

    // pub fn set_pyth(&mut self, pyth: Address) -> Result<(), Vec<u8>> {
    //     self.only_gov()?;

    //     self.pyth.set(pyth);

    //     Ok(())
    // }

    pub fn set_price(&mut self, token: Address, price: U256) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.prices.insert(token, price);

        Ok(())
    }

    pub fn set_spread_basis_points(
        &mut self,
        token: Address,
        spread_basis_points: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        if spread_basis_points > MAX_SPREAD_BASIS_POINTS {
            return Err(VaultPriceFeedError::InvalidSpreadBasisPoints.into());
        }

        self.spread_basis_points.insert(token, spread_basis_points);

        Ok(())
    }

    pub fn set_max_strict_price_deviation(
        &mut self,
        max_strict_price_deviation: U256,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.max_strict_price_deviation
            .set(max_strict_price_deviation);

        Ok(())
    }

    pub fn set_token_config(
        &mut self,
        token: Address,
        is_strict_stable: bool,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        // self.price_feeds.insert(token, price_feed);
        self.strict_stable_tokens.insert(token, is_strict_stable);

        Ok(())
    }

    pub fn get_price(&self, token: Address, maximize: bool) -> Result<U256, Vec<u8>> {
        let price = self.get_price_v1(token, maximize)?;

        let adjustment_bps = self.adjustment_basis_points.get(token);
        if adjustment_bps > U256::ZERO {
            let is_additive = self.is_adjustment_additive.get(token);
            if is_additive {
                price
                    .checked_mul(BASIS_POINTS_DIVISOR + adjustment_bps)
                    .ok_or_else(|| VaultPriceFeedError::InvalidPrice.into())
                    .and_then(|price| {
                        price
                            .checked_div(BASIS_POINTS_DIVISOR)
                            .ok_or_else(|| VaultPriceFeedError::InvalidPrice.into())
                    })
            } else {
                price
                    .checked_mul(BASIS_POINTS_DIVISOR - adjustment_bps)
                    .ok_or_else(|| VaultPriceFeedError::InvalidPrice.into())
                    .and_then(|price| {
                        price
                            .checked_div(BASIS_POINTS_DIVISOR)
                            .ok_or_else(|| VaultPriceFeedError::InvalidPrice.into())
                    })
            }
        } else {
            Ok(price)
        }
    }

    pub fn get_price_v1(&self, token: Address, maximize: bool) -> Result<U256, Vec<u8>> {
        let price = self.get_primary_price(token, maximize)?;

        if self.strict_stable_tokens.get(token) {
            let delta = price.abs_diff(ONE_USD);
            if delta <= self.max_strict_price_deviation.get() {
                return Ok(ONE_USD);
            }

            // if maximize and price is e.g. 1.02, return 1.02
            // if !maximize and price is e.g. 0.98, return 0.98
            if (maximize && price > ONE_USD) || (!maximize && price < ONE_USD) {
                return Ok(price);
            }

            return Ok(ONE_USD);
        }

        let spread_basis_points = self.spread_basis_points.get(token);

        let multiplier = if maximize {
            BASIS_POINTS_DIVISOR
                .checked_add(spread_basis_points)
                .ok_or(VaultPriceFeedError::PriceOverflow)?
        } else {
            BASIS_POINTS_DIVISOR
                .checked_sub(spread_basis_points)
                .ok_or(VaultPriceFeedError::PriceOverflow)?
        };

        safe_mul_ratio(price, multiplier, BASIS_POINTS_DIVISOR)
    }

    pub fn get_primary_price(&self, token: Address, _maximize: bool) -> Result<U256, Vec<u8>> {
        // let feed = self.price_feeds.get(token);

        // let pyth = IPyth::new(self.pyth.get());
        // let price = pyth
        //     .get_price(self, feed)
        //     .map_err(|_| VaultPriceFeedError::CouldNotFetchPrice)?;

        // if price.publish_time + MAX_PRICE_AGE < U256::from(block::timestamp()) {
        //     return Err(VaultPriceFeedError::PriceTooOld.into());
        // }

        // // normalize price precision
        // let price_decimals = price.expo;

        // if price.price < 0 {
        //     return Err(VaultPriceFeedError::InvalidPrice.into());
        // }

        // let price = U256::from(price.price);

        // let normalized_price = if price_decimals >= 0 {
        //     let scale = uint!(10_U256).pow(U256::from(price_decimals));
        //     price
        //         .checked_mul(PRICE_PRECISION)
        //         .ok_or(VaultPriceFeedError::PriceOverflow)?
        //         .checked_mul(scale)
        //         .ok_or(VaultPriceFeedError::PriceOverflow)?
        // } else {
        //     let scale = uint!(10_U256).pow(U256::from(-price_decimals));
        //     safe_mul_ratio(price, PRICE_PRECISION, scale)?
        // };

        // Ok(normalized_price)
        let price = self.prices.get(token);

        if price == U256::ZERO {
            Err(VaultPriceFeedError::InvalidPrice.into())
        } else {
            Ok(price)
        }
    }
}
