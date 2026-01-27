//! # Divination AI Pallet Weights
//!
//! AI 占卜模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn request_interpretation() -> Weight;
    fn submit_result() -> Weight;
    fn cancel_request() -> Weight;
    fn set_oracle_config() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn request_interpretation() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn submit_result() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn cancel_request() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn set_oracle_config() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn request_interpretation() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn submit_result() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn cancel_request() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn set_oracle_config() -> Weight { Weight::from_parts(20_000_000, 0) }
}
