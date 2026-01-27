//! # Divination NFT Pallet Weights
//!
//! 占卜 NFT 模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn mint_nft() -> Weight;
    fn transfer_nft() -> Weight;
    fn burn_nft() -> Weight;
    fn list_nft() -> Weight;
    fn cancel_listing() -> Weight;
    fn buy_nft() -> Weight;
    fn make_offer() -> Weight;
    fn cancel_offer() -> Weight;
    fn accept_offer() -> Weight;
    fn create_collection() -> Weight;
    fn add_to_collection() -> Weight;
    fn remove_from_collection() -> Weight;
    fn delete_collection() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn mint_nft() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    fn transfer_nft() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn burn_nft() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn list_nft() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn cancel_listing() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn buy_nft() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    fn make_offer() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn cancel_offer() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn accept_offer() -> Weight {
        Weight::from_parts(80_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    fn create_collection() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn add_to_collection() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn remove_from_collection() -> Weight {
        Weight::from_parts(40_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn delete_collection() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn mint_nft() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn transfer_nft() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn burn_nft() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn list_nft() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn cancel_listing() -> Weight { Weight::from_parts(30_000_000, 0) }
    fn buy_nft() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn make_offer() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn cancel_offer() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn accept_offer() -> Weight { Weight::from_parts(80_000_000, 0) }
    fn create_collection() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn add_to_collection() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn remove_from_collection() -> Weight { Weight::from_parts(40_000_000, 0) }
    fn delete_collection() -> Weight { Weight::from_parts(50_000_000, 0) }
}
