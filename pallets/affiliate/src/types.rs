//! 函数级中文注释：Affiliate 统一类型定义
//!
//! 包含所有子模块共享的类型定义

extern crate alloc;
use alloc::vec;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use scale_info::TypeInfo;

/// 函数级中文注释：结算模式枚举
///
/// 三种模式：
/// - Weekly: 全部使用周结算
/// - Instant: 全部使用即时分成
/// - Hybrid: 前N层即时，后M层周结算
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
#[codec(mel_bound())]
pub enum SettlementMode {
    /// 全周结算模式
    Weekly,
    /// 全即时分成模式
    Instant,
    /// 混合模式
    Hybrid {
        /// 前N层使用即时分成
        instant_levels: u8,
        /// 后M层使用周结算
        weekly_levels: u8,
    },
}

impl Default for SettlementMode {
    fn default() -> Self {
        Self::Weekly
    }
}

/// 函数级中文注释：分成比例配置（15层）
///
/// 用于 Instant 和 Weekly 模式的分成比例
pub type LevelPercents = BoundedVec<u8, ConstU32<15>>;

/// 函数级中文注释：默认即时分成比例
///
/// 总计：99%（剩余1%并入国库）
/// - L1: 30%（最高奖励）
/// - L2: 25%
/// - L3: 15%
/// - L4: 10%
/// - L5: 7%
/// - L6: 3%
/// - L7-L9: 各2%
/// - L10-L15: 各1%
pub fn default_instant_percents() -> LevelPercents {
    BoundedVec::try_from(vec![30, 25, 15, 10, 7, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1])
        .expect("default_instant_percents: length is 15")
}

/// 函数级中文注释：默认周结算分成比例
///
/// 总计：82%（剩余18%系统费用）
/// - L1: 20%
/// - L2: 10%
/// - L3-L15: 各4%
pub fn default_weekly_percents() -> LevelPercents {
    BoundedVec::try_from(vec![20, 10, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4])
        .expect("default_weekly_percents: length is 15")
}

/// 函数级中文注释：推荐链最大层数
pub const MAX_REFERRAL_CHAIN: u32 = 15;

/// 函数级中文注释：推荐码最大长度
pub const MAX_CODE_LEN: u32 = 16;

/// 函数级中文注释：推荐码最小长度
pub const MIN_CODE_LEN: u32 = 4;

// ===== 🆕 2025-10-29: Trading Pallet 集成接口 =====

/// 函数级详细中文注释：联盟计酬分配器接口（供Trading Pallet调用）
/// 
/// 此trait提供了Trading Pallet所需的联盟奖励分配功能。
/// 支持即时分成和周结算两种模式。
pub trait AffiliateDistributor<AccountId, Balance, BlockNumber> {
    /// 分配联盟奖励
    /// 
    /// # 参数
    /// - `buyer`: 买家账户（佣金来源）
    /// - `amount`: 佣金金额
    /// - `target`: 可选的目标信息（域, ID），用于某些特殊场景
    /// 
    /// # 返回
    /// - `Ok(实际分配金额)`: 成功分配
    /// - `Err`: 分配失败
    fn distribute_rewards(
        buyer: &AccountId,
        amount: Balance,
        target: Option<(u8, u64)>,
    ) -> Result<Balance, sp_runtime::DispatchError>;
}

