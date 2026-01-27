//! # Matchmaking Interaction Pallet Weights
//!
//! 交友互动模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn like() -> Weight;
    fn super_like() -> Weight;
    fn pass() -> Weight;
    fn block_user() -> Weight;
    fn unblock_user() -> Weight;
    fn initiate_matchmaking_chat() -> Weight;
    fn view_profile() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn like() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn super_like() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    fn pass() -> Weight {
        Weight::from_parts(20_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn block_user() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn unblock_user() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn initiate_matchmaking_chat() -> Weight {
        Weight::from_parts(60_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn view_profile() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn like() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn super_like() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn pass() -> Weight { Weight::from_parts(20_000_000, 0) }
    fn block_user() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn unblock_user() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn initiate_matchmaking_chat() -> Weight { Weight::from_parts(60_000_000, 0) }
    fn view_profile() -> Weight { Weight::from_parts(30_000_000, 0) }
}
