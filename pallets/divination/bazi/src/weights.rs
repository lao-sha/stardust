//! # Divination Bazi Pallet Weights
//!
//! 八字模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn create_bazi_chart() -> Weight;
    fn delete_bazi_chart() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_bazi_chart() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn delete_bazi_chart() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn create_bazi_chart() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn delete_bazi_chart() -> Weight { Weight::from_parts(30_000_000, 0) }
}
