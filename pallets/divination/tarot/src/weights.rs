//! # Divination Tarot Pallet Weights
//!
//! 塔罗牌模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn divine_random() -> Weight;
    fn divine_by_time() -> Weight;
    fn divine_by_numbers() -> Weight;
    fn divine_manual() -> Weight;
    fn divine_random_with_cut() -> Weight;
    fn set_reading_privacy_mode() -> Weight;
    fn delete_reading() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn divine_random() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_by_time() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_by_numbers() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_manual() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn divine_random_with_cut() -> Weight {
        Weight::from_parts(55_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn set_reading_privacy_mode() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn delete_reading() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn divine_random() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn divine_by_time() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn divine_by_numbers() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn divine_manual() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn divine_random_with_cut() -> Weight { Weight::from_parts(55_000_000, 0) }
    fn set_reading_privacy_mode() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn delete_reading() -> Weight { Weight::from_parts(50_000_000, 0) }
}
