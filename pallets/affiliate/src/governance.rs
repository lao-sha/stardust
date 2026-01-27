//! # 综合治理模块
//!
//! 本模块实现全民投票机制修改关键参数：
//! - **即时分成比例**（InstantLevelPercents）：15层联盟分成比例
//! - **年费等级价格**（MembershipPrices）：4个等级的USDT价格
//!
//! ## 核心功能
//!
//! - **提案创建**：持币大户、社区联署可发起提案
//! - **投票机制**：加权投票（持币70% + 参与20% + 贡献10%）+ 信念投票
//! - **自动执行**：通过后自动生效，无需人工干预
//! - **紧急机制**：技术委员会可紧急暂停治理（但无法否决提案）
//!
//! ## 安全保障
//!
//! - **唯一修改通道**：关键参数只能通过治理提案修改
//! - **严格验证**：参数合理性检查
//! - **防垃圾提案**：押金机制、频率限制、冷却期
//! - **审计追溯**：完整的提案和投票历史记录
//! - **🔥 技术委员会无否决权**：所有提案都必须通过全民投票

use super::*;
use crate::types::LevelPercents;
use frame_support::{pallet_prelude::*, traits::Currency};
use sp_runtime::Perbill;

/// 提案状态
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalStatus {
    /// 讨论期
    Discussion,
    /// 投票中
    Voting,
    /// 已通过，等待执行
    Approved,
    /// 已拒绝
    Rejected,
    /// 已取消
    Cancelled,
    /// 已执行
    Executed,
}

/// 投票选项
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Vote {
    /// 支持
    Aye,
    /// 反对
    Nay,
    /// 弃权
    Abstain,
}

impl Vote {
    /// 转换为 u8 编码（用于事件）
    pub fn to_u8(&self) -> u8 {
        match self {
            Vote::Aye => 0,
            Vote::Nay => 1,
            Vote::Abstain => 2,
        }
    }
}

/// 信念投票（锁定时长换取权重倍数）
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Conviction {
    /// 不锁定，权重 x1
    None,
    /// 锁定1周，权重 x1.5
    Locked1x,
    /// 锁定2周，权重 x2
    Locked2x,
    /// 锁定4周，权重 x3
    Locked3x,
    /// 锁定8周，权重 x4
    Locked4x,
    /// 锁定16周，权重 x5
    Locked5x,
    /// 锁定32周，权重 x6
    Locked6x,
}

impl Conviction {
    /// 获取权重倍数
    pub fn multiplier(&self) -> u128 {
        match self {
            Conviction::None => 1,
            Conviction::Locked1x => 15, // 1.5x * 10
            Conviction::Locked2x => 20,
            Conviction::Locked3x => 30,
            Conviction::Locked4x => 40,
            Conviction::Locked5x => 50,
            Conviction::Locked6x => 60,
        }
    }

    /// 获取锁定周数
    pub fn lock_weeks(&self) -> u32 {
        match self {
            Conviction::None => 0,
            Conviction::Locked1x => 1,
            Conviction::Locked2x => 2,
            Conviction::Locked3x => 4,
            Conviction::Locked4x => 8,
            Conviction::Locked5x => 16,
            Conviction::Locked6x => 32,
        }
    }
}

/// 分成比例调整提案
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct PercentageAdjustmentProposal<T: Config> {
    /// 提案ID
    pub proposal_id: u64,

    /// 提案发起人
    pub proposer: T::AccountId,

    /// 提案标题（IPFS CID）
    pub title_cid: BoundedVec<u8, ConstU32<64>>,

    /// 提案详情（IPFS CID）
    pub description_cid: BoundedVec<u8, ConstU32<64>>,

    /// 新的分成比例（15层）
    pub new_percentages: LevelPercents,

    /// 生效区块高度
    pub effective_block: BlockNumberFor<T>,

    /// 提案理由（IPFS CID）
    pub rationale_cid: BoundedVec<u8, ConstU32<64>>,

    /// 影响分析（IPFS CID，可选）
    pub impact_analysis_cid: Option<BoundedVec<u8, ConstU32<64>>>,

    /// 提案状态
    pub status: ProposalStatus,

    /// 是否重大提案（>10%变化）
    pub is_major: bool,

    /// 创建时间
    pub created_at: BlockNumberFor<T>,

    /// 投票开始时间
    pub voting_start: Option<BlockNumberFor<T>>,

    /// 投票结束时间
    pub voting_end: Option<BlockNumberFor<T>>,
}

