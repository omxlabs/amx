#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use alloy_primitives::{Address, U256};
use omx_common::{call_context::GetCallContext, safe_mul_ratio};
use omx_interfaces::{
    base_token::IBaseToken,
    olp_manager::IOlpManager,
    reward_router::{RewardRouterError, StakeOmx, UnstakeOlp, UnstakeOmx},
    reward_tracker::IRewardTrackerStaking,
};
use stylus_sdk::{evm, msg, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct RewardRouter {
        bool initialized;

        address gov;

        address omx;
        address es_omx;
        address bn_omx;

        address olp; // OMX Liquidity Provider token

        address staked_omx_tracker;
        address bonus_omx_tracker;
        address staked_omx_tracker_staking;
        address bonus_omx_tracker_staking;
        address fee_omx_tracker_staking;
        address staked_olp_tracker_staking;
        address fee_olp_tracker_staking;

        address olp_manager;
    }
}

impl RewardRouter {
    fn only_gov(&self) -> Result<(), RewardRouterError> {
        if self.gov.get() != msg::sender() {
            return Err(RewardRouterError::Forbidden);
        }

        Ok(())
    }

    fn compound_internal(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.compound_omx_internal(account)?;
        self.compound_olp_internal(account)?;

        Ok(())
    }

    fn compound_omx_internal(&mut self, account: Address) -> Result<(), Vec<u8>> {
        let staked_omx_tracker = IRewardTrackerStaking::new(self.staked_omx_tracker_staking.get());
        let bonus_omx_tracker = IRewardTrackerStaking::new(self.bonus_omx_tracker_staking.get());

        let es_omx_amount = staked_omx_tracker.claim_for_account(self.ctx(), account, account)?;
        if es_omx_amount > U256::ZERO {
            self.stake_omx_internal(account, account, self.es_omx.get(), es_omx_amount)?;
        }

        let bn_omx = self.bn_omx.get();

        let bn_omx_amount = bonus_omx_tracker.claim_for_account(self.ctx(), account, account)?;
        if bn_omx_amount > U256::ZERO {
            let fee_omx_tracker = IRewardTrackerStaking::new(self.fee_omx_tracker_staking.get());
            fee_omx_tracker.stake_for_account(
                self.ctx(),
                account,
                account,
                bn_omx,
                bn_omx_amount,
            )?;
        }

        Ok(())
    }

    fn compound_olp_internal(&mut self, account: Address) -> Result<(), Vec<u8>> {
        let staked_olp_tracker = IRewardTrackerStaking::new(self.staked_olp_tracker_staking.get());
        let es_omx_amount = staked_olp_tracker.claim_for_account(self.ctx(), account, account)?;

        if es_omx_amount > U256::ZERO {
            self.stake_omx_internal(account, account, self.es_omx.get(), es_omx_amount)?;
        }

        Ok(())
    }

    fn stake_omx_internal(
        &mut self,
        funding_account: Address,
        account: Address,
        token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if amount == U256::ZERO {
            return Err(RewardRouterError::InvalidAmount.into());
        }

        let staked_omx_tracker_staking =
            IRewardTrackerStaking::new(self.staked_omx_tracker_staking.get());
        let bonus_omx_tracker_staking =
            IRewardTrackerStaking::new(self.bonus_omx_tracker_staking.get());
        let fee_omx_tracker_staking =
            IRewardTrackerStaking::new(self.fee_omx_tracker_staking.get());

        let staked_omx_tracker = self.staked_omx_tracker.get();
        let bonus_omx_tracker = self.bonus_omx_tracker.get();

        staked_omx_tracker_staking.stake_for_account(
            self.ctx(),
            funding_account,
            account,
            token,
            amount,
        )?;
        bonus_omx_tracker_staking.stake_for_account(
            self.ctx(),
            account,
            account,
            staked_omx_tracker,
            amount,
        )?;
        fee_omx_tracker_staking.stake_for_account(
            self.ctx(),
            account,
            account,
            bonus_omx_tracker,
            amount,
        )?;

        evm::log(StakeOmx { account, amount });

        Ok(())
    }

