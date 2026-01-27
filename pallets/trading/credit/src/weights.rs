//! # Trading Credit Pallet Weights
//!
//! 交易信用模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn endorse_buyer() -> Weight;
    fn set_referrer() -> Weight;
    fn rate_maker() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn endorse_buyer() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn set_referrer() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn rate_maker() -> Weight {
        Weight::from_parts(35_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn endorse_buyer() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn set_referrer() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn rate_maker() -> Weight { Weight::from_parts(35_000_000, 0) }
}