/// 投票记录
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct VoteRecord<T: Config> {
    /// 投票人
    pub voter: T::AccountId,

    /// 投票选项
    pub vote: Vote,

    /// 信念投票
    pub conviction: Conviction,

    /// 投票权重
    pub weight: u128,

    /// 投票时间
    pub timestamp: BlockNumberFor<T>,
}

/// 投票统计
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct VoteTally {
    /// 支持票权重
    pub aye_votes: u128,

    /// 反对票权重
    pub nay_votes: u128,

    /// 弃权票权重
    pub abstain_votes: u128,

    /// 总投票权重
    pub total_turnout: u128,
}

impl VoteTally {
    /// 计算支持率（支持票 / (支持票 + 反对票)）
    pub fn approval_rate(&self) -> Perbill {
        let total = self.aye_votes.saturating_add(self.nay_votes);
        if total == 0 {
            return Perbill::zero();
        }
        Perbill::from_rational(self.aye_votes, total)
    }

    /// 计算参与率（总投票 / 总投票权）
    pub fn participation_rate(&self, total_power: u128) -> Perbill {
        if total_power == 0 {
            return Perbill::zero();
        }
        Perbill::from_rational(self.total_turnout, total_power)
    }
}

/// 比例变更历史记录
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct PercentageChangeRecord<T: Config> {
    /// 提案ID
    pub proposal_id: u64,

    /// 旧比例
    pub old_percentages: LevelPercents,

    /// 新比例
    pub new_percentages: LevelPercents,

    /// 执行区块
    pub executed_at: BlockNumberFor<T>,

    /// 执行者（通常是"Governance"）
    pub executed_by: BoundedVec<u8, ConstU32<32>>,
}

impl<T: Config> PercentageAdjustmentProposal<T> {
    /// 计算分成比例调整提案的押金金额
    pub fn calculate_deposit(&self) -> BalanceOf<T> {
        let units: u128 = 1_000_000_000_000_000_000u128; // 18位精度
        if self.is_major {
            (10000u128 * units).saturated_into() // 10,000 DUST（重大提案）
        } else {
            (1000u128 * units).saturated_into()  // 1,000 DUST（微调提案）
        }
    }
}

impl<T: Config> Pallet<T> {
    // ========================================
    // 提案验证
    // ========================================

    /// 验证新分成比例的有效性
    ///
    /// 🔥 2025-11-13 更新：第三层分成比例可以为0（全民投票决定）
    ///
    /// 验证规则：
    /// - 前2层（第1、2层）不能为0，确保基础激励
    /// - 第3层可以为0，允许社区通过投票调整
    /// - 第4-15层可以为0，提供灵活性
    /// - 总和必须在50-99%范围内
    /// - 前5层必须递减（包括0值）
    pub fn validate_percentages(percentages: &LevelPercents) -> DispatchResult {
        // 1. 检查长度
        ensure!(
            percentages.len() == 15,
            Error::<T>::InvalidPercentageLength
        );

        // 2. 检查单个比例范围
        for (index, &percentage) in percentages.iter().enumerate() {
            ensure!(
                percentage <= 100,
                Error::<T>::PercentageTooHigh
            );

            // 前2层不能为0，第3层可以为0（基于全民投票决定）
            if index < 2 {
                ensure!(
                    percentage > 0,
                    Error::<T>::CriticalLayerZero
                );
            }
        }

        // 3. 检查总和合理性
        let total: u32 = percentages.iter().map(|&x| x as u32).sum();
        ensure!(
            total >= 50,
            Error::<T>::TotalPercentageTooLow
        );
        ensure!(
            total <= 99,
            Error::<T>::TotalPercentageTooHigh
        );

        // 4. 检查递减合理性（前5层应该递减，但允许第3层为0的特殊情况）
        for i in 1..5 {
            // 🔥 2025-11-13：特殊处理第3层为0的情况
            // 如果第3层为0，允许第4、5层有合理的非零值
            if i == 2 && percentages[i] == 0 {
                // 第3层为0时，跳过这次递减检查
                continue;
            }
            if i == 3 && percentages[2] == 0 && percentages[i] > 0 {
                // 第3层为0，第4层不为0时，检查第4层是否合理（不超过第2层）
                ensure!(
                    percentages[i] <= percentages[1],
                    Error::<T>::NonDecreasingPercentage
                );
                continue;
            }
            if i == 4 && percentages[2] == 0 && percentages[i] > 0 {
                // 第3层为0，第5层不为0时，检查第5层是否合理（不超过第4层）
                if percentages[3] > 0 {
                    ensure!(
                        percentages[i] <= percentages[3],
                        Error::<T>::NonDecreasingPercentage
                    );
                } else {
                    // 如果第3、4层都为0，第5层不超过第2层
                    ensure!(
                        percentages[i] <= percentages[1],
                        Error::<T>::NonDecreasingPercentage
                    );
                }
                continue;
            }

            // 常规递减检查
            ensure!(
                percentages[i] <= percentages[i - 1],
                Error::<T>::NonDecreasingPercentage
            );
        }

        // 5. 检查极值（防止寡头垄断）
        ensure!(
            percentages[0] <= 50,
            Error::<T>::FirstLayerTooHigh
        );

        Ok(())
    }

