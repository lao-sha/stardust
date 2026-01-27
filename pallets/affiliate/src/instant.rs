//! 函数级中文注释：即时分成子模块
//!
//! 功能：
//! - 实时转账分配
//! - 推荐链验证
//! - 立即到账
//!
//! 整合自：pallet-affiliate-instant

use super::*;
use frame_support::traits::{Currency, ExistenceRequirement};
use sp_runtime::traits::{Saturating, Zero, AccountIdConversion};
use pallet_affiliate_referral::MembershipProvider;

/// 函数级中文注释：即时分成实现
impl<T: Config> Pallet<T> {
    /// 函数级中文注释：即时分配
    ///
    /// 参数：
    /// - buyer: 购买者/供奉者
    /// - distributable_amount: 可分配金额（已扣除系统费用）
    /// - levels: 分配层数（用于 Hybrid 模式）
    ///
    /// 返回：实际分配总额
    pub fn do_instant_distribute(
        buyer: &T::AccountId,
        distributable_amount: BalanceOf<T>,
        levels: u8,
    ) -> BalanceOf<T> {
        // 获取推荐链
        let referral_chain = Self::get_referral_chain(buyer);

        // 获取即时分成比例配置
        let level_percents = InstantLevelPercents::<T>::get();

        let mut total_distributed = BalanceOf::<T>::zero();
        let levels_to_process = levels.min(15) as usize;

        // 逐层分配
        for (index, referrer) in referral_chain.iter().enumerate().take(levels_to_process) {
            // 获取该层分成比例
            let percent = if let Some(p) = level_percents.get(index) {
                *p
            } else {
                0
            };

            if percent == 0 {
                continue;
            }

            // 验证推荐人资格
            if !Self::is_valid_referrer(referrer, index as u8 + 1) {
                // 无效推荐人，份额并入国库
                continue;
            }

            // 计算分成金额
            let share = Self::calculate_share(distributable_amount, percent);

            if share.is_zero() {
                continue;
            }

            // 立即转账
            if let Err(_) = T::Currency::transfer(
                &T::EscrowPalletId::get().into_account_truncating(),
                referrer,
                share,
                ExistenceRequirement::KeepAlive,
            ) {
                // 转账失败，继续下一层
                continue;
            }

            total_distributed = total_distributed.saturating_add(share);

            // 发射事件
            Self::deposit_event(Event::InstantRewardDistributed {
                referrer: referrer.clone(),
                buyer: buyer.clone(),
                level: (index as u8) + 1,
                amount: share,
            });
        }

        // 更新累计统计
        TotalInstantDistributed::<T>::mutate(|total| {
            *total = total.saturating_add(total_distributed);
        });

        total_distributed
    }

    /// 函数级中文注释：验证推荐人资格
    ///
    /// 参数：
    /// - referrer: 推荐人账户
    /// - level: 层级（1-15）
    ///
    /// 验证：
    /// - 是否为有效会员
    /// - 可拿代数是否覆盖该层
    ///
    /// 返回：是否有效
    fn is_valid_referrer(referrer: &T::AccountId, level: u8) -> bool {
        // 验证：是否为有效会员
        if !T::MembershipProvider::is_valid_member(referrer) {
            return false;
        }

        // 验证：可拿代数（假设所有会员都可拿15层，简化实现）
        // 如需动态可拿代数，可从 MembershipProvider 获取
        if level > 15 {
            return false;
        }

        true
    }

    /// 函数级中文注释：计算分成金额
    ///
    /// 参数：
    /// - total: 总金额
    /// - percent: 百分比（0-100）
    ///
    /// 返回：分成金额
    fn calculate_share(total: BalanceOf<T>, percent: u8) -> BalanceOf<T> {
        if percent == 0 || percent > 100 {
            return BalanceOf::<T>::zero();
        }

        // 计算：total * percent / 100
        let percent_balance: BalanceOf<T> = percent.into();
        let hundred: BalanceOf<T> = 100u32.into();

        total.saturating_mul(percent_balance) / hundred
    }
}

