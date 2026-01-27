//! # Matchmaking Matching Pallet Weights
//!
//! 匹配模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn create_request() -> Weight;
    fn authorize_request() -> Weight;
    fn reject_request() -> Weight;
    fn cancel_request() -> Weight;
    fn generate_report() -> Weight;
    fn complete_match() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_request() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn authorize_request() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn reject_request() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn cancel_request() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn generate_report() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn complete_match() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn create_request() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn authorize_request() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn reject_request() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn cancel_request() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn generate_report() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn complete_match() -> Weight { Weight::from_parts(60_000_000, 0) }
}
