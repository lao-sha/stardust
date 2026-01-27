//! # Divination Ziwei Pallet Weights
//!
//! 紫微斗数模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn divine_by_time() -> Weight;
    fn divine_manual() -> Weight;
    fn divine_random() -> Weight;
    fn divine_by_solar_time() -> Weight;
    fn set_chart_visibility() -> Weight;
    fn divine_by_time_encrypted() -> Weight;
    fn update_encrypted_data() -> Weight;
    fn delete_chart() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn divine_by_time() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_manual() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_random() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_by_solar_time() -> Weight {
        Weight::from_parts(120_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn set_chart_visibility() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn divine_by_time_encrypted() -> Weight {
        Weight::from_parts(120_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    fn update_encrypted_data() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn delete_chart() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(5))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn divine_by_time() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn divine_manual() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn divine_random() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn divine_by_solar_time() -> Weight { Weight::from_parts(120_000_000, 0) }
    fn set_chart_visibility() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn divine_by_time_encrypted() -> Weight { Weight::from_parts(120_000_000, 0) }
    fn update_encrypted_data() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn delete_chart() -> Weight { Weight::from_parts(50_000_000, 0) }
}
