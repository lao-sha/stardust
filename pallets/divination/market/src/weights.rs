//! # Divination Market Pallet Weights
//!
//! 占卜市场模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn register_diviner() -> Weight;
    fn update_diviner_profile() -> Weight;
    fn set_diviner_status() -> Weight;
    fn create_service() -> Weight;
    fn update_service() -> Weight;
    fn delete_service() -> Weight;
    fn create_order() -> Weight;
    fn accept_order() -> Weight;
    fn complete_order() -> Weight;
    fn cancel_order() -> Weight;
    fn rate_order() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn register_diviner() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn update_diviner_profile() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn set_diviner_status() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn create_service() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn update_service() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn delete_service() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn create_order() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn accept_order() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn complete_order() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn cancel_order() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn rate_order() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn register_diviner() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn update_diviner_profile() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn set_diviner_status() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn create_service() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn update_service() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn delete_service() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn create_order() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn accept_order() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn complete_order() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn cancel_order() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn rate_order() -> Weight { Weight::from_parts(40_000_000, 0) }
}
