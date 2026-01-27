//! # Storage Service Pallet Weights
//!
//! 存储服务模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn request_pin() -> Weight;
    fn mark_pinned() -> Weight;
    fn mark_pin_failed() -> Weight;
    fn charge_due(n: u32) -> Weight;
    fn set_billing_params() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn request_pin() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    fn mark_pinned() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn mark_pin_failed() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn charge_due(n: u32) -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(Weight::from_parts(30_000_000, 0).saturating_mul(n as u64))
            .saturating_add(T::DbWeight::get().reads(2 + n as u64 * 2))
            .saturating_add(T::DbWeight::get().writes(n as u64 * 2))
    }
    fn set_billing_params() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn request_pin() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn mark_pinned() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn mark_pin_failed() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn charge_due(n: u32) -> Weight { Weight::from_parts(50_000_000 + 30_000_000 * n as u64, 0) }
    fn set_billing_params() -> Weight { Weight::from_parts(20_000_000, 0) }
}
