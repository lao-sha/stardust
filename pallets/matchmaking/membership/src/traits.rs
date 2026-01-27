//! # 婚恋会员模块 - Trait 定义
//!
//! 定义会员服务的 Trait 接口，供其他模块调用。

use crate::types::{MembershipTier, MembershipBenefits};

/// 会员服务提供者 Trait
/// 
/// 其他模块可以通过此 Trait 查询用户的会员状态和权益
pub trait MembershipProvider<AccountId, BlockNumber> {
    /// 获取用户的会员等级
    fn get_tier(who: &AccountId) -> MembershipTier;
    
    /// 检查会员是否有效（未过期）
    fn is_active_member(who: &AccountId) -> bool;
    
    /// 获取会员到期时间
    fn get_expiry(who: &AccountId) -> Option<BlockNumber>;
    
    /// 获取会员权益配置
    fn get_benefits(who: &AccountId) -> MembershipBenefits;
    
    /// 检查用户是否有特定权益
    fn has_benefit(who: &AccountId, benefit: MembershipBenefit) -> bool;
}

/// 会员权益枚举（用于权益检查）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MembershipBenefit {
    /// 查看谁喜欢了我
    SeeWhoLikesMe,
    /// 查看访客记录
    SeeVisitors,
    /// 隐身浏览
    BrowseInvisibly,
    /// 优先展示
    PriorityDisplay,
    /// 专属客服
    DedicatedSupport,
    /// 消息已读回执
    ReadReceipts,
    /// 高级筛选
    AdvancedFilters,
}

/// 会员使用量追踪 Trait
pub trait MembershipUsageTracker<AccountId> {
    /// 检查是否可以使用推荐功能
    fn can_use_recommendation(who: &AccountId) -> bool;
    
    /// 检查是否可以使用超级喜欢
    fn can_use_super_like(who: &AccountId) -> bool;
    
    /// 检查是否可以使用合婚分析
    fn can_use_compatibility_check(who: &AccountId) -> bool;
    
    /// 记录使用推荐功能
    fn record_recommendation_usage(who: &AccountId) -> Result<(), &'static str>;
    
    /// 记录使用超级喜欢
    fn record_super_like_usage(who: &AccountId) -> Result<(), &'static str>;
    
    /// 记录使用合婚分析
    fn record_compatibility_check_usage(who: &AccountId) -> Result<(), &'static str>;
}
