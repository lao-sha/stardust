//! # Membership Pallet
//!
//! A pallet for managing membership subscriptions, DUST rewards, and member profiles
//! for the Stardust divination platform.
//!
//! ## Overview
//!
//! This pallet provides:
//! - **Subscription Management**: 6-tier membership system (Free, Bronze, Silver, Gold, Platinum, Diamond)
//! - **DUST Rewards**: Token rewards for user activities with anti-abuse mechanisms
//! - **Member Profiles**: Partially encrypted profile storage for divination auto-fill
//! - **Check-in System**: Daily check-in with streak bonuses
//!
//! ## Features
//!
//! - Subscribe to membership tiers with monthly/yearly billing
//! - Upgrade/downgrade membership tiers
//! - Receive DUST rewards for activities (AI interpretation, reviews, etc.)
//! - Daily check-in with consecutive day bonuses
//! - Store member profiles with partial encryption (birth info in plaintext for divination)
//!
//! ## Anti-Abuse Mechanisms
//!
//! - 7-day cooldown period for new accounts
//! - Minimum balance requirement (≥1 DUST) for rewards
//! - Daily reward limits per category
//! - Dynamic reward adjustment based on reward pool balance

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

mod types;
pub use types::*;

mod traits;
pub use traits::*;

mod impl_traits;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency, Get},
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero, CheckedDiv},
    };
    use sp_std::vec::Vec;
    use pallet_trading_common::PricingProvider;
    use pallet_affiliate::UserFundingProvider;
    use pallet_affiliate::types::AffiliateDistributor;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency mechanism for payments and rewards.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Weight information for extrinsics.
        type WeightInfo: WeightInfo;

        /// The pallet's ID, used for deriving the reward pool account.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Treasury account for receiving membership fees.
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// 销毁账户
        type BurnAccount: Get<Self::AccountId>;

        /// 用户存储资金账户提供者
        type UserFundingProvider: pallet_affiliate::UserFundingProvider<Self::AccountId>;

        /// 联盟计酬分配器（15层推荐链分配）
        type AffiliateDistributor: pallet_affiliate::types::AffiliateDistributor<
            Self::AccountId,
            u128,
            BlockNumberFor<Self>,
        >;

        /// Percentage of membership fees allocated to reward pool (in basis points, e.g., 1000 = 10%).
        #[pallet::constant]
        type RewardPoolAllocation: Get<u32>;

        /// New account cooldown period in blocks (7 days at 12s/block ≈ 50400 blocks).
        #[pallet::constant]
        type NewAccountCooldown: Get<BlockNumberFor<Self>>;

        /// Minimum balance required to receive rewards (1 DUST).
        #[pallet::constant]
        type MinBalanceForRewards: Get<BalanceOf<Self>>;

        /// Blocks per day (for check-in calculation, ~7200 at 12s/block).
        #[pallet::constant]
        type BlocksPerDay: Get<BlockNumberFor<Self>>;

        /// Blocks per month (for subscription, ~216000 at 12s/block).
        #[pallet::constant]
        type BlocksPerMonth: Get<BlockNumberFor<Self>>;

        /// Maximum display name length.
        #[pallet::constant]
        type MaxDisplayNameLength: Get<u32>;

        /// Maximum encrypted data length.
        #[pallet::constant]
        type MaxEncryptedDataLength: Get<u32>;

        /// Maximum reward history entries per user (ring buffer size).
        #[pallet::constant]
        type MaxRewardHistorySize: Get<u32>;

        /// 定价接口（用于 USDT 到 DUST 换算）
        type Pricing: pallet_trading_common::PricingProvider<BalanceOf<Self>>;
    }

    // ============ Storage Items ============

    /// Member subscription information.
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        MemberInfo<BlockNumberFor<T>, BalanceOf<T>>,
        OptionQuery,
    >;

    /// Member profiles (partially encrypted).
    #[pallet::storage]
    #[pallet::getter(fn member_profiles)]
    pub type MemberProfiles<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        MemberProfile<BlockNumberFor<T>, T::MaxDisplayNameLength, T::MaxEncryptedDataLength>,
        OptionQuery,
    >;

    /// DUST reward balances and statistics.
    #[pallet::storage]
    #[pallet::getter(fn reward_balances)]
    pub type RewardBalances<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        RewardBalance<BlockNumberFor<T>, BalanceOf<T>>,
        ValueQuery,
    >;

    /// Reward transaction history (ring buffer, last N entries).
    #[pallet::storage]
    pub type RewardHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u32, // Index (0 to MaxRewardHistorySize-1)
        RewardTransaction<BlockNumberFor<T>, BalanceOf<T>>,
        OptionQuery,
    >;

    /// Current index in the reward history ring buffer.
    #[pallet::storage]
    pub type RewardHistoryIndex<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// Daily check-in records.
    #[pallet::storage]
    #[pallet::getter(fn check_in_records)]
    pub type CheckInRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        CheckInRecord,
        ValueQuery,
    >;

    /// Account creation block (for cooldown tracking).
    #[pallet::storage]
    pub type AccountCreationBlock<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// Monthly free AI usage tracking (account, month_index) -> used_count.
    #[pallet::storage]
    pub type MonthlyFreeAiUsage<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u32, // Month index since genesis
        u32, // Used count
        ValueQuery,
    >;

    /// Global membership statistics.
    #[pallet::storage]
    #[pallet::getter(fn global_stats)]
    pub type GlobalStats<T: Config> = StorageValue<
        _,
        MembershipStats<BalanceOf<T>>,
        ValueQuery,
    >;

    /// Total rewards distributed in the current month (for budget tracking).
    #[pallet::storage]
    pub type MonthlyRewardsDistributed<T: Config> = StorageMap<
        _,
        Twox64Concat,
        u32, // Month index
        BalanceOf<T>,
        ValueQuery,
    >;

    // ============ Events ============

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user subscribed to a membership tier.
        Subscribed {
            who: T::AccountId,
            tier: MemberTier,
            duration: SubscriptionDuration,
            amount_paid: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// A user upgraded their membership tier.
        TierUpgraded {
            who: T::AccountId,
            old_tier: MemberTier,
            new_tier: MemberTier,
            amount_paid: BalanceOf<T>,
        },
        /// A user cancelled auto-renewal.
        SubscriptionCancelled {
            who: T::AccountId,
            expires_at: BlockNumberFor<T>,
        },
        /// A membership expired.
        MembershipExpired {
            who: T::AccountId,
            old_tier: MemberTier,
        },
        /// A user checked in.
        CheckedIn {
            who: T::AccountId,
            streak: u32,
            reward: BalanceOf<T>,
        },
        /// DUST reward granted.
        RewardGranted {
            who: T::AccountId,
            amount: BalanceOf<T>,
            tx_type: RewardTxType,
        },
        /// Member profile updated.
        ProfileUpdated {
            who: T::AccountId,
        },
        /// Sensitive profile data cleared.
        SensitiveDataCleared {
            who: T::AccountId,
        },
        /// Provider status applied.
        ProviderApplied {
            who: T::AccountId,
        },
        /// Provider verified by admin.
        ProviderVerified {
            provider: T::AccountId,
            verified: bool,
        },
        /// Reward pool adjustment factor changed.
        RewardPoolAdjusted {
            factor: u32, // Basis points (10000 = 100%)
        },
    }

    // ============ Errors ============

    #[pallet::error]
    pub enum Error<T> {
        /// Already subscribed to a tier.
        AlreadySubscribed,
        /// Not a member or membership expired.
        NotAMember,
        /// Cannot downgrade tier (use cancel and re-subscribe).
        CannotDowngrade,
        /// Same tier selected.
        SameTier,
        /// Insufficient balance for subscription.
        InsufficientBalance,
        /// Account is in cooldown period.
        AccountInCooldown,
        /// Balance too low to receive rewards.
        BalanceTooLow,
        /// Daily reward limit exceeded.
        DailyLimitExceeded,
        /// Already checked in today.
        AlreadyCheckedIn,
        /// Reward pool is empty.
        RewardPoolEmpty,
        /// Invalid tier for operation.
        InvalidTier,
        /// Profile not found.
        ProfileNotFound,
        /// Display name too long.
        DisplayNameTooLong,
        /// Invalid encrypted data.
        InvalidEncryptedData,
        /// Unsupported encryption version.
        UnsupportedEncryptionVersion,
        /// Not authorized for this operation.
        NotAuthorized,
        /// Provider already applied.
        AlreadyProvider,
        /// Monthly free AI quota exceeded.
        FreeAiQuotaExceeded,
        /// Arithmetic overflow.
        ArithmeticOverflow,
    }

    // ============ Hooks ============

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }
    }

    // ============ Extrinsics ============

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Subscribe to a membership tier.
        ///
        /// # Arguments
        /// * `tier` - The membership tier to subscribe to (cannot be Free).
        /// * `duration` - Monthly or Yearly subscription.
        /// * `auto_renew` - Whether to auto-renew (currently disabled for regulatory compliance).
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::subscribe())]
        pub fn subscribe(
            origin: OriginFor<T>,
            tier: MemberTier,
            duration: SubscriptionDuration,
            _auto_renew: bool, // Ignored for now (regulatory compliance)
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Cannot subscribe to Free tier
            ensure!(tier != MemberTier::Free, Error::<T>::InvalidTier);

            // Check if already subscribed to a paid tier
            if let Some(member) = Members::<T>::get(&who) {
                let now = frame_system::Pallet::<T>::block_number();
                if member.tier != MemberTier::Free && member.expires_at > now {
                    return Err(Error::<T>::AlreadySubscribed.into());
                }
            }

            // Calculate fee
            let fee = Self::calculate_subscription_fee(tier, duration);

            // Check balance
            ensure!(
                T::Currency::free_balance(&who) >= fee,
                Error::<T>::InsufficientBalance
            );

            // Calculate expiration
            let now = frame_system::Pallet::<T>::block_number();
            let duration_blocks = match duration {
                SubscriptionDuration::Monthly => T::BlocksPerMonth::get(),
                SubscriptionDuration::Yearly => T::BlocksPerMonth::get().saturating_mul(12u32.into()),
            };
            let expires_at = now.saturating_add(duration_blocks);

            // 费用分配：销毁 5%，国库 2%，存储 3%，推荐链 90%
            Self::distribute_fee(&who, fee)?;

            // Create/update member info
            let member_info = MemberInfo {
                tier,
                expires_at,
                subscribed_at: now,
                total_paid: Members::<T>::get(&who)
                    .map(|m| m.total_paid)
                    .unwrap_or_default()
                    .saturating_add(fee),
                auto_renew: false, // Disabled for regulatory compliance
            };

            Members::<T>::insert(&who, member_info);

            // Track first activity for cooldown
            if AccountCreationBlock::<T>::get(&who).is_none() {
                AccountCreationBlock::<T>::insert(&who, now);
            }

            // Update global stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_revenue = stats.total_revenue.saturating_add(fee);
                let tier_idx = tier as usize;
                if tier_idx < stats.tier_counts.len() {
                    stats.tier_counts[tier_idx] = stats.tier_counts[tier_idx].saturating_add(1);
                }
            });

            Self::deposit_event(Event::Subscribed {
                who,
                tier,
                duration,
                amount_paid: fee,
                expires_at,
            });

            Ok(())
        }

        /// Upgrade to a higher membership tier.
        ///
        /// Pays the prorated difference for the remaining subscription period.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::upgrade_tier())]
        pub fn upgrade_tier(
            origin: OriginFor<T>,
            new_tier: MemberTier,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let member = Members::<T>::get(&who).ok_or(Error::<T>::NotAMember)?;
            let now = frame_system::Pallet::<T>::block_number();

            // Check membership is active
            ensure!(member.expires_at > now, Error::<T>::NotAMember);

            // Cannot downgrade or stay same
            ensure!(new_tier as u8 > member.tier as u8, Error::<T>::CannotDowngrade);

            // Calculate prorated upgrade cost
            let remaining_blocks: u32 = member.expires_at.saturating_sub(now).try_into().unwrap_or(0);
            let blocks_per_month: u32 = T::BlocksPerMonth::get().try_into().unwrap_or(216000);
            let old_monthly = Self::get_tier_monthly_fee(member.tier);
            let new_monthly = Self::get_tier_monthly_fee(new_tier);
            let monthly_diff = new_monthly.saturating_sub(old_monthly);

            // Prorated cost = (monthly_diff * remaining_blocks) / blocks_per_month
            let upgrade_cost = monthly_diff
                .saturating_mul(remaining_blocks.into())
                / blocks_per_month.into();

            // Transfer upgrade cost
            ensure!(
                T::Currency::free_balance(&who) >= upgrade_cost,
                Error::<T>::InsufficientBalance
            );

            // 费用分配：销毁 5%，国库 2%，存储 3%，推荐链 90%
            Self::distribute_fee(&who, upgrade_cost)?;

            // Update tier (keep same expiration)
            let old_tier = member.tier;
            Members::<T>::mutate(&who, |m| {
                if let Some(member) = m {
                    member.tier = new_tier;
                    member.total_paid = member.total_paid.saturating_add(upgrade_cost);
                }
            });

            // Update global stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_revenue = stats.total_revenue.saturating_add(upgrade_cost);
                let old_idx = old_tier as usize;
                let new_idx = new_tier as usize;
                if old_idx < stats.tier_counts.len() && stats.tier_counts[old_idx] > 0 {
                    stats.tier_counts[old_idx] = stats.tier_counts[old_idx].saturating_sub(1);
                }
                if new_idx < stats.tier_counts.len() {
                    stats.tier_counts[new_idx] = stats.tier_counts[new_idx].saturating_add(1);
                }
            });

            Self::deposit_event(Event::TierUpgraded {
                who,
                old_tier,
                new_tier,
                amount_paid: upgrade_cost,
            });

            Ok(())
        }

        /// Cancel auto-renewal (membership remains active until expiration).
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::cancel_subscription())]
        pub fn cancel_subscription(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let member = Members::<T>::get(&who).ok_or(Error::<T>::NotAMember)?;

            Members::<T>::mutate(&who, |m| {
                if let Some(member) = m {
                    member.auto_renew = false;
                }
            });

            Self::deposit_event(Event::SubscriptionCancelled {
                who,
                expires_at: member.expires_at,
            });

            Ok(())
        }

        /// Daily check-in to earn DUST rewards.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::check_in())]
        pub fn check_in(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Check cooldown
            ensure!(Self::is_past_cooldown(&who), Error::<T>::AccountInCooldown);

            // Check minimum balance
            ensure!(
                T::Currency::free_balance(&who) >= T::MinBalanceForRewards::get(),
                Error::<T>::BalanceTooLow
            );

            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);

            let mut record = CheckInRecords::<T>::get(&who);

            // Check if already checked in today
            ensure!(record.last_check_in_day != today, Error::<T>::AlreadyCheckedIn);

            // Update streak
            let yesterday = today.saturating_sub(1);
            if record.last_check_in_day == yesterday {
                record.streak = record.streak.saturating_add(1);
            } else {
                record.streak = 1;
            }

            record.last_check_in_day = today;
            record.total_days = record.total_days.saturating_add(1);

            // Calculate week bitmap (bit 0-6 = Mon-Sun)
            let day_of_week = (today % 7) as u8;
            if day_of_week == 0 {
                // Monday, reset week bitmap
                record.this_week = 1;
            } else {
                record.this_week |= 1 << day_of_week;
            }

            CheckInRecords::<T>::insert(&who, record.clone());

            // Calculate and grant reward
            let base_reward = Self::get_check_in_base_reward();
            let streak_multiplier = if record.streak >= 7 { 15000u32 } else { 10000u32 }; // 1.5x for 7+ days
            let reward_with_streak = base_reward
                .saturating_mul(streak_multiplier.into())
                / 10000u32.into();

            let actual_reward = Self::do_grant_reward(
                &who,
                reward_with_streak,
                RewardTxType::CheckIn,
                b"daily_check_in",
            )?;

            Self::deposit_event(Event::CheckedIn {
                who,
                streak: record.streak,
                reward: actual_reward,
            });

            Ok(())
        }

        /// Update member profile.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_profile())]
        pub fn update_profile(
            origin: OriginFor<T>,
            display_name: BoundedVec<u8, T::MaxDisplayNameLength>,
            gender: Option<Gender>,
            birth_date: Option<BirthDate>,
            birth_hour: Option<u8>,
            longitude: Option<i32>,
            latitude: Option<i32>,
            // Encrypted sensitive data as raw components (ciphertext, nonce, version)
            encrypted_sensitive: Option<(BoundedVec<u8, T::MaxEncryptedDataLength>, [u8; 12], u8)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Convert raw encrypted data to struct if present
            let encrypted_sensitive_data = match encrypted_sensitive {
                Some((ciphertext, nonce, version)) => {
                    ensure!(version == 1, Error::<T>::UnsupportedEncryptionVersion);
                    ensure!(ciphertext.len() >= 16, Error::<T>::InvalidEncryptedData);
                    Some(EncryptedSensitiveData {
                        ciphertext,
                        nonce,
                        version,
                    })
                }
                None => None,
            };

            // Validate birth hour
            if let Some(hour) = birth_hour {
                ensure!(hour < 24, Error::<T>::InvalidEncryptedData);
            }

            let now = frame_system::Pallet::<T>::block_number();

            let profile = MemberProfile {
                display_name,
                gender,
                birth_date,
                birth_hour,
                longitude,
                latitude,
                encrypted_sensitive: encrypted_sensitive_data,
                is_provider: MemberProfiles::<T>::get(&who)
                    .map(|p| p.is_provider)
                    .unwrap_or(false),
                provider_verified: MemberProfiles::<T>::get(&who)
                    .map(|p| p.provider_verified)
                    .unwrap_or(false),
                updated_at: now,
            };

            MemberProfiles::<T>::insert(&who, profile);

            // Track first activity for cooldown
            if AccountCreationBlock::<T>::get(&who).is_none() {
                AccountCreationBlock::<T>::insert(&who, now);
            }

            Self::deposit_event(Event::ProfileUpdated { who });

            Ok(())
        }

        /// Clear sensitive (encrypted) profile data, keeping display name and birth info.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::clear_sensitive_data())]
        pub fn clear_sensitive_data(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            MemberProfiles::<T>::mutate(&who, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.encrypted_sensitive = None;
                    profile.updated_at = frame_system::Pallet::<T>::block_number();
                }
            });

            Self::deposit_event(Event::SensitiveDataCleared { who });

            Ok(())
        }

        /// Apply to become a service provider.
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::apply_provider())]
        pub fn apply_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Must have a profile
            let mut profile = MemberProfiles::<T>::get(&who).ok_or(Error::<T>::ProfileNotFound)?;

            ensure!(!profile.is_provider, Error::<T>::AlreadyProvider);

            profile.is_provider = true;
            profile.updated_at = frame_system::Pallet::<T>::block_number();

            MemberProfiles::<T>::insert(&who, profile);

            Self::deposit_event(Event::ProviderApplied { who });

            Ok(())
        }

        /// Admin: Verify or unverify a service provider.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::verify_provider())]
        pub fn verify_provider(
            origin: OriginFor<T>,
            provider: T::AccountId,
            verified: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            MemberProfiles::<T>::mutate(&provider, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.provider_verified = verified;
                    profile.updated_at = frame_system::Pallet::<T>::block_number();
                }
            });

            Self::deposit_event(Event::ProviderVerified { provider, verified });

            Ok(())
        }

        /// Use a free AI interpretation credit.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::use_free_ai())]
        pub fn use_free_ai(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let tier = Self::get_tier(&who);
            let quota = Self::get_monthly_free_ai_quota(tier);

            ensure!(quota > 0, Error::<T>::FreeAiQuotaExceeded);

            let now = frame_system::Pallet::<T>::block_number();
            let current_month = Self::block_to_month(now);

            let used = MonthlyFreeAiUsage::<T>::get(&who, current_month);
            ensure!(used < quota, Error::<T>::FreeAiQuotaExceeded);

            MonthlyFreeAiUsage::<T>::insert(&who, current_month, used + 1);

            Ok(())
        }
    }

    // ============ Helper Functions ============

    impl<T: Config> Pallet<T> {
        /// Get the reward pool account.
        pub fn reward_pool_account() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Get the current day number since genesis.
        pub fn block_to_day(block: BlockNumberFor<T>) -> u32 {
            let blocks_per_day: u32 = T::BlocksPerDay::get().try_into().unwrap_or(7200);
            let block_num: u32 = block.try_into().unwrap_or(0);
            block_num / blocks_per_day
        }

        /// Get the current month number since genesis.
        pub fn block_to_month(block: BlockNumberFor<T>) -> u32 {
            let blocks_per_month: u32 = T::BlocksPerMonth::get().try_into().unwrap_or(216000);
            let block_num: u32 = block.try_into().unwrap_or(0);
            block_num / blocks_per_month
        }

        /// Check if account is past the cooldown period.
        pub fn is_past_cooldown(who: &T::AccountId) -> bool {
            if let Some(created_at) = AccountCreationBlock::<T>::get(who) {
                let now = frame_system::Pallet::<T>::block_number();
                now >= created_at.saturating_add(T::NewAccountCooldown::get())
            } else {
                // No record means account is new, start tracking
                false
            }
        }

        /// Get member tier (Free if not subscribed or expired).
        pub fn get_tier(who: &T::AccountId) -> MemberTier {
            if let Some(member) = Members::<T>::get(who) {
                let now = frame_system::Pallet::<T>::block_number();
                if member.expires_at > now {
                    return member.tier;
                }
            }
            MemberTier::Free
        }

        /// Get monthly fee for a tier in USDT (precision 10^6, e.g., 5_000_000 = 5 USDT).
        pub fn get_tier_monthly_fee_usdt(tier: MemberTier) -> u64 {
            match tier {
                MemberTier::Free => 0,
                MemberTier::Bronze => 5_000_000,      // 5 USDT
                MemberTier::Silver => 25_000_000,     // 25 USDT
                MemberTier::Gold => 80_000_000,       // 80 USDT
                MemberTier::Platinum => 200_000_000,  // 200 USDT
                MemberTier::Diamond => 500_000_000,   // 500 USDT
            }
        }

        /// Get monthly fee for a tier (in DUST, converted from USDT via pricing).
        pub fn get_tier_monthly_fee(tier: MemberTier) -> BalanceOf<T> {
            let usdt_amount = Self::get_tier_monthly_fee_usdt(tier);
            Self::usdt_to_dust(usdt_amount)
        }

        /// Convert USDT to DUST using pricing module.
        /// 
        /// USDT amount has precision 10^6 (e.g., 25_000_000 = 25 USDT)
        /// Returns equivalent DUST amount with precision 10^12
        fn usdt_to_dust(usdt_amount: u64) -> BalanceOf<T> {
            if usdt_amount == 0 {
                return Zero::zero();
            }
            
            // Try to get DUST/USD rate from pricing module
            if let Some(rate) = T::Pricing::get_dust_to_usd_rate() {
                // rate is DUST/USD rate (precision 10^6), meaning 1 DUST = rate/10^6 USD
                // dust_amount = usdt_amount * 10^12 / rate
                let rate_u128: u128 = TryInto::<u128>::try_into(rate).unwrap_or(1_000_000);
                if rate_u128 > 0 {
                    let dust_amount = (usdt_amount as u128)
                        .saturating_mul(1_000_000_000_000u128) // 10^12 (DUST precision)
                        / rate_u128;
                    return TryInto::<BalanceOf<T>>::try_into(dust_amount).ok().unwrap_or_else(Zero::zero);
                }
            }
            
            // Fallback: assume 1 DUST = 1 USDT
            let dust = 1_000_000_000_000u128; // 1 DUST
            let fallback = (usdt_amount as u128).saturating_mul(dust) / 1_000_000u128;
            TryInto::<BalanceOf<T>>::try_into(fallback).ok().unwrap_or_else(Zero::zero)
        }

        /// Calculate subscription fee based on tier and duration.
        pub fn calculate_subscription_fee(
            tier: MemberTier,
            duration: SubscriptionDuration,
        ) -> BalanceOf<T> {
            let monthly_usdt = Self::get_tier_monthly_fee_usdt(tier);
            let total_usdt = match duration {
                SubscriptionDuration::Monthly => monthly_usdt,
                SubscriptionDuration::Yearly => {
                    // 10 months for price of 12 (≈16.7% discount)
                    monthly_usdt.saturating_mul(10)
                }
            };
            Self::usdt_to_dust(total_usdt)
        }

        /// Get storage deposit discount rate (basis points, 3000 = 30%).
        pub fn get_storage_discount(tier: MemberTier) -> u32 {
            match tier {
                MemberTier::Free => 0,
                MemberTier::Bronze => 3000,    // 30%
                MemberTier::Silver => 3000,    // 30%
                MemberTier::Gold => 4000,      // 40%
                MemberTier::Platinum => 5000,  // 50%
                MemberTier::Diamond => 6000,   // 60%
            }
        }

        /// Get AI interpretation discount rate (basis points).
        pub fn get_ai_discount(tier: MemberTier) -> u32 {
            match tier {
                MemberTier::Free => 0,
                MemberTier::Bronze => 1500,    // 15%
                MemberTier::Silver => 2000,    // 20%
                MemberTier::Gold => 5000,      // 50%
                MemberTier::Platinum => 7000,  // 70%
                MemberTier::Diamond => 8000,   // 80%
            }
        }

        /// Get monthly free AI interpretation quota.
        pub fn get_monthly_free_ai_quota(tier: MemberTier) -> u32 {
            match tier {
                MemberTier::Free => 0,
                MemberTier::Bronze => 1,
                MemberTier::Silver => 5,
                MemberTier::Gold => 5,
                MemberTier::Platinum => 20,
                MemberTier::Diamond => 50,
            }
        }

        /// Get daily free divination quota.
        pub fn get_daily_free_divination_quota(tier: MemberTier) -> u32 {
            match tier {
                MemberTier::Free => 3,
                MemberTier::Bronze => 5,
                MemberTier::Silver => 10,
                MemberTier::Gold => 20,
                MemberTier::Platinum => 50,
                MemberTier::Diamond => 100,
            }
        }

        /// Get DUST reward multiplier (basis points, 10000 = 1.0x).
        pub fn get_reward_multiplier_base(tier: MemberTier) -> u32 {
            match tier {
                MemberTier::Free => 10000,     // 1.0x
                MemberTier::Bronze => 12000,   // 1.2x
                MemberTier::Silver => 15000,   // 1.5x
                MemberTier::Gold => 20000,     // 2.0x
                MemberTier::Platinum => 30000, // 3.0x
                MemberTier::Diamond => 50000,  // 5.0x
            }
        }

        /// Get base check-in reward (0.001 DUST).
        pub fn get_check_in_base_reward() -> BalanceOf<T> {
            let reward = 1_000_000_000u128; // 0.001 DUST
            reward.try_into().ok().unwrap_or_else(Zero::zero)
        }

        /// Get daily reward limit for a category.
        pub fn get_daily_reward_limit(tx_type: RewardTxType) -> BalanceOf<T> {
            let dust = 1_000_000_000_000u128;
            let limit = match tx_type {
                RewardTxType::CheckIn => dust / 1000,     // 0.001 DUST
                RewardTxType::Divination => dust / 10,    // 0.1 DUST
                RewardTxType::AiCashback => dust * 1000,  // No practical limit
                RewardTxType::Delete => dust / 20,        // 0.05 DUST
                RewardTxType::MarketCashback => dust * 1000,
                RewardTxType::Review => dust / 20,        // 0.05 DUST
                RewardTxType::Referral => dust / 2,       // 0.5 DUST
                RewardTxType::NftMint => dust * 1000,
                RewardTxType::NftTrade => dust * 1000,
            };
            limit.try_into().ok().unwrap_or_else(Zero::zero)
        }

        /// Get reward pool balance.
        pub fn reward_pool_balance() -> BalanceOf<T> {
            T::Currency::free_balance(&Self::reward_pool_account())
        }

        /// Get dynamic reward adjustment factor based on pool balance.
        pub fn get_pool_adjustment_factor() -> u32 {
            let pool_balance = Self::reward_pool_balance();

            // Estimate monthly burn (use last month's data or default)
            let current_month = Self::block_to_month(frame_system::Pallet::<T>::block_number());
            let last_month_burn = MonthlyRewardsDistributed::<T>::get(current_month.saturating_sub(1));

            let monthly_burn = if last_month_burn.is_zero() {
                // Default: assume 2000 DUST/month
                let default_burn: BalanceOf<T> = (2_000_000_000_000_000u128)
                    .try_into()
                    .ok()
                    .unwrap_or_else(Zero::zero);
                default_burn
            } else {
                last_month_burn
            };

            if monthly_burn.is_zero() {
                return 10000; // 100%
            }

            // Calculate months remaining
            let months_remaining = pool_balance
                .checked_div(&monthly_burn)
                .map(|m| {
                    let m_u32: u32 = m.try_into().unwrap_or(u32::MAX);
                    m_u32
                })
                .unwrap_or(u32::MAX);

            match months_remaining {
                0..=2 => 5000,   // 50% - Emergency
                3..=5 => 7500,   // 75% - Warning
                _ => 10000,      // 100% - Normal
            }
        }

        /// Internal function to grant DUST reward.
        pub fn do_grant_reward(
            who: &T::AccountId,
            base_amount: BalanceOf<T>,
            tx_type: RewardTxType,
            memo: &[u8],
        ) -> Result<BalanceOf<T>, DispatchError> {
            // 1. Check cooldown
            ensure!(Self::is_past_cooldown(who), Error::<T>::AccountInCooldown);

            // 2. Check minimum balance
            ensure!(
                T::Currency::free_balance(who) >= T::MinBalanceForRewards::get(),
                Error::<T>::BalanceTooLow
            );

            // 3. Get tier multiplier
            let tier = Self::get_tier(who);
            let tier_multiplier = Self::get_reward_multiplier_base(tier);

            // 4. Get pool adjustment factor
            let pool_factor = Self::get_pool_adjustment_factor();

            // 5. Calculate final reward: base * tier_mult * pool_factor / 10000 / 10000
            let final_amount = base_amount
                .saturating_mul(tier_multiplier.into())
                .saturating_mul(pool_factor.into())
                / 100_000_000u32.into();

            if final_amount.is_zero() {
                return Ok(Zero::zero());
            }

            // 6. Check daily limit
            let now = frame_system::Pallet::<T>::block_number();
            let today = Self::block_to_day(now);

            let mut balance = RewardBalances::<T>::get(who);
            if balance.today_date != today {
                balance.today_date = today;
                balance.today_earned = Zero::zero();
            }

            let daily_limit = Self::get_daily_reward_limit(tx_type);
            let remaining = daily_limit.saturating_sub(balance.today_earned);

            if remaining.is_zero() {
                return Err(Error::<T>::DailyLimitExceeded.into());
            }

            let actual_amount = final_amount.min(remaining);

            // 7. Check reward pool balance
            let pool_balance = Self::reward_pool_balance();
            ensure!(pool_balance >= actual_amount, Error::<T>::RewardPoolEmpty);

            // 8. Transfer from reward pool
            T::Currency::transfer(
                &Self::reward_pool_account(),
                who,
                actual_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 9. Update balance stats
            balance.today_earned = balance.today_earned.saturating_add(actual_amount);
            balance.total_earned = balance.total_earned.saturating_add(actual_amount);
            balance.last_updated = now;
            RewardBalances::<T>::insert(who, balance);

            // 10. Record history (ring buffer)
            let mut index = RewardHistoryIndex::<T>::get(who);
            let max_size = T::MaxRewardHistorySize::get();

            let memo_bounded: BoundedVec<u8, ConstU32<32>> = memo
                .iter()
                .take(32)
                .cloned()
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap_or_default();

            let tx = RewardTransaction {
                tx_type,
                amount: actual_amount,
                timestamp: now,
                memo: memo_bounded,
            };

            RewardHistory::<T>::insert(who, index, tx);
            index = (index + 1) % max_size;
            RewardHistoryIndex::<T>::insert(who, index);

            // 11. Update monthly stats
            let current_month = Self::block_to_month(now);
            MonthlyRewardsDistributed::<T>::mutate(current_month, |total| {
                *total = total.saturating_add(actual_amount);
            });

            // 12. Update global stats
            GlobalStats::<T>::mutate(|stats| {
                stats.total_rewards_issued = stats.total_rewards_issued.saturating_add(actual_amount);
            });

            Self::deposit_event(Event::RewardGranted {
                who: who.clone(),
                amount: actual_amount,
                tx_type,
            });

            Ok(actual_amount)
        }

        /// Check if a user can receive rewards.
        pub fn can_receive_reward(who: &T::AccountId) -> bool {
            Self::is_past_cooldown(who)
                && T::Currency::free_balance(who) >= T::MinBalanceForRewards::get()
        }

        /// 分配费用
        /// 
        /// 费用分配：销毁 5%，国库 2%，存储 3%，推荐链 90%
        fn distribute_fee(who: &T::AccountId, fee: BalanceOf<T>) -> DispatchResult {
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
                T::Currency::transfer(
                    who,
                    &T::BurnAccount::get(),
                    burn_amount,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            // 国库
            if !treasury_amount.is_zero() {
                T::Currency::transfer(
                    who,
                    &T::TreasuryAccount::get(),
                    treasury_amount,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            // 存储 - 充值到用户的 UserFunding 账户
            if !storage_amount.is_zero() {
                let user_funding_account = T::UserFundingProvider::derive_user_funding_account(who);
                T::Currency::transfer(
                    who,
                    &user_funding_account,
                    storage_amount,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            // 推荐链分配（90%）
            if !distributable.is_zero() {
                let distributable_u128: u128 = distributable.try_into().unwrap_or(0);
                let _ = T::AffiliateDistributor::distribute_rewards(
                    who,
                    distributable_u128,
                    None,
                );
            }

            Ok(())
        }

        /// 计算百分比
        fn calculate_percent(total: BalanceOf<T>, percent: u8) -> BalanceOf<T> {
            if percent == 0 || percent > 100 {
                return BalanceOf::<T>::zero();
            }
            total.saturating_mul(percent.into()) / 100u32.into()
        }

        /// Get remaining free AI quota for current month.
        pub fn get_remaining_free_ai(who: &T::AccountId) -> u32 {
            let tier = Self::get_tier(who);
            let quota = Self::get_monthly_free_ai_quota(tier);
            let now = frame_system::Pallet::<T>::block_number();
            let current_month = Self::block_to_month(now);
            let used = MonthlyFreeAiUsage::<T>::get(who, current_month);
            quota.saturating_sub(used)
        }
    }
}