    fn unstake_omx_internal(
        &mut self,
        account: Address,
        token: Address,
        amount: U256,
    ) -> Result<(), Vec<u8>> {
        if amount == U256::ZERO {
            return Err(RewardRouterError::InvalidAmount.into());
        }

        let staked_omx_tracker_staking =
            IRewardTrackerStaking::new(self.staked_omx_tracker_staking.get());
        let bonus_omx_tracker_staking =
            IRewardTrackerStaking::new(self.bonus_omx_tracker_staking.get());
        let fee_omx_tracker_staking =
            IRewardTrackerStaking::new(self.fee_omx_tracker_staking.get());

        let staked_omx_tracker = self.staked_omx_tracker.get();
        let bonus_omx_tracker = self.bonus_omx_tracker.get();

        let balance = staked_omx_tracker_staking.staked_amount(self.ctx(), account)?;

        fee_omx_tracker_staking.unstake_for_account(
            self.ctx(),
            account,
            bonus_omx_tracker,
            amount,
            account,
        )?;
        bonus_omx_tracker_staking.unstake_for_account(
            self.ctx(),
            account,
            staked_omx_tracker,
            amount,
            account,
        )?;
        staked_omx_tracker_staking.unstake_for_account(
            self.ctx(),
            account,
            token,
            amount,
            account,
        )?;

        let bn_omx_amount =
            bonus_omx_tracker_staking.claim_for_account(self.ctx(), account, account)?;

        let bn_omx = self.bn_omx.get();

        if bn_omx_amount > U256::ZERO {
            fee_omx_tracker_staking.stake_for_account(
                self.ctx(),
                account,
                account,
                bn_omx,
                bn_omx_amount,
            )?;
        }

        let staked_bn_omx = fee_omx_tracker_staking.deposit_balance(self.ctx(), account, bn_omx)?;
        if staked_bn_omx > U256::ZERO {
            let reduction_amount = safe_mul_ratio(staked_bn_omx, amount, balance)?;
            fee_omx_tracker_staking.unstake_for_account(
                self.ctx(),
                account,
                bn_omx,
                reduction_amount,
                account,
            )?;
            IBaseToken::new(bn_omx).burn(self.ctx(), account, reduction_amount)?;
        }

        evm::log(UnstakeOmx { account, amount });

        Ok(())
    }
}

