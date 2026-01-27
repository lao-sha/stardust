//! # 婚恋会员模块 (Matchmaking Membership)
//!
//! 婚恋平台的年费会员管理模块，用于区分免费用户和付费会员。
//!
//! ## 功能概述
//!
//! - **会员订阅**：支持月付、季付、半年付、年付和终身会员
//! - **会员续费**：支持手动续费和自动续费
//! - **会员升级**：从年费会员升级到终身会员
//! - **权益管理**：不同等级享有不同权益
//! - **使用量追踪**：追踪每日功能使用情况
//!
//! ## 会员等级
//!
//! | 等级 | 说明 |
//! |------|------|
//! | Free | 免费用户，基础功能 |
//! | Annual | 年费会员，完整功能 |
//! | Lifetime | 终身会员，永久权益 |
//!
//! ## 会员权益
//!
//! | 权益 | Free | Annual | Lifetime |
//! |------|------|--------|----------|
//! | 每日推荐数 | 10 | 50 | 100 |
//! | 超级喜欢 | 0 | 5/天 | 10/天 |
//! | 合婚分析 | 1/天 | 10/天 | 30/天 |
//! | 查看谁喜欢我 | ❌ | ✅ | ✅ |
//! | 查看访客 | ❌ | ✅ | ✅ |
//! | 隐身浏览 | ❌ | ✅ | ✅ |
//! | 优先展示 | ❌ | ✅ | ✅ |
//! | 专属客服 | ❌ | ❌ | ✅ |
//!
//! ## 费用分配
//!
//! 会员费用分配（15层推荐链）：
//! - 销毁：5%
//! - 国库：2%
//! - 存储：3%
//! - 推荐链分配：90%

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests;

pub mod types;
pub mod traits;
pub mod weights;

pub use types::*;
pub use traits::*;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::fungible::{Inspect, Mutate},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Saturating, Zero, Bounded};
    use pallet_trading_common::PricingProvider;
    use pallet_affiliate::types::AffiliateDistributor;
