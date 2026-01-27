//! # Matchmaking Profile Pallet Weights
//!
//! 交友档案模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn create_profile() -> Weight;
    fn update_profile() -> Weight;
    fn update_preferences() -> Weight;
    fn link_bazi() -> Weight;
    fn update_privacy_mode() -> Weight;
    fn delete_profile() -> Weight;
    fn set_visibility() -> Weight;
    fn verify_profile() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_profile() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn update_profile() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn update_preferences() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn link_bazi() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn update_privacy_mode() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn delete_profile() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn set_visibility() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn verify_profile() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn create_profile() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn update_profile() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn update_preferences() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn link_bazi() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn update_privacy_mode() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn delete_profile() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn set_visibility() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn verify_profile() -> Weight { Weight::from_parts(30_000_000, 0) }
}
