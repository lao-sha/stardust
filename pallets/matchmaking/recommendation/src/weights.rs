//! # Matchmaking Recommendation Pallet Weights
//!
//! 推荐模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn update_preferences() -> Weight;
    fn refresh_recommendations() -> Weight;
    fn clear_recommendations() -> Weight;
    fn report_user() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn update_preferences() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn refresh_recommendations() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(10))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn clear_recommendations() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn report_user() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn update_preferences() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn refresh_recommendations() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn clear_recommendations() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn report_user() -> Weight { Weight::from_parts(30_000_000, 0) }
}
