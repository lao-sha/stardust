//! # Divination Almanac Pallet Weights
//!
//! 黄历模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn query_day_info() -> Weight;
    fn query_auspicious_days() -> Weight;
    fn set_almanac_data() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn query_day_info() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
    }
    fn query_auspicious_days() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
    }
    fn set_almanac_data() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn query_day_info() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn query_auspicious_days() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn set_almanac_data() -> Weight { Weight::from_parts(40_000_000, 0) }
}
