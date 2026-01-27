//! # 婚恋会员模块 - 权重定义
//!
//! 定义各个 extrinsic 的权重。

use frame_support::weights::Weight;

pub trait WeightInfo {
    fn subscribe() -> Weight;
    fn renew() -> Weight;
    fn upgrade() -> Weight;
    fn cancel_auto_renew() -> Weight;
    fn use_benefit() -> Weight;
}

/// 默认权重实现
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn subscribe() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn renew() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn upgrade() -> Weight {
        Weight::from_parts(45_000_000, 0)
    }

    fn cancel_auto_renew() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }

    fn use_benefit() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }
}

/// 单元测试权重
impl WeightInfo for () {
    fn subscribe() -> Weight {
        Weight::from_parts(50_000_000, 0)
    }

    fn renew() -> Weight {
        Weight::from_parts(40_000_000, 0)
    }

    fn upgrade() -> Weight {
        Weight::from_parts(45_000_000, 0)
    }

    fn cancel_auto_renew() -> Weight {
        Weight::from_parts(20_000_000, 0)
    }

    fn use_benefit() -> Weight {
        Weight::from_parts(25_000_000, 0)
    }
}