    /// 计算变化幅度（百分点）
    pub fn calculate_change_magnitude(
        old: &LevelPercents,
        new: &LevelPercents,
    ) -> u32 {
        let mut total_change = 0u32;
        for i in 0..15 {
            let diff = if new[i] > old[i] {
                new[i] - old[i]
            } else {
                old[i] - new[i]
            };
            total_change = total_change.saturating_add(diff as u32);
        }
        total_change
    }

    // 🔥 2025-11-13：已删除微调提案阈值函数
    // 所有分成比例提案现在都使用统一的全民投票机制

    // ========================================
    // 投票权重计算
    // ========================================

    /// 计算账户的总投票权重
    /// 持币权重（70%） + 参与权重（20%） + 贡献权重（10%）
    pub fn calculate_total_voting_power(account: &T::AccountId) -> u128 {
        let stake_weight = Self::calculate_stake_weight(account)
            .saturating_mul(70)
            .saturating_div(100);

        let participation_weight = Self::calculate_participation_weight(account)
            .saturating_mul(20)
            .saturating_div(100);

        let contribution_weight = Self::calculate_contribution_weight(account)
            .saturating_mul(10)
            .saturating_div(100);

        stake_weight
            .saturating_add(participation_weight)
            .saturating_add(contribution_weight)
    }

    /// 计算持币权重（平方根，避免巨鲸垄断）
    fn calculate_stake_weight(account: &T::AccountId) -> u128 {
        let balance = T::Currency::free_balance(account);
        let balance_u128: u128 = balance.saturated_into();

        // 平方根权重
        let sqrt_balance = Self::integer_sqrt(balance_u128);

        // 权重上限：相当于100万 DUST 的权重
        let max_weight = 1000u128; // sqrt(1,000,000) = 1000

        sqrt_balance.min(max_weight)
    }

    /// 计算参与权重（历史投票次数）
    fn calculate_participation_weight(account: &T::AccountId) -> u128 {
        let vote_count = VoteHistory::<T>::get(account).len() as u128;

        match vote_count {
            0..=2 => 10,      // 新手
            3..=5 => 25,      // 活跃
            6..=10 => 50,     // 资深
            _ => 100,         // 元老
        }
    }

    /// 计算贡献权重（推荐贡献 + 委员会成员）
    fn calculate_contribution_weight(account: &T::AccountId) -> u128 {
        let mut weight = 0u128;

        // 推荐贡献（每个成功推荐 +2 分，最多50人 = 100分）
        let referral_count = Self::count_successful_referrals(account);
        weight = weight.saturating_add(referral_count.min(50).saturating_mul(2));

        // TODO: 技术委员会成员额外投票权重 +200
        // 注意：虽然技术委员会有额外权重，但无法否决任何治理提案
        // 所有提案都必须达到全民投票的参与率和支持率门槛
        // if Self::is_council_member(account) {
        //     weight = weight.saturating_add(200);
        // }

        weight.min(300)
    }

    /// 计算整数平方根（牛顿迭代法）
    fn integer_sqrt(n: u128) -> u128 {
        if n == 0 {
            return 0;
        }

        let mut x = n;
        let mut y = (x + 1) / 2;

        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }

