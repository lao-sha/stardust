//! # Affiliate Pallet Weights
//!
//! 函数级中文注释：Affiliate Pallet 权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn bind_sponsor() -> Weight;
    fn claim_code() -> Weight;
    fn set_settlement_mode() -> Weight;
    fn set_weekly_percents() -> Weight;
    fn set_blocks_per_week() -> Weight;
    fn settle_cycle() -> Weight;
    fn propose_percentage_adjustment() -> Weight;
    fn vote_on_percentage_proposal() -> Weight;
    fn cancel_proposal() -> Weight;
    fn emergency_pause_governance() -> Weight;
    fn resume_governance() -> Weight;
    fn propose_membership_price_adjustment() -> Weight;
    fn vote_on_membership_price_proposal() -> Weight;
    fn cancel_membership_price_proposal() -> Weight;
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn bind_sponsor() -> Weight {
        Weight::from_parts(25_000, 0)
    }
    fn claim_code() -> Weight {
        Weight::from_parts(20_000, 0)
    }
    fn set_settlement_mode() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn set_weekly_percents() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    fn set_blocks_per_week() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    fn settle_cycle() -> Weight {
        Weight::from_parts(100_000, 0)
    }
    fn propose_percentage_adjustment() -> Weight {
        Weight::from_parts(50_000, 0)
    }
    fn vote_on_percentage_proposal() -> Weight {
        Weight::from_parts(30_000, 0)
    }
    fn cancel_proposal() -> Weight {
        Weight::from_parts(20_000, 0)
    }
    fn emergency_pause_governance() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    fn resume_governance() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    fn propose_membership_price_adjustment() -> Weight {
        Weight::from_parts(50_000, 0)
    }
    fn vote_on_membership_price_proposal() -> Weight {
        Weight::from_parts(30_000, 0)
    }
    fn cancel_membership_price_proposal() -> Weight {
        Weight::from_parts(20_000, 0)
    }
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn bind_sponsor() -> Weight {
        // 读取: Sponsors, AccountByCode, 循环检测(最多20次)
        // 写入: Sponsors, ReferralStats
        Weight::from_parts(25_000, 0)
            .saturating_add(T::DbWeight::get().reads(22))
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn claim_code() -> Weight {
        // 读取: MembershipProvider, AccountByCode, CodeByAccount
        // 写入: AccountByCode, CodeByAccount, ReferralStats
        Weight::from_parts(20_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn set_settlement_mode() -> Weight {
        // 写入: SettlementMode
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn set_weekly_percents() -> Weight {
        // 写入: WeeklyLevelPercents
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn set_blocks_per_week() -> Weight {
        // 写入: BlocksPerWeek
        Weight::from_parts(10_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn settle_cycle() -> Weight {
        // 读取: SettleCursor, Entitlement (多次), EscrowBalance
        // 写入: SettleCursor, Entitlement (多次), TotalWeeklyDistributed
        Weight::from_parts(100_000, 0)
            .saturating_add(T::DbWeight::get().reads(50))
            .saturating_add(T::DbWeight::get().writes(50))
    }

    fn propose_percentage_adjustment() -> Weight {
        // 读取: GovernancePaused, InstantLevelPercents, ActiveProposalsByAccount, LastProposalBlock, ProposalCooldown
        // 写入: ActiveProposals, ProposalDeposits, ActiveProposalsByAccount, LastProposalBlock, NextProposalId
        Weight::from_parts(50_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    fn vote_on_percentage_proposal() -> Weight {
        // 读取: GovernancePaused, ActiveProposals, ProposalVotes
        // 写入: ProposalVotes, VoteTally, VoteHistory
        Weight::from_parts(30_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn cancel_proposal() -> Weight {
        // 读取: ActiveProposals, ProposalDeposits
        // 写入: ActiveProposals, ActiveProposalsByAccount, 退还押金
        Weight::from_parts(20_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn emergency_pause_governance() -> Weight {
        // 写入: GovernancePaused, PauseReason
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn resume_governance() -> Weight {
        // 写入: GovernancePaused, PauseReason
        Weight::from_parts(15_000, 0)
            .saturating_add(T::DbWeight::get().writes(2))
    }

    fn propose_membership_price_adjustment() -> Weight {
        // 读取: GovernancePaused, ActiveProposalsByAccount, LastProposalBlock
        // 写入: ActiveMembershipPriceProposals, MembershipPriceProposalDeposits, ActiveProposalsByAccount, LastProposalBlock, NextProposalId
        Weight::from_parts(50_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(5))
    }

    fn vote_on_membership_price_proposal() -> Weight {
        // 读取: GovernancePaused, ActiveMembershipPriceProposals, MembershipPriceProposalVotes
        // 写入: MembershipPriceProposalVotes, MembershipPriceVoteTally, VoteHistory
        Weight::from_parts(30_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }

    fn cancel_membership_price_proposal() -> Weight {
        // 读取: ActiveMembershipPriceProposals, MembershipPriceProposalDeposits
        // 写入: ActiveMembershipPriceProposals, MembershipPriceVoteTally, 清理投票, ActiveProposalsByAccount
        Weight::from_parts(20_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(4))
    }
}
