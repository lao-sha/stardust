//! # Divination Liuyao Pallet Weights
//!
//! 六爻模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn divine_by_coins() -> Weight;
    fn divine_by_numbers() -> Weight;
    fn divine_random() -> Weight;
    fn divine_manual() -> Weight;
    fn divine_by_time() -> Weight;
    fn divine_by_solar_time() -> Weight;
    fn set_gua_visibility() -> Weight;
    fn delete_gua() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn divine_by_coins() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_by_numbers() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_random() -> Weight {
        Weight::from_parts(70_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_manual() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_by_time() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_by_solar_time() -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn set_gua_visibility() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn delete_gua() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn divine_by_coins() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn divine_by_numbers() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn divine_random() -> Weight { Weight::from_parts(70_000_000, 0) }
    fn divine_manual() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn divine_by_time() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn divine_by_solar_time() -> Weight { Weight::from_parts(100_000_000, 0) }
    fn set_gua_visibility() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn delete_gua() -> Weight { Weight::from_parts(30_000_000, 0) }
}
