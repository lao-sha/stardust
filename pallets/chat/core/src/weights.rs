//! # Chat Core Pallet Weights
//!
//! 聊天核心模块权重定义

use frame_support::{traits::Get, weights::Weight};

/// 权重信息 Trait
pub trait WeightInfo {
    fn send_message() -> Weight;
    fn mark_as_read() -> Weight;
    fn delete_message() -> Weight;
    fn mark_batch_as_read(n: u32) -> Weight;
    fn mark_session_as_read(n: u32) -> Weight;
    fn archive_session() -> Weight;
    fn block_user() -> Weight;
    fn unblock_user() -> Weight;
    fn cleanup_old_messages(n: u32) -> Weight;
    fn register_chat_user() -> Weight;
    fn update_chat_profile() -> Weight;
    fn set_user_status() -> Weight;
    fn update_privacy_settings() -> Weight;
}

/// Substrate 权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn send_message() -> Weight {
        Weight::from_parts(225_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    fn mark_as_read() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn delete_message() -> Weight {
        Weight::from_parts(75_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn mark_batch_as_read(n: u32) -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(Weight::from_parts(25_000_000, 0).saturating_mul(n as u64))
            .saturating_add(T::DbWeight::get().reads(1 + n as u64))
            .saturating_add(T::DbWeight::get().writes(1 + n as u64))
    }
    fn mark_session_as_read(_n: u32) -> Weight {
        Weight::from_parts(100_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn archive_session() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn block_user() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn unblock_user() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn cleanup_old_messages(n: u32) -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(Weight::from_parts(50_000_000, 0).saturating_mul(n as u64))
            .saturating_add(T::DbWeight::get().reads(n as u64))
            .saturating_add(T::DbWeight::get().writes(n as u64))
    }
    fn register_chat_user() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    fn update_chat_profile() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn set_user_status() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn update_privacy_settings() -> Weight {
        Weight::from_parts(25_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

/// 默认权重实现（用于测试）
impl WeightInfo for () {
    fn send_message() -> Weight { Weight::from_parts(225_000_000, 0) }
    fn mark_as_read() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn delete_message() -> Weight { Weight::from_parts(75_000_000, 0) }
    fn mark_batch_as_read(n: u32) -> Weight { Weight::from_parts(50_000_000 + 25_000_000 * n as u64, 0) }
    fn mark_session_as_read(_n: u32) -> Weight { Weight::from_parts(100_000_000, 0) }
    fn archive_session() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn block_user() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn unblock_user() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn cleanup_old_messages(n: u32) -> Weight { Weight::from_parts(50_000_000 + 50_000_000 * n as u64, 0) }
    fn register_chat_user() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn update_chat_profile() -> Weight { Weight::from_parts(50_000_000, 0) }
    fn set_user_status() -> Weight { Weight::from_parts(25_000_000, 0) }
    fn update_privacy_settings() -> Weight { Weight::from_parts(25_000_000, 0) }
}
