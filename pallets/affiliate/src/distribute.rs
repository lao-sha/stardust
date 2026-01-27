//! 函数级中文注释：统一分配入口
//!
//! 功能：
//! - 模式路由（Instant/Weekly/Hybrid）
//! - 系统费用扣除（Burn/Treasury/Storage）
//! - 统一分配逻辑协调
//!
//! 整合自：pallet-affiliate-config

use super::*;
use frame_support::traits::{Currency, ExistenceRequirement};
use sp_runtime::traits::{Saturating, Zero};

/// 函数级中文注释：统一分配实现
impl<T: Config> Pallet<T> {
    /// 函数级中文注释：通用分配（含系统费用）
    ///
    /// 参数：
    /// - buyer: 购买者/供奉者
    /// - gross_amount: 总金额
    /// - duration_weeks: 供奉时长（可选）
    ///
    /// 扣费项目：
    /// - 销毁：5%
    /// - 国库：2%
    /// - 存储：3%
    /// - 可分配：90%
    ///
    /// 返回：实际分配总额
    pub fn do_distribute_rewards(
        buyer: &T::AccountId,
        gross_amount: BalanceOf<T>,
        duration_weeks: Option<u32>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if gross_amount.is_zero() {
            return Ok(BalanceOf::<T>::zero());
        }

        // 扣除系统费用
        let (distributable, burn_amount, treasury_amount, storage_amount) = 
            Self::deduct_system_fees(gross_amount);

        // 销毁
        if !burn_amount.is_zero() {
            let burn_account = T::BurnAccount::get();
            T::Currency::transfer(
                buyer,
                &burn_account,
                burn_amount,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        // 国库
        if !treasury_amount.is_zero() {
            let treasury_account = T::TreasuryAccount::get();
            T::Currency::transfer(
                buyer,
                &treasury_account,
                treasury_amount,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        // 存储 - 充值到 buyer 的 UserFunding 账户
        if !storage_amount.is_zero() {
            let user_funding_account = T::UserFundingProvider::derive_user_funding_account(buyer);
            T::Currency::transfer(
                buyer,
                &user_funding_account,
                storage_amount,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        // 根据结算模式分配
        let distributed = Self::distribute_by_mode(buyer, distributable, duration_weeks)?;

        // 未分配的金额转入国库（无推荐人或推荐人无效时）
        let undistributed = distributable.saturating_sub(distributed);
        if !undistributed.is_zero() {
            let treasury_account = T::TreasuryAccount::get();
            T::Currency::transfer(
                buyer,
                &treasury_account,
                undistributed,
                ExistenceRequirement::KeepAlive,
            )?;
        }

        Ok(distributed)
    }

    /// 函数级中文注释：会员专用分配（100%推荐链）
    ///
    /// 参数：
    /// - buyer: 购买者（新会员）
    /// - amount: 会员费金额
    ///
    /// 特点：
    /// - 无系统扣费
    /// - 100%分配到推荐链
    /// - 使用即时分成模式（快速到账）
    ///
    /// 返回：实际分配总额
    pub fn do_distribute_membership_rewards(
        buyer: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if amount.is_zero() {
            return Ok(BalanceOf::<T>::zero());
        }

        // 会员费100%即时分成，无系统扣费
        let distributed = Self::do_instant_distribute(buyer, amount, 15);

        Ok(distributed)
    }

    /// 函数级中文注释：根据结算模式分配
    ///
    /// 参数：
    /// - buyer: 购买者
    /// - distributable_amount: 可分配金额
    /// - duration_weeks: 供奉时长
    ///
    /// 返回：实际分配总额
    fn distribute_by_mode(
        buyer: &T::AccountId,
        distributable_amount: BalanceOf<T>,
        duration_weeks: Option<u32>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let mode = SettlementMode::<T>::get();

        match mode {
            crate::types::SettlementMode::Weekly => {
                // 全周结算模式
                Self::do_report_consumption(buyer, distributable_amount, duration_weeks, 15);
                Ok(BalanceOf::<T>::zero()) // 周结算时立即返回0，实际结算在周末
            }

            crate::types::SettlementMode::Instant => {
                // 全即时分成模式
                let distributed = Self::do_instant_distribute(buyer, distributable_amount, 15);
                Ok(distributed)
            }

            crate::types::SettlementMode::Hybrid {
                instant_levels,
                weekly_levels,
            } => {
                // 混合模式
                let instant_levels = instant_levels.min(15);
                let weekly_levels = weekly_levels.min(15);

                // 前N层即时分成
                let instant_distributed = if instant_levels > 0 {
                    Self::do_instant_distribute(buyer, distributable_amount, instant_levels)
                } else {
                    BalanceOf::<T>::zero()
                };

                // 后M层周结算
                if weekly_levels > 0 {
                    Self::do_report_consumption(buyer, distributable_amount, duration_weeks, weekly_levels);
                }

                Ok(instant_distributed)
            }
        }
    }

    /// 函数级中文注释：扣除系统费用
    ///
    /// 参数：
    /// - gross_amount: 总金额
    ///
    /// 返回：(可分配金额, 销毁金额, 国库金额, 存储金额)
    fn deduct_system_fees(
        gross_amount: BalanceOf<T>,
    ) -> (BalanceOf<T>, BalanceOf<T>, BalanceOf<T>, BalanceOf<T>) {
        // 销毁：5%
        let burn_amount = Self::calculate_percent(gross_amount, 5);

        // 国库：2%
        let treasury_amount = Self::calculate_percent(gross_amount, 2);

        // 存储：3%
        let storage_amount = Self::calculate_percent(gross_amount, 3);

        // 可分配：90%
        let distributable = gross_amount
            .saturating_sub(burn_amount)
            .saturating_sub(treasury_amount)
            .saturating_sub(storage_amount);

        (distributable, burn_amount, treasury_amount, storage_amount)
    }

    /// 函数级中文注释：计算百分比
    fn calculate_percent(total: BalanceOf<T>, percent: u8) -> BalanceOf<T> {
        if percent == 0 || percent > 100 {
            return BalanceOf::<T>::zero();
        }

        let percent_balance: BalanceOf<T> = percent.into();
        let hundred: BalanceOf<T> = 100u32.into();

        total.saturating_mul(percent_balance) / hundred
    }
}