#[external]
impl RewardRouter {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        gov: Address,
        omx: Address,
        es_omx: Address,
        bn_omx: Address,
        olp: Address,
        staked_omx_tracker: Address,
        bonus_omx_tracker: Address,
        staked_omx_tracker_staking: Address,
        bonus_omx_tracker_staking: Address,
        fee_omx_tracker_staking: Address,
        fee_olp_tracker_staking: Address,
        staked_olp_tracker_staking: Address,
        olp_manager: Address,
    ) -> Result<(), Vec<u8>> {
        if self.initialized.get() {
            return Err(RewardRouterError::AlreadyInitialized.into());
        }

        self.gov.set(gov);

        self.omx.set(omx);
        self.es_omx.set(es_omx);
        self.bn_omx.set(bn_omx);
        self.olp.set(olp);
        self.staked_omx_tracker.set(staked_omx_tracker);
        self.bonus_omx_tracker.set(bonus_omx_tracker);
        self.staked_omx_tracker_staking
            .set(staked_omx_tracker_staking);
        self.bonus_omx_tracker_staking
            .set(bonus_omx_tracker_staking);
        self.fee_omx_tracker_staking.set(fee_omx_tracker_staking);
        self.fee_olp_tracker_staking.set(fee_olp_tracker_staking);
        self.staked_olp_tracker_staking
            .set(staked_olp_tracker_staking);
        self.olp_manager.set(olp_manager);

        self.initialized.set(true);

        Ok(())
    }

    pub fn set_gov(&mut self, gov: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.gov.set(gov);

        Ok(())
    }

    pub fn batch_stake_omx_for_account(
        &mut self,
        accounts: Vec<Address>,
        amounts: Vec<U256>,
    ) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        let omx = self.omx.get();

        for (account, amount) in accounts.into_iter().zip(amounts.into_iter()) {
            self.stake_omx_internal(msg::sender(), account, omx, amount)?;
        }

        Ok(())
    }

    pub fn stake_omx_for_account(&mut self, account: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.stake_omx_internal(msg::sender(), account, self.omx.get(), amount)?;

        Ok(())
    }

    pub fn stake_omx(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.stake_omx_internal(msg::sender(), msg::sender(), self.omx.get(), amount)?;

        Ok(())
    }

    pub fn stake_es_omx(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.stake_omx_internal(msg::sender(), msg::sender(), self.es_omx.get(), amount)?;

        Ok(())
    }

    pub fn unstake_omx(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.unstake_omx_internal(msg::sender(), self.omx.get(), amount)?;

        Ok(())
    }

    pub fn unstake_es_omx(&mut self, amount: U256) -> Result<(), Vec<u8>> {
        self.unstake_omx_internal(msg::sender(), self.es_omx.get(), amount)?;

        Ok(())
    }

    pub fn mint_and_stake_olp(
        &mut self,
        token: Address,
        amount: U256,
        min_usdo: U256,
        min_olp: U256,
    ) -> Result<U256, Vec<u8>> {
        if amount == U256::ZERO {
            return Err(RewardRouterError::InvalidAmount.into());
        }

        let account = msg::sender();
        let olp = self.olp.get();

        let olp_manager = IOlpManager::new(self.olp_manager.get());
        let olp_amount = olp_manager.add_liquidity_for_account(
            self.ctx(),
            account,
            account,
            token,
            amount,
            min_usdo,
            min_olp,
        )?;

        let fee_olp_tracker = IRewardTrackerStaking::new(self.fee_olp_tracker_staking.get());
        fee_olp_tracker.stake_for_account(self.ctx(), account, account, olp, olp_amount)?;

        let staked_olp_tracker = IRewardTrackerStaking::new(self.staked_olp_tracker_staking.get());
        staked_olp_tracker.stake_for_account(
            self.ctx(),
            account,
            account,
            fee_olp_tracker.address,
            olp_amount,
        )?;

        evm::log(StakeOmx { account, amount });

        Ok(olp_amount)
    }

    pub fn unstake_and_redeem_olp(
        &mut self,
        token_out: Address,
        olp_amount: U256,
        min_out: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        if olp_amount == U256::ZERO {
            return Err(RewardRouterError::InvalidOlpAmount.into());
        }

        let fee_olp_tracker = IRewardTrackerStaking::new(self.fee_olp_tracker_staking.get());
        let staked_olp_tracker = IRewardTrackerStaking::new(self.staked_olp_tracker_staking.get());

        staked_olp_tracker.unstake_for_account(
            self.ctx(),
            msg::sender(),
            fee_olp_tracker.address,
            olp_amount,
            msg::sender(),
        )?;

        let olp = self.olp.get();
        fee_olp_tracker.unstake_for_account(
            self.ctx(),
            msg::sender(),
            olp,
            olp_amount,
            msg::sender(),
        )?;

        let olp_manager = IOlpManager::new(self.olp_manager.get());
        let amount_out = olp_manager.remove_liquidity_for_account(
            self.ctx(),
            msg::sender(),
            token_out,
            olp_amount,
            min_out,
            receiver,
        )?;

        evm::log(UnstakeOlp {
            account: msg::sender(),
            amount: olp_amount,
        });

        Ok(amount_out)
    }

    pub fn claim(&mut self) -> Result<(), Vec<u8>> {
        IRewardTrackerStaking::new(self.fee_omx_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;
        IRewardTrackerStaking::new(self.fee_olp_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;

        IRewardTrackerStaking::new(self.staked_omx_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;
        IRewardTrackerStaking::new(self.staked_olp_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;

        Ok(())
    }

    pub fn claim_es_omx(&mut self) -> Result<(), Vec<u8>> {
        IRewardTrackerStaking::new(self.staked_omx_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;
        IRewardTrackerStaking::new(self.staked_olp_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;

        Ok(())
    }

    pub fn claim_fees(&mut self) -> Result<(), Vec<u8>> {
        IRewardTrackerStaking::new(self.fee_omx_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;
        IRewardTrackerStaking::new(self.fee_olp_tracker_staking.get()).claim_for_account(
            self.ctx(),
            msg::sender(),
            msg::sender(),
        )?;

        Ok(())
    }

    pub fn compound(&mut self) -> Result<(), Vec<u8>> {
        self.compound_internal(msg::sender())?;

        Ok(())
    }

    pub fn compound_for_account(&mut self, account: Address) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        self.compound_internal(account)?;

        Ok(())
    }

    pub fn batch_compound_for_accounts(&mut self, accounts: Vec<Address>) -> Result<(), Vec<u8>> {
        self.only_gov()?;

        for account in accounts {
            self.compound_internal(account)?;
        }

        Ok(())
    }
}
