//! # 婚恋会员模块 - 类型定义
//!
//! 定义会员等级、订阅信息等类型。

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// 会员等级
/// 
/// 婚恋平台的会员分为三个等级：
/// - Free: 免费用户（基础功能）
/// - Annual: 年费会员（完整功能）
/// - Lifetime: 终身会员（永久权益）
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug, Default)]
pub enum MembershipTier {
    /// 免费用户
    #[default]
    Free,
    /// 年费会员
    Annual,
    /// 终身会员
    Lifetime,
}

/// 订阅时长
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Debug)]
pub enum SubscriptionDuration {
    /// 1个月
    OneMonth,
    /// 3个月
    ThreeMonths,
    /// 6个月
    SixMonths,
    /// 12个月（年费）
    OneYear,
    /// 终身
    Lifetime,
}

impl SubscriptionDuration {
    /// 获取月数
    pub fn months(&self) -> u32 {
        match self {
            Self::OneMonth => 1,
            Self::ThreeMonths => 3,
            Self::SixMonths => 6,
            Self::OneYear => 12,
            Self::Lifetime => u32::MAX, // 终身
        }
    }
    
    /// 获取折扣率（基点，10000 = 100%，即无折扣）
    pub fn discount_rate(&self) -> u32 {
        match self {
            Self::OneMonth => 10000,      // 无折扣
            Self::ThreeMonths => 9500,    // 5% 折扣
            Self::SixMonths => 9000,      // 10% 折扣
            Self::OneYear => 8000,        // 20% 折扣
            Self::Lifetime => 10000,      // 终身会员单独定价
        }
    }
}

/// 会员信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct MembershipInfo<BlockNumber, Balance> {
    /// 会员等级
    pub tier: MembershipTier,
    /// 订阅开始时间（区块号）
    pub subscribed_at: BlockNumber,
    /// 到期时间（区块号，终身会员为 BlockNumber::MAX）
    pub expires_at: BlockNumber,
    /// 累计支付金额
    pub total_paid: Balance,
    /// 是否自动续费
    pub auto_renew: bool,
    /// 连续订阅月数（用于忠诚度奖励）
    pub consecutive_months: u32,
    /// 推荐人（首次订阅时记录）
    pub referrer: Option<[u8; 32]>,
}

/// 会员权益配置
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct MembershipBenefits {
    /// 每日推荐数量上限
    pub daily_recommendations: u32,
    /// 每日超级喜欢次数
    pub daily_super_likes: u32,
    /// 每日合婚分析次数
    pub daily_compatibility_checks: u32,
    /// 是否可查看谁喜欢了我
    pub can_see_who_likes_me: bool,
    /// 是否可查看访客记录
    pub can_see_visitors: bool,
    /// 是否可隐身浏览
    pub can_browse_invisibly: bool,
    /// 是否可优先展示
    pub priority_display: bool,
    /// 是否有专属客服
    pub dedicated_support: bool,
    /// 消息已读回执
    pub read_receipts: bool,
    /// 高级筛选功能
    pub advanced_filters: bool,
}

impl Default for MembershipBenefits {
    fn default() -> Self {
        Self::free_tier()
    }
}

impl MembershipBenefits {
    /// 免费用户权益
    pub fn free_tier() -> Self {
        Self {
            daily_recommendations: 10,
            daily_super_likes: 0,
            daily_compatibility_checks: 1,
            can_see_who_likes_me: false,
            can_see_visitors: false,
            can_browse_invisibly: false,
            priority_display: false,
            dedicated_support: false,
            read_receipts: false,
            advanced_filters: false,
        }
    }
    
    /// 年费会员权益
    pub fn annual_tier() -> Self {
        Self {
            daily_recommendations: 50,
            daily_super_likes: 5,
            daily_compatibility_checks: 10,
            can_see_who_likes_me: true,
            can_see_visitors: true,
            can_browse_invisibly: true,
            priority_display: true,
            dedicated_support: false,
            read_receipts: true,
            advanced_filters: true,
        }
    }
    
    /// 终身会员权益
    pub fn lifetime_tier() -> Self {
        Self {
            daily_recommendations: 100,
            daily_super_likes: 10,
            daily_compatibility_checks: 30,
            can_see_who_likes_me: true,
            can_see_visitors: true,
            can_browse_invisibly: true,
            priority_display: true,
            dedicated_support: true,
            read_receipts: true,
            advanced_filters: true,
        }
    }
    
    /// 根据会员等级获取权益
    pub fn for_tier(tier: MembershipTier) -> Self {
        match tier {
            MembershipTier::Free => Self::free_tier(),
            MembershipTier::Annual => Self::annual_tier(),
            MembershipTier::Lifetime => Self::lifetime_tier(),
        }
    }
}

/// 会员统计信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct MembershipStats<Balance> {
    /// 免费用户数
    pub free_count: u64,
    /// 年费会员数
    pub annual_count: u64,
    /// 终身会员数
    pub lifetime_count: u64,
    /// 总收入
    pub total_revenue: Balance,
    /// 本月收入
    pub monthly_revenue: Balance,
    /// 当前月份索引
    pub current_month_index: u32,
}

/// 订阅交易记录
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct SubscriptionTransaction<BlockNumber, Balance> {
    /// 交易类型
    pub tx_type: SubscriptionTxType,
    /// 金额
    pub amount: Balance,
    /// 时长（月数）
    pub duration_months: u32,
    /// 交易时间
    pub timestamp: BlockNumber,
}

/// 订阅交易类型
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen, Debug)]
pub enum SubscriptionTxType {
    /// 新订阅
    NewSubscription,
    /// 续费
    Renewal,
    /// 升级
    Upgrade,
    /// 退款
    Refund,
}

/// 每日使用记录
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct DailyUsage {
    /// 日期索引（自创世以来的天数）
    pub day_index: u32,
    /// 已使用推荐次数
    pub recommendations_used: u32,
    /// 已使用超级喜欢次数
    pub super_likes_used: u32,
    /// 已使用合婚分析次数
    pub compatibility_checks_used: u32,
}
