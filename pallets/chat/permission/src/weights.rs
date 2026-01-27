//! # Chat Permission Pallet Weights
//!
//! 聊天权限模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn set_permission_level() -> Weight;
    fn reject_scene_type() -> Weight;
    fn block_user() -> Weight;
    fn unblock_user() -> Weight;
    fn add_friend() -> Weight;
    fn remove_friend() -> Weight;
    fn add_to_whitelist() -> Weight;
    fn remove_from_whitelist() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn set_permission_level() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn reject_scene_type() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn block_user() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn unblock_user() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn add_friend() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn remove_friend() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn add_to_whitelist() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn remove_from_whitelist() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn set_permission_level() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn reject_scene_type() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn block_user() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn unblock_user() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn add_friend() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn remove_friend() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn add_to_whitelist() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn remove_from_whitelist() -> Weight { Weight::from_parts(25_000_000, 0) }
}