use pallet_affiliate::UserFundingProvider;

    /// 余额类型别名
    pub type BalanceOf<T> = <T as Config>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 权重信息
        type WeightInfo: WeightInfo;

        /// Fungible 接口：用于支付会员费
        type Fungible: Inspect<Self::AccountId, Balance = BalanceOf<Self>>
            + Mutate<Self::AccountId>;

        /// 余额类型
        type Balance: codec::FullCodec
            + codec::MaxEncodedLen
            + Copy
            + MaybeSerializeDeserialize
            + core::fmt::Debug
            + Default
            + scale_info::TypeInfo
            + Saturating
            + Zero
            + PartialOrd
            + Ord
            + TryFrom<u128>
            + TryInto<u128>;

        /// 每月区块数（约 216000 块，按 12 秒/块计算）
        #[pallet::constant]
        type BlocksPerMonth: Get<BlockNumberFor<Self>>;

        /// 每天区块数（约 7200 块）
        #[pallet::constant]
        type BlocksPerDay: Get<BlockNumberFor<Self>>;

        /// 月费兜底金额（DUST 数量，pricing 不可用时使用）
        #[pallet::constant]
        type MonthlyFee: Get<BalanceOf<Self>>;

        /// 月费 USD 价值（精度 10^6，例如 10_000_000 = 10 USDT）
        #[pallet::constant]
        type MonthlyFeeUsd: Get<u64>;

        /// 终身会员费兜底金额
        #[pallet::constant]
        type LifetimeFee: Get<BalanceOf<Self>>;

        /// 终身会员费 USD 价值（精度 10^6，例如 500_000_000 = 500 USDT）
        #[pallet::constant]
        type LifetimeFeeUsd: Get<u64>;

        /// 定价接口（用于获取 DUST/USD 汇率）
        type Pricing: PricingProvider<BalanceOf<Self>>;

        /// 国库账户
        type TreasuryAccount: Get<Self::AccountId>;

        /// 销毁账户
        type BurnAccount: Get<Self::AccountId>;

        /// 用户存储资金账户提供者
        /// 
        /// 用于将存储费用充值到用户的 UserFunding 账户
        type UserFundingProvider: pallet_affiliate::UserFundingProvider<Self::AccountId>;

        /// 联盟计酬分配器（15层推荐链分配）
        type AffiliateDistributor: pallet_affiliate::types::AffiliateDistributor<
            Self::AccountId,
            u128,
            BlockNumberFor<Self>,
        >;
    }

    // ========================================================================
    // 存储
    // ========================================================================

    /// 会员信息存储
    #[pallet::storage]
    #[pallet::getter(fn memberships)]
    pub type Memberships<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        MembershipInfo<BlockNumberFor<T>, BalanceOf<T>>,
    >;

    /// 每日使用记录
    #[pallet::storage]
    #[pallet::getter(fn daily_usage)]
    pub type DailyUsages<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        DailyUsage,
        ValueQuery,
    >;

    /// 全局统计信息
    #[pallet::storage]
    #[pallet::getter(fn global_stats)]
    pub type GlobalStats<T: Config> = StorageValue<
        _,
        MembershipStats<BalanceOf<T>>,
        ValueQuery,
    >;

    /// 订阅交易历史（最近 10 条）
    #[pallet::storage]
    pub type SubscriptionHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u32, // 索引 0-9
        SubscriptionTransaction<BlockNumberFor<T>, BalanceOf<T>>,
    >;

    /// 订阅历史索引（环形缓冲区）
    #[pallet::storage]
    pub type SubscriptionHistoryIndex<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    // ========================================================================
    // 事件
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 会员已订阅
        Subscribed {
            who: T::AccountId,
            tier: MembershipTier,
            duration_months: u32,
            amount: u128,
            expires_at: BlockNumberFor<T>,
        },
        /// 会员已续费
        Renewed {
            who: T::AccountId,
            duration_months: u32,
            amount: u128,
            new_expiry: BlockNumberFor<T>,
        },
        /// 会员已升级
        Upgraded {
            who: T::AccountId,
            old_tier: MembershipTier,
            new_tier: MembershipTier,
            amount: u128,
        },
        /// 自动续费已取消
        AutoRenewCancelled {
            who: T::AccountId,
        },
        /// 会员已过期
        Expired {
            who: T::AccountId,
            old_tier: MembershipTier,
        },
        /// 权益已使用
        BenefitUsed {
            who: T::AccountId,
            benefit_type: BenefitType,
        },
    }

    /// 权益类型（用于事件）
    #[derive(Clone, Copy, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub enum BenefitType {
        Recommendation,
        SuperLike,
        CompatibilityCheck,
    }

    // ========================================================================
    // 错误
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 已经是会员
        AlreadyMember,
        /// 不是会员
        NotAMember,
        /// 会员已过期
        MembershipExpired,
        /// 余额不足
        InsufficientBalance,
        /// 无效的订阅时长
        InvalidDuration,
        /// 无法降级
        CannotDowngrade,
        /// 已是终身会员
        AlreadyLifetime,
        /// 每日使用次数已达上限
        DailyLimitReached,
        /// 无此权益
        BenefitNotAvailable,
        /// 算术溢出
        ArithmeticOverflow,
    }

    // ========================================================================
    // Extrinsics
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 订阅会员
        ///
        /// 新用户订阅年费会员或终身会员。
        /// 费用分配：销毁 5%，国库 2%，存储 3%，推荐链 90%。
        ///
        /// # 参数
        /// - `duration`: 订阅时长
        /// - `auto_renew`: 是否自动续费（仅年费会员有效）
        /// - `referrer`: 推荐人账户（可选）
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::subscribe())]
        pub fn subscribe(
            origin: OriginFor<T>,
            duration: SubscriptionDuration,
            auto_renew: bool,
            referrer: Option<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查是否已是有效会员
            if let Some(membership) = Memberships::<T>::get(&who) {
                let now = frame_system::Pallet::<T>::block_number();
                if membership.expires_at > now && membership.tier != MembershipTier::Free {
                    return Err(Error::<T>::AlreadyMember.into());
                }
            }

            // 计算费用
            let (tier, fee, duration_months) = Self::calculate_subscription_fee(duration)?;

            // 检查余额
            let balance = T::Fungible::balance(&who);
            ensure!(balance >= fee, Error::<T>::InsufficientBalance);

            // 计算到期时间
            let now = frame_system::Pallet::<T>::block_number();
            let expires_at = if tier == MembershipTier::Lifetime {
                BlockNumberFor::<T>::max_value()
            } else {
                let blocks = T::BlocksPerMonth::get()
                    .saturating_mul(duration_months.into());
                now.saturating_add(blocks)
            };

            // 分配费用
            Self::distribute_fee(&who, fee, referrer.clone())?;

            // 创建会员信息
            let referrer_bytes = referrer.as_ref().map(|r| {
                let encoded = r.encode();
                let mut bytes = [0u8; 32];
                let len = encoded.len().min(32);
                bytes[..len].copy_from_slice(&encoded[..len]);
                bytes
            });

            let membership = MembershipInfo {
                tier,
                subscribed_at: now,
                expires_at,
                total_paid: fee,
                auto_renew: auto_renew && tier != MembershipTier::Lifetime,
                consecutive_months: duration_months,
                referrer: referrer_bytes,
            };

            Memberships::<T>::insert(&who, membership);

            // 更新统计
            GlobalStats::<T>::mutate(|stats| {
                match tier {
                    MembershipTier::Annual => stats.annual_count = stats.annual_count.saturating_add(1),
                    MembershipTier::Lifetime => stats.lifetime_count = stats.lifetime_count.saturating_add(1),
                    _ => {}
                }
                stats.total_revenue = stats.total_revenue.saturating_add(fee);
            });

            // 记录交易历史
            Self::record_transaction(&who, SubscriptionTxType::NewSubscription, fee, duration_months);

            Self::deposit_event(Event::Subscribed {
                who,
                tier,
                duration_months,
                amount: fee.try_into().unwrap_or(0u128),
                expires_at,
            });

            Ok(())
        }

        /// 续费会员
        ///
        /// 年费会员续费，延长会员有效期。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::renew())]
        pub fn renew(
            origin: OriginFor<T>,
            duration: SubscriptionDuration,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 终身会员不需要续费
            ensure!(duration != SubscriptionDuration::Lifetime, Error::<T>::AlreadyLifetime);

            let mut membership = Memberships::<T>::get(&who)
                .ok_or(Error::<T>::NotAMember)?;

            // 终身会员不需要续费
            ensure!(membership.tier != MembershipTier::Lifetime, Error::<T>::AlreadyLifetime);

            // 计算费用
            let (_, fee, duration_months) = Self::calculate_subscription_fee(duration)?;

            // 检查余额
            let balance = T::Fungible::balance(&who);
            ensure!(balance >= fee, Error::<T>::InsufficientBalance);

            // 分配费用（续费使用原推荐人）
            let referrer = membership.referrer.map(|bytes| {
                T::AccountId::decode(&mut &bytes[..]).ok()
            }).flatten();
            Self::distribute_fee(&who, fee, referrer)?;

            // 更新会员信息
            let now = frame_system::Pallet::<T>::block_number();
            let base_time = if membership.expires_at > now {
                membership.expires_at
            } else {
                now
            };
            let blocks = T::BlocksPerMonth::get()
                .saturating_mul(duration_months.into());
            membership.expires_at = base_time.saturating_add(blocks);
            membership.total_paid = membership.total_paid.saturating_add(fee);
            membership.consecutive_months = membership.consecutive_months.saturating_add(duration_months);
            membership.tier = MembershipTier::Annual;

            Memberships::<T>::insert(&who, membership.clone());

            // 记录交易历史
            Self::record_transaction(&who, SubscriptionTxType::Renewal, fee, duration_months);

            Self::deposit_event(Event::Renewed {
                who,
                duration_months,
                amount: fee.try_into().unwrap_or(0u128),
                new_expiry: membership.expires_at,
            });

            Ok(())
        }

        /// 升级到终身会员
        ///
        /// 年费会员可以升级到终身会员，支付差价。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::upgrade())]
        pub fn upgrade(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut membership = Memberships::<T>::get(&who)
                .ok_or(Error::<T>::NotAMember)?;

            // 检查当前等级
            ensure!(membership.tier != MembershipTier::Lifetime, Error::<T>::AlreadyLifetime);

            let now = frame_system::Pallet::<T>::block_number();

            // 计算终身会员费用
            let lifetime_fee = Self::calculate_lifetime_fee();

            // 计算剩余年费会员价值（如果还在有效期内）
            let credit = if membership.tier == MembershipTier::Annual && membership.expires_at > now {
                let remaining_blocks: u128 = membership.expires_at
                    .saturating_sub(now)
                    .try_into()
                    .unwrap_or(0);
                let blocks_per_month: u128 = T::BlocksPerMonth::get()
                    .try_into()
                    .unwrap_or(216000);
                let monthly_fee = Self::calculate_monthly_fee();
                let monthly_fee_u128: u128 = monthly_fee.try_into().unwrap_or(0);
                
                // credit = (remaining_blocks / blocks_per_month) * monthly_fee
                let credit_u128 = remaining_blocks
                    .saturating_mul(monthly_fee_u128)
                    / blocks_per_month;
                credit_u128.try_into().unwrap_or(BalanceOf::<T>::zero())
            } else {
                BalanceOf::<T>::zero()
            };

            // 需要支付的差价
            let upgrade_fee = lifetime_fee.saturating_sub(credit);

            // 检查余额
            let balance = T::Fungible::balance(&who);
            ensure!(balance >= upgrade_fee, Error::<T>::InsufficientBalance);

            // 分配费用
            let referrer = membership.referrer.map(|bytes| {
                T::AccountId::decode(&mut &bytes[..]).ok()
            }).flatten();
            Self::distribute_fee(&who, upgrade_fee, referrer)?;

            // 更新会员信息
            let old_tier = membership.tier;
            membership.tier = MembershipTier::Lifetime;
            membership.expires_at = BlockNumberFor::<T>::max_value();
            membership.total_paid = membership.total_paid.saturating_add(upgrade_fee);
            membership.auto_renew = false;

            Memberships::<T>::insert(&who, membership);

            // 更新统计
            GlobalStats::<T>::mutate(|stats| {
                if old_tier == MembershipTier::Annual {
                    stats.annual_count = stats.annual_count.saturating_sub(1);
                }
                stats.lifetime_count = stats.lifetime_count.saturating_add(1);
                stats.total_revenue = stats.total_revenue.saturating_add(upgrade_fee);
            });

            // 记录交易历史
            Self::record_transaction(&who, SubscriptionTxType::Upgrade, upgrade_fee, 0);

            Self::deposit_event(Event::Upgraded {
                who,
                old_tier,
                new_tier: MembershipTier::Lifetime,
                amount: upgrade_fee.try_into().unwrap_or(0u128),
            });

            Ok(())
        }

        /// 取消自动续费
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cancel_auto_renew())]
        pub fn cancel_auto_renew(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Memberships::<T>::try_mutate(&who, |maybe_membership| {
                let membership = maybe_membership.as_mut()
                    .ok_or(Error::<T>::NotAMember)?;
                
                membership.auto_renew = false;
                
                Self::deposit_event(Event::AutoRenewCancelled { who: who.clone() });
                
                Ok(())
            })
        }

        /// 使用权益
        ///
        /// 记录会员使用特定权益，用于每日限额追踪。
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::use_benefit())]
        pub fn use_benefit(
            origin: OriginFor<T>,
            benefit_type: BenefitType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取会员等级和权益
            let tier = Self::get_tier(&who);
            let benefits = MembershipBenefits::for_tier(tier);

            // 获取当前日期索引
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);

            // 获取或重置每日使用记录
            let mut usage = DailyUsages::<T>::get(&who);
            if usage.day_index != today {
                usage = DailyUsage {
                    day_index: today,
                    recommendations_used: 0,
                    super_likes_used: 0,
                    compatibility_checks_used: 0,
                };
            }

            // 检查并更新使用量
            match benefit_type {
                BenefitType::Recommendation => {
                    ensure!(
                        usage.recommendations_used < benefits.daily_recommendations,
                        Error::<T>::DailyLimitReached
                    );
                    usage.recommendations_used = usage.recommendations_used.saturating_add(1);
                }
                BenefitType::SuperLike => {
                    ensure!(
                        benefits.daily_super_likes > 0,
                        Error::<T>::BenefitNotAvailable
                    );
                    ensure!(
                        usage.super_likes_used < benefits.daily_super_likes,
                        Error::<T>::DailyLimitReached
                    );
                    usage.super_likes_used = usage.super_likes_used.saturating_add(1);
                }
                BenefitType::CompatibilityCheck => {
                    ensure!(
                        usage.compatibility_checks_used < benefits.daily_compatibility_checks,
                        Error::<T>::DailyLimitReached
                    );
                    usage.compatibility_checks_used = usage.compatibility_checks_used.saturating_add(1);
                }
            }

            DailyUsages::<T>::insert(&who, usage);

            Self::deposit_event(Event::BenefitUsed {
                who,
                benefit_type,
            });

            Ok(())
        }
    }

    // ========================================================================
    // 辅助函数
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 计算订阅费用
        fn calculate_subscription_fee(
            duration: SubscriptionDuration,
        ) -> Result<(MembershipTier, BalanceOf<T>, u32), DispatchError> {
            let months = duration.months();
            
            if duration == SubscriptionDuration::Lifetime {
                let fee = Self::calculate_lifetime_fee();
                return Ok((MembershipTier::Lifetime, fee, 0));
            }

            let monthly_fee = Self::calculate_monthly_fee();
            let discount_rate = duration.discount_rate();
            
            // total = monthly_fee * months * discount_rate / 10000
            let monthly_u128: u128 = monthly_fee.try_into().unwrap_or(0);
            let total_u128 = monthly_u128
                .saturating_mul(months as u128)
                .saturating_mul(discount_rate as u128)
                / 10000u128;
            
            let fee = total_u128.try_into()
                .map_err(|_| Error::<T>::ArithmeticOverflow)?;

            Ok((MembershipTier::Annual, fee, months))
        }

        /// 计算月费金额
        /// 
        /// 使用 DUST/USD 汇率计算，如果汇率不可用则使用兜底值
        fn calculate_monthly_fee() -> BalanceOf<T> {
            let usd_value = T::MonthlyFeeUsd::get(); // 精度 10^6
            if let Some(rate) = T::Pricing::get_dust_to_usd_rate() {
                // rate 是 DUST/USD 汇率（精度 10^6），表示 1 DUST = rate/10^6 USD
                // usd_value 是 USD 金额（精度 10^6）
                // dust_amount = usd_value * 10^12 / rate
                let rate_u128: u128 = rate.try_into().unwrap_or(1_000_000);
                if rate_u128 > 0 {
                    let dust_amount = (usd_value as u128)
                        .saturating_mul(1_000_000_000_000u128) // 10^12 (DUST 精度)
                        / rate_u128;
                    return dust_amount.try_into().unwrap_or(T::MonthlyFee::get());
                }
            }
            T::MonthlyFee::get()
        }

        /// 计算终身会员费
        fn calculate_lifetime_fee() -> BalanceOf<T> {
            let usd_value = T::LifetimeFeeUsd::get();
            if let Some(rate) = T::Pricing::get_dust_to_usd_rate() {
                let rate_u128: u128 = rate.try_into().unwrap_or(1_000_000);
                if rate_u128 > 0 {
                    let dust_amount = (usd_value as u128)
                        .saturating_mul(1_000_000_000_000u128)
                        / rate_u128;
                    return dust_amount.try_into().unwrap_or(T::LifetimeFee::get());
                }
            }
            T::LifetimeFee::get()
        }

        /// 分配费用
        fn distribute_fee(
            who: &T::AccountId,
            fee: BalanceOf<T>,
            referrer: Option<T::AccountId>,
        ) -> DispatchResult {
            // 销毁：5%
            let burn_amount = Self::calculate_percent(fee, 5);
            // 国库：2%
            let treasury_amount = Self::calculate_percent(fee, 2);
            // 存储：3%
            let storage_amount = Self::calculate_percent(fee, 3);
            // 可分配：90%
            let distributable = fee
                .saturating_sub(burn_amount)
                .saturating_sub(treasury_amount)
                .saturating_sub(storage_amount);

            // 销毁
            if !burn_amount.is_zero() {
                let burn_account = T::BurnAccount::get();
                T::Fungible::transfer(
                    who,
                    &burn_account,
                    burn_amount,
                    frame_support::traits::tokens::Preservation::Preserve,
                )?;
            }

            // 国库
            if !treasury_amount.is_zero() {
                let treasury = T::TreasuryAccount::get();
                T::Fungible::transfer(
                    who,
                    &treasury,
                    treasury_amount,
                    frame_support::traits::tokens::Preservation::Preserve,
                )?;
            }

            // 存储 - 充值到用户的 UserFunding 账户
            if !storage_amount.is_zero() {
                let user_funding_account = T::UserFundingProvider::derive_user_funding_account(who);
                T::Fungible::transfer(
                    who,
                    &user_funding_account,
                    storage_amount,
                    frame_support::traits::tokens::Preservation::Preserve,
                )?;
            }

            // 推荐链分配
            if !distributable.is_zero() {
                if let Some(ref referrer_account) = referrer {
                    let distributable_u128: u128 = distributable.try_into().unwrap_or(0);
                    let _ = T::AffiliateDistributor::distribute_rewards(
                        referrer_account,
                        distributable_u128,
                        None,
                    );
                } else {
                    // 无推荐人，全部转入国库
                    let treasury = T::TreasuryAccount::get();
                    T::Fungible::transfer(
                        who,
                        &treasury,
                        distributable,
                        frame_support::traits::tokens::Preservation::Preserve,
                    )?;
                }
            }

            Ok(())
        }

        /// 计算百分比
        fn calculate_percent(amount: BalanceOf<T>, percent: u32) -> BalanceOf<T> {
            let amount_u128: u128 = amount.try_into().unwrap_or(0);
            let result = amount_u128.saturating_mul(percent as u128) / 100u128;
            result.try_into().unwrap_or(BalanceOf::<T>::zero())
        }

        /// 获取用户会员等级
        pub fn get_tier(who: &T::AccountId) -> MembershipTier {
            if let Some(membership) = Memberships::<T>::get(who) {
                let now = frame_system::Pallet::<T>::block_number();
                if membership.expires_at > now {
                    return membership.tier;
                }
            }
            MembershipTier::Free
        }

        /// 区块号转换为天数索引
        fn block_to_day(block: BlockNumberFor<T>) -> u32 {
            let blocks_per_day: u32 = T::BlocksPerDay::get()
                .try_into()
                .unwrap_or(7200);
            let block_num: u32 = block.try_into().unwrap_or(0);
            block_num / blocks_per_day
        }

        /// 记录交易历史
        fn record_transaction(
            who: &T::AccountId,
            tx_type: SubscriptionTxType,
            amount: BalanceOf<T>,
            duration_months: u32,
        ) {
            let now = frame_system::Pallet::<T>::block_number();
            let tx = SubscriptionTransaction {
                tx_type,
                amount,
                duration_months,
                timestamp: now,
            };

            let index = SubscriptionHistoryIndex::<T>::get(who);
            SubscriptionHistory::<T>::insert(who, index, tx);
            SubscriptionHistoryIndex::<T>::insert(who, (index + 1) % 10);
        }
    }

    // ========================================================================
    // Trait 实现
    // ========================================================================

    impl<T: Config> MembershipProvider<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
        fn get_tier(who: &T::AccountId) -> MembershipTier {
            Self::get_tier(who)
        }

        fn is_active_member(who: &T::AccountId) -> bool {
            let tier = Self::get_tier(who);
            tier != MembershipTier::Free
        }

        fn get_expiry(who: &T::AccountId) -> Option<BlockNumberFor<T>> {
            Memberships::<T>::get(who).map(|m| m.expires_at)
        }

        fn get_benefits(who: &T::AccountId) -> MembershipBenefits {
            let tier = Self::get_tier(who);
            MembershipBenefits::for_tier(tier)
        }

        fn has_benefit(who: &T::AccountId, benefit: MembershipBenefit) -> bool {
            let benefits = Self::get_benefits(who);
            match benefit {
                MembershipBenefit::SeeWhoLikesMe => benefits.can_see_who_likes_me,
                MembershipBenefit::SeeVisitors => benefits.can_see_visitors,
                MembershipBenefit::BrowseInvisibly => benefits.can_browse_invisibly,
                MembershipBenefit::PriorityDisplay => benefits.priority_display,
                MembershipBenefit::DedicatedSupport => benefits.dedicated_support,
                MembershipBenefit::ReadReceipts => benefits.read_receipts,
                MembershipBenefit::AdvancedFilters => benefits.advanced_filters,
            }
        }
    }

    impl<T: Config> MembershipUsageTracker<T::AccountId> for Pallet<T> {
        fn can_use_recommendation(who: &T::AccountId) -> bool {
            let tier = Self::get_tier(who);
            let benefits = MembershipBenefits::for_tier(tier);
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);
            let usage = DailyUsages::<T>::get(who);
            
            if usage.day_index != today {
                return true;
            }
            usage.recommendations_used < benefits.daily_recommendations
        }

        fn can_use_super_like(who: &T::AccountId) -> bool {
            let tier = Self::get_tier(who);
            let benefits = MembershipBenefits::for_tier(tier);
            
            if benefits.daily_super_likes == 0 {
                return false;
            }
            
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);
            let usage = DailyUsages::<T>::get(who);
            
            if usage.day_index != today {
                return true;
            }
            usage.super_likes_used < benefits.daily_super_likes
        }

        fn can_use_compatibility_check(who: &T::AccountId) -> bool {
            let tier = Self::get_tier(who);
            let benefits = MembershipBenefits::for_tier(tier);
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);
            let usage = DailyUsages::<T>::get(who);
            
            if usage.day_index != today {
                return true;
            }
            usage.compatibility_checks_used < benefits.daily_compatibility_checks
        }

        fn record_recommendation_usage(who: &T::AccountId) -> Result<(), &'static str> {
            if !Self::can_use_recommendation(who) {
                return Err("Daily limit reached");
            }
            
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);
            
            DailyUsages::<T>::mutate(who, |usage| {
                if usage.day_index != today {
                    *usage = DailyUsage {
                        day_index: today,
                        recommendations_used: 0,
                        super_likes_used: 0,
                        compatibility_checks_used: 0,
                    };
                }
                usage.recommendations_used = usage.recommendations_used.saturating_add(1);
            });
            
            Ok(())
        }

        fn record_super_like_usage(who: &T::AccountId) -> Result<(), &'static str> {
            if !Self::can_use_super_like(who) {
                return Err("Daily limit reached or benefit not available");
            }
            
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);
            
            DailyUsages::<T>::mutate(who, |usage| {
                if usage.day_index != today {
                    *usage = DailyUsage {
                        day_index: today,
                        recommendations_used: 0,
                        super_likes_used: 0,
                        compatibility_checks_used: 0,
                    };
                }
                usage.super_likes_used = usage.super_likes_used.saturating_add(1);
            });
            
            Ok(())
        }

        fn record_compatibility_check_usage(who: &T::AccountId) -> Result<(), &'static str> {
            if !Self::can_use_compatibility_check(who) {
                return Err("Daily limit reached");
            }
            
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);
            
            DailyUsages::<T>::mutate(who, |usage| {
                if usage.day_index != today {
                    *usage = DailyUsage {
                        day_index: today,
                        recommendations_used: 0,
                        super_likes_used: 0,
                        compatibility_checks_used: 0,
                    };
                }
                usage.compatibility_checks_used = usage.compatibility_checks_used.saturating_add(1);
            });
            
            Ok(())
        }
    }
}
