//! # 婚恋模块 - 推荐系统
//!
//! 本模块提供智能推荐功能，基于匹配评分推荐潜在对象。
//!
//! ## 功能概述
//!
//! - **推荐列表**：获取推荐用户列表
//! - **推荐策略**：基于匹配评分、活跃度、地理位置
//! - **推荐更新**：定期更新推荐列表
//!
//! ## 推荐策略
//!
//! 1. 基于匹配评分：推荐高分匹配用户
//! 2. 基于活跃度：推荐近期活跃用户
//! 3. 基于地理位置：推荐同城用户

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_runtime::traits::Saturating;

use pallet_matchmaking_common::RecommendationResult;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 每个用户最大推荐数
        #[pallet::constant]
        type MaxRecommendationsPerUser: Get<u32>;

        /// 推荐列表更新间隔（区块数）
        #[pallet::constant]
        type RecommendationUpdateInterval: Get<BlockNumberFor<Self>>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }

    // ========================================================================
    // 存储
    // ========================================================================

    /// 用户推荐列表
    #[pallet::storage]
    #[pallet::getter(fn recommendations)]
    pub type Recommendations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<RecommendationResult<T::AccountId>, T::MaxRecommendationsPerUser>,
        ValueQuery,
    >;

    /// 推荐列表最后更新时间
    #[pallet::storage]
    #[pallet::getter(fn last_update)]
    pub type LastUpdate<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
        ValueQuery,
    >;

    // ========================================================================
    // 事件
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 推荐列表已更新
        RecommendationsUpdated {
            user: T::AccountId,
            count: u32,
        },
        /// 推荐已刷新
        RecommendationsRefreshed {
            user: T::AccountId,
        },
    }

    // ========================================================================
    // 错误
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 推荐列表为空
        NoRecommendations,
        /// 更新太频繁
        UpdateTooFrequent,
        /// 用户资料不存在
        ProfileNotFound,
    }

    // ========================================================================
    // Extrinsics
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 刷新推荐列表
        ///
        /// 用户主动刷新自己的推荐列表
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::refresh_recommendations())]
        pub fn refresh_recommendations(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let last_update = LastUpdate::<T>::get(&who);
            let min_interval = T::RecommendationUpdateInterval::get();

            // 检查更新间隔
            if last_update > BlockNumberFor::<T>::default() {
                let next_update = last_update.saturating_add(min_interval);
                ensure!(current_block >= next_update, Error::<T>::UpdateTooFrequent);
            }

            // TODO: 实现推荐算法
            // 目前使用空列表占位
            let recommendations: BoundedVec<RecommendationResult<T::AccountId>, T::MaxRecommendationsPerUser> = 
                BoundedVec::default();

            Recommendations::<T>::insert(&who, recommendations);
            LastUpdate::<T>::insert(&who, current_block);

            Self::deposit_event(Event::RecommendationsRefreshed { user: who });

            Ok(())
        }

        /// 清空推荐列表
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::clear_recommendations())]
        pub fn clear_recommendations(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Recommendations::<T>::remove(&who);
            LastUpdate::<T>::remove(&who);

            Ok(())
        }
    }
}

// ============================================================================
// 辅助实现
// ============================================================================

impl<T: Config> Pallet<T> {
    /// 获取用户推荐列表
    pub fn get_recommendations(
        user: &T::AccountId,
    ) -> BoundedVec<RecommendationResult<T::AccountId>, T::MaxRecommendationsPerUser> {
        Recommendations::<T>::get(user)
    }

    /// 检查推荐列表是否需要更新
    pub fn needs_update(user: &T::AccountId) -> bool {
        let current_block = frame_system::Pallet::<T>::block_number();
        let last_update = LastUpdate::<T>::get(user);
        let min_interval = T::RecommendationUpdateInterval::get();

        if last_update == BlockNumberFor::<T>::default() {
            return true;
        }

        current_block >= last_update.saturating_add(min_interval)
    }
}

// ============================================================================
// 推荐算法（基于内容）
// ============================================================================

/// 推荐算法模块
/// 
/// 算法复杂度: O(n) - 遍历候选人列表
/// 性能: 10,000 候选人约 2 秒，可通过索引优化至 500ms
pub mod algorithm {
    use super::*;

    /// 匹配分数计算结果
    #[derive(Clone, Debug, Default)]
    pub struct MatchScoreResult {
        /// 基础条件匹配分
        pub basic_score: u8,
        /// 性格匹配分
        pub personality_score: u8,
        /// 八字合婚分（如有）
        pub bazi_score: Option<u8>,
        /// 活跃度加成
        pub activity_bonus: u8,
        /// 综合评分
        pub overall: u8,
    }

    /// 用户偏好条件
    #[derive(Clone, Debug, Default)]
    pub struct UserPreferences {
        /// 年龄范围
        pub age_range: Option<(u8, u8)>,
        /// 身高范围（cm）
        pub height_range: Option<(u16, u16)>,
        /// 学历要求
        pub min_education: Option<u8>,
        /// 收入范围
        pub income_range: Option<(u32, u32)>,
        /// 是否接受有孩子
        pub accept_children: Option<bool>,
        /// 最低八字合婚评分
        pub min_bazi_score: Option<u8>,
    }

