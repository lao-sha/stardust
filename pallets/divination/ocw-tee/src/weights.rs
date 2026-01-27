//! # Divination OCW-TEE Pallet Weights
//!
//! OCW-TEE 模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn create_public_request() -> Weight;
    fn create_encrypted_request() -> Weight;
    fn submit_result() -> Weight;
    fn update_request_status() -> Weight;
    fn mark_request_failed() -> Weight;
    fn cancel_request() -> Weight;
    fn update_divination() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_public_request() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn create_encrypted_request() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn submit_result() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn update_request_status() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn mark_request_failed() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn cancel_request() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn update_divination() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn create_public_request() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn create_encrypted_request() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn submit_result() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn update_request_status() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn mark_request_failed() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn cancel_request() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn update_divination() -> Weight { Weight::from_parts(100_000_000, 0) }
}
