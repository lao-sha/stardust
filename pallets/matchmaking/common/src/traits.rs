//! # 婚恋模块 - Trait 定义
//!
//! 定义婚恋系统的核心 Trait 接口。

use crate::types::*;
use frame_support::pallet_prelude::*;
use pallet_bazi_chart::types::SiZhuIndex;
use pallet_bazi_chart::interpretation::{CoreInterpretation, CompactXingGe};

/// 八字数据提供者 Trait
pub trait BaziDataProvider<AccountId> {
    /// 检查八字是否存在
    fn exists(bazi_id: u64) -> bool;

    /// 检查账户是否是八字所有者
    fn is_owner(account: &AccountId, bazi_id: u64) -> bool;

    /// 获取四柱索引
    fn get_sizhu_index(bazi_id: u64) -> Option<SiZhuIndex>;

    /// 获取解盘结果
    fn get_interpretation(bazi_id: u64) -> Option<CoreInterpretation>;

    /// 获取性格分析
    fn get_personality(bazi_id: u64) -> Option<CompactXingGe>;
}

/// 用户资料提供者 Trait
pub trait ProfileProvider<AccountId, T: ProfileConfig> {
    /// 检查用户资料是否存在
    fn profile_exists(account: &AccountId) -> bool;

    /// 获取用户资料
    fn get_profile(account: &AccountId) -> Option<UserProfile<T>>;

    /// 获取用户八字 ID
    fn get_bazi_id(account: &AccountId) -> Option<u64>;

    /// 检查用户是否已验证
    fn is_verified(account: &AccountId) -> bool;
}

/// 匹配算法 Trait
pub trait MatchingAlgorithm<AccountId> {
    /// 计算两个用户的匹配评分
    fn calculate_match_score(
        user_a: &AccountId,
        user_b: &AccountId,
    ) -> Result<MatchingScore, DispatchError>;

    /// 计算八字合婚评分
    fn calculate_bazi_compatibility(
        bazi_id_a: u64,
        bazi_id_b: u64,
    ) -> Result<u8, DispatchError>;

    /// 计算性格匹配评分
    fn calculate_personality_match(
        bazi_id_a: u64,
        bazi_id_b: u64,
    ) -> Result<u8, DispatchError>;
}

/// 推荐系统 Trait
pub trait RecommendationEngine<AccountId> {
    /// 获取推荐列表
    fn get_recommendations(
        user: &AccountId,
        limit: u32,
    ) -> Result<sp_std::vec::Vec<RecommendationResult<AccountId>>, DispatchError>;

    /// 更新推荐列表
    fn update_recommendations(user: &AccountId) -> DispatchResult;
}

/// 互动管理 Trait
pub trait InteractionManager<AccountId, BlockNumber> {
    /// 记录互动
    fn record_interaction(
        from: &AccountId,
        to: &AccountId,
        interaction_type: InteractionType,
    ) -> DispatchResult;

    /// 检查是否互相喜欢（匹配成功）
    fn is_mutual_like(user_a: &AccountId, user_b: &AccountId) -> bool;

    /// 检查是否被屏蔽
    fn is_blocked(from: &AccountId, to: &AccountId) -> bool;
}