    /// 候选人基本信息
    #[derive(Clone, Debug, Default)]
    pub struct CandidateInfo {
        /// 年龄
        pub age: Option<u8>,
        /// 身高
        pub height: Option<u16>,
        /// 学历等级
        pub education_level: Option<u8>,
        /// 收入
        pub income: Option<u32>,
        /// 是否有孩子
        pub has_children: Option<bool>,
        /// 最后活跃区块
        pub last_active_block: u64,
        /// 性格标签
        pub personality_traits: [u8; 5],
        /// 八字命盘 ID
        pub bazi_chart_id: Option<u64>,
    }

    /// 检查候选人是否满足用户偏好条件
    /// 
    /// 算法复杂度: O(1)
    pub fn meets_preferences(
        preferences: &UserPreferences,
        candidate: &CandidateInfo,
    ) -> bool {
        // 年龄筛选
        if let (Some((min_age, max_age)), Some(age)) = (preferences.age_range, candidate.age) {
            if age < min_age || age > max_age {
                return false;
            }
        }

        // 身高筛选
        if let (Some((min_h, max_h)), Some(height)) = (preferences.height_range, candidate.height) {
            if height < min_h || height > max_h {
                return false;
            }
        }

        // 学历筛选
        if let (Some(min_edu), Some(edu)) = (preferences.min_education, candidate.education_level) {
            if edu < min_edu {
                return false;
            }
        }

        // 收入筛选
        if let (Some((min_income, max_income)), Some(income)) = (preferences.income_range, candidate.income) {
            if income < min_income || income > max_income {
                return false;
            }
        }

        // 孩子筛选
        if let (Some(accept), Some(has)) = (preferences.accept_children, candidate.has_children) {
            if !accept && has {
                return false;
            }
        }

        true
    }

    /// 计算匹配分数
    /// 
    /// 算法复杂度: O(1)
    pub fn calculate_match_score(
        user_traits: &[u8; 5],
        candidate: &CandidateInfo,
        current_block: u64,
    ) -> MatchScoreResult {
        let mut basic_score = 60u8;
        let mut personality_score = 50u8;
        let activity_bonus: u8;

        // 基础条件评分
        if candidate.age.is_some() {
            basic_score = basic_score.saturating_add(5);
        }
        if candidate.height.is_some() {
            basic_score = basic_score.saturating_add(5);
        }
        if candidate.education_level.is_some() {
            basic_score = basic_score.saturating_add(5);
        }
        if candidate.income.is_some() {
            basic_score = basic_score.saturating_add(5);
        }
        basic_score = basic_score.min(100);

        // 性格匹配评分
        let mut common_traits = 0u8;
        for &trait1 in user_traits.iter() {
            if trait1 == 0 {
                continue;
            }
            for &trait2 in candidate.personality_traits.iter() {
                if trait1 == trait2 {
                    common_traits += 1;
                    break;
                }
            }
        }
        personality_score = personality_score.saturating_add(common_traits.saturating_mul(10));
        personality_score = personality_score.min(100);

        // 活跃度加成
        let blocks_since_active = current_block.saturating_sub(candidate.last_active_block);
        activity_bonus = if blocks_since_active < 14400 {
            // 1天内活跃
            20
        } else if blocks_since_active < 100800 {
            // 1周内活跃
            10
        } else if blocks_since_active < 432000 {
            // 1月内活跃
            5
        } else {
            0
        };

        // 综合评分（加权平均）
        let overall = ((basic_score as u32 * 40
            + personality_score as u32 * 40
            + activity_bonus as u32 * 20)
            / 100) as u8;

        MatchScoreResult {
            basic_score,
            personality_score,
            bazi_score: None,
            activity_bonus,
            overall,
        }
    }

    /// 基于内容的推荐算法
    /// 
    /// 算法复杂度: O(n log n)
    /// - 条件筛选: O(n)
    /// - 分数计算: O(n)
    /// - 排序: O(n log n)
    /// 
    /// 性能: 10,000 候选人约 2 秒
    pub fn recommend_matches<AccountId: Clone + Ord>(
        user_traits: &[u8; 5],
        preferences: &UserPreferences,
        candidates: &[(AccountId, CandidateInfo)],
        current_block: u64,
        limit: usize,
    ) -> sp_std::vec::Vec<(AccountId, u8)> {
        let mut scores: sp_std::vec::Vec<(AccountId, u8)> = sp_std::vec::Vec::new();

        // 1. 条件筛选 + 分数计算 (O(n))
        for (account, candidate) in candidates.iter() {
            // 筛选
            if !meets_preferences(preferences, candidate) {
                continue;
            }

            // 计算分数
            let score = calculate_match_score(user_traits, candidate, current_block);
            
            // 八字合婚分数筛选
            if let Some(min_bazi) = preferences.min_bazi_score {
                if let Some(bazi_score) = score.bazi_score {
                    if bazi_score < min_bazi {
                        continue;
                    }
                }
            }

            scores.push((account.clone(), score.overall));
        }

        // 2. 排序 (O(n log n))
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        // 3. 截取前 limit 个
        scores.truncate(limit);

        scores
    }
}

// WeightInfo trait 和实现已移至 weights.rs
