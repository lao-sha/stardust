//! # Chat Group Pallet Weights
//!
//! 群聊模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn create_group() -> Weight;
    fn send_group_message() -> Weight;
    fn join_group() -> Weight;
    fn leave_group() -> Weight;
    fn disband_group() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_group() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn send_group_message() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn join_group() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn leave_group() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn disband_group() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(5))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn create_group() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn send_group_message() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn join_group() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn leave_group() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn disband_group() -> Weight { Weight::from_parts(100_000_000, 0) }
}