        x
    }

    /// 统计成功推荐数量
    /// 通过遍历 Sponsors 存储，统计 sponsor == account 的数量
    fn count_successful_referrals(account: &T::AccountId) -> u128 {
        // 遍历 Sponsors 存储，统计该账户作为推荐人的次数
        let mut count = 0u128;
        for (_who, sponsor) in pallet_affiliate_referral::Sponsors::<T>::iter() {
            if &sponsor == account {
                count = count.saturating_add(1);
            }
        }
        count
    }

    // ========================================
    // 通过条件检查
    // ========================================

    /// 检查分成比例提案是否通过（技术委员会无法否决，所有提案都使用全民投票）
    pub fn check_proposal_passed(
        _proposal: &PercentageAdjustmentProposal<T>,
        tally: &VoteTally,
    ) -> bool {
        // 🔥 2025-11-13 重要修改：删除微调提案的技术委员会否决权
        // 所有分成比例提案现在都必须通过全民投票，技术委员会无法否决

        // 全民投票机制：最低参与率要求
        // 总投票权 = 总发行量的平方根（归一化处理，避免巨鲸主导）
        // 使用 pallet-balances 总发行量作为基准
        let total_issuance: u128 = T::Currency::total_issuance().saturated_into();
        let total_power = Self::integer_sqrt(total_issuance).max(100000u128);
        let participation = tally.participation_rate(total_power);

        // 最低参与率门槛：15%
        if participation < Perbill::from_percent(15) {
            return false;
        }

        // 自适应阈值：参与率越高，通过门槛越低
        let required_approval = if participation >= Perbill::from_percent(50) {
            Perbill::from_percent(50) // 50%参与 → 50%支持
        } else if participation >= Perbill::from_percent(30) {
            Perbill::from_percent(55) // 30%参与 → 55%支持
        } else {
            Perbill::from_percent(60) // 15%参与 → 60%支持
        };

        tally.approval_rate() >= required_approval
    }
}

// ========================================
// 年费价格治理模块 🆕
// ========================================

/// 年费等级枚举（重新导入以避免循环依赖）
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MembershipLevel {
    Year1,  // 1年会员
    Year3,  // 3年会员
    Year5,  // 5年会员
    Year10, // 10年会员
}

impl MembershipLevel {
    /// 转换为ID
    pub fn to_id(&self) -> u8 {
        match self {
            Self::Year1 => 0,
            Self::Year3 => 1,
            Self::Year5 => 2,
            Self::Year10 => 3,
        }
    }

    /// 从ID创建
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Year1),
            1 => Some(Self::Year3),
            2 => Some(Self::Year5),
            3 => Some(Self::Year10),
            _ => None,
        }
    }

    /// 获取等级名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Year1 => "Year1",
            Self::Year3 => "Year3",
            Self::Year5 => "Year5",
            Self::Year10 => "Year10",
        }
    }
}

/// 年费价格调整提案
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct MembershipPriceProposal<T: Config> {
    /// 提案ID
    pub proposal_id: u64,

    /// 提案发起人
    pub proposer: T::AccountId,

    /// 提案标题（IPFS CID）
    pub title_cid: BoundedVec<u8, ConstU32<64>>,

    /// 提案详情（IPFS CID）
    pub description_cid: BoundedVec<u8, ConstU32<64>>,

    /// 提案理由（IPFS CID）
    pub rationale_cid: BoundedVec<u8, ConstU32<64>>,

    /// 新的年费价格（USDT，精度 10^6）
    /// 按顺序：[Year1, Year3, Year5, Year10]
    pub new_prices_usdt: [u64; 4],

    /// 生效区块高度
    pub effective_block: BlockNumberFor<T>,

    /// 提案状态
    pub status: ProposalStatus,

    /// 是否为重大提案（价格变化 >20%）
    pub is_major: bool,

    /// 创建时间
    pub created_at: BlockNumberFor<T>,

    /// 投票开始时间
    pub voting_start: Option<BlockNumberFor<T>>,

    /// 投票结束时间
    pub voting_end: Option<BlockNumberFor<T>>,
}

impl<T: Config> MembershipPriceProposal<T> {
    /// 创建新的年费价格提案
    pub fn new(
        proposal_id: u64,
        proposer: T::AccountId,
        title_cid: BoundedVec<u8, ConstU32<64>>,
        description_cid: BoundedVec<u8, ConstU32<64>>,
        rationale_cid: BoundedVec<u8, ConstU32<64>>,
        new_prices_usdt: [u64; 4],
        current_block: BlockNumberFor<T>,
    ) -> Result<Self, &'static str> {
        // 验证价格范围（10-1000 USDT）
        for price in &new_prices_usdt {
            if *price < 10_000_000 || *price > 1_000_000_000 {
                return Err("Price out of range (10-1000 USDT)");
            }
        }

        // 验证价格递增性
        if new_prices_usdt[0] > new_prices_usdt[1] ||
           new_prices_usdt[1] > new_prices_usdt[2] ||
           new_prices_usdt[2] > new_prices_usdt[3] {
            return Err("Prices should be in ascending order");
        }

        // 计算是否为重大提案（假设当前价格）
        let current_prices = [50_000_000u64, 100_000_000, 200_000_000, 300_000_000];
        let is_major = Self::calculate_is_major(&new_prices_usdt, &current_prices);

