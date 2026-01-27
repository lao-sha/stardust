//! # Escrow Pallet Weights
//!
//! 托管模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn lock() -> Weight;
    fn release() -> Weight;
    fn refund() -> Weight;
    fn dispute() -> Weight;
    fn resolve() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn lock() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn release() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn refund() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn dispute() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn resolve() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn lock() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn release() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn refund() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn dispute() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn resolve() -> Weight { Weight::from_parts(80_000_000, 0) }
}