        // 计算执行延迟（重大提案7天，微调提案3天）
        let delay_blocks = if is_major { 201600u32 } else { 43200u32 };
        let effective_block = current_block + delay_blocks.into();

        Ok(Self {
            proposal_id,
            proposer,
            title_cid,
            description_cid,
            rationale_cid,
            new_prices_usdt,
            effective_block,
            status: ProposalStatus::Discussion,
            is_major,
            created_at: current_block,
            voting_start: None,
            voting_end: None,
        })
    }

    /// 计算是否为重大提案（任一价格变化超过20%）
    fn calculate_is_major(new_prices: &[u64; 4], current_prices: &[u64; 4]) -> bool {
        for i in 0..4 {
            let change_percent = if new_prices[i] > current_prices[i] {
                ((new_prices[i] - current_prices[i]) * 100) / current_prices[i]
            } else {
                ((current_prices[i] - new_prices[i]) * 100) / current_prices[i]
            };

            if change_percent > 20 {
                return true;
            }
        }
        false
    }

    /// 验证年费价格
    pub fn validate_prices(prices: &[u64; 4]) -> Result<(), &'static str> {
        // 1. 价格范围检查（10-1000 USDT）
        for price in prices {
            if *price < 10_000_000 || *price > 1_000_000_000 {
                return Err("Price out of range (10-1000 USDT)");
            }
        }

        // 2. 递增性检查
        if prices[0] > prices[1] || prices[1] > prices[2] || prices[2] > prices[3] {
            return Err("Prices must be in ascending order");
        }

        // 3. 合理性检查（相邻价格差距不超过10倍）
        for i in 0..3 {
            if prices[i + 1] > prices[i] * 10 {
                return Err("Price gap too large between adjacent levels");
            }
        }

        Ok(())
    }

    /// 计算押金金额
    pub fn calculate_deposit(&self) -> BalanceOf<T> {
        let units: u128 = 1_000_000_000_000_000_000u128; // 18位精度
        if self.is_major {
            (10000u128 * units).saturated_into() // 10,000 DUST
        } else {
            (1000u128 * units).saturated_into()  // 1,000 DUST
        }
    }
}

/// 年费价格变更历史记录
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct MembershipPriceChangeRecord<T: Config> {
    /// 提案ID
    pub proposal_id: u64,

    /// 旧价格（USDT）
    pub old_prices_usdt: [u64; 4],

    /// 新价格（USDT）
    pub new_prices_usdt: [u64; 4],

    /// 执行区块
    pub executed_at: BlockNumberFor<T>,

    /// 执行者（通常是"Governance"）
    pub executed_by: BoundedVec<u8, ConstU32<32>>,
}

/// 通用提案类型枚举
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum GovernanceProposal<T: Config> {
    /// 分成比例调整提案
    PercentageAdjustment(PercentageAdjustmentProposal<T>),
    /// 年费价格调整提案
    MembershipPrice(MembershipPriceProposal<T>),
}

impl<T: Config> GovernanceProposal<T> {
    /// 获取提案ID
    pub fn proposal_id(&self) -> u64 {
        match self {
            Self::PercentageAdjustment(p) => p.proposal_id,
            Self::MembershipPrice(p) => p.proposal_id,
        }
    }

    /// 获取提案人
    pub fn proposer(&self) -> &T::AccountId {
        match self {
            Self::PercentageAdjustment(p) => &p.proposer,
            Self::MembershipPrice(p) => &p.proposer,
        }
    }

    /// 获取提案状态
    pub fn status(&self) -> &ProposalStatus {
        match self {
            Self::PercentageAdjustment(p) => &p.status,
            Self::MembershipPrice(p) => &p.status,
        }
    }

    /// 是否为重大提案
    pub fn is_major(&self) -> bool {
        match self {
            Self::PercentageAdjustment(p) => p.is_major,
            Self::MembershipPrice(p) => p.is_major,
        }
    }

    /// 计算押金金额
    pub fn calculate_deposit(&self) -> BalanceOf<T> {
        match self {
            Self::PercentageAdjustment(p) => p.calculate_deposit(),
            Self::MembershipPrice(p) => p.calculate_deposit(),
        }
    }

    /// 获取提案类型名称
    pub fn proposal_type(&self) -> &'static str {
        match self {
            Self::PercentageAdjustment(_) => "PercentageAdjustment",
            Self::MembershipPrice(_) => "MembershipPrice",
        }
    }
}

// 治理相关存储项在主 pallet (lib.rs) 中定义
// 这里只是文档说明参考
