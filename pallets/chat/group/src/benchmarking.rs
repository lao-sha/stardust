//! # Chat Group Pallet Benchmarking
//!
//! 群聊模块基准测试

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use pallet::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_group() {
        let caller: T::AccountId = whitelisted_caller();
        let name: Vec<u8> = b"TestGroup".to_vec();
        let description: Option<Vec<u8>> = Some(b"A test group".to_vec());

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            name,
            description,
            EncryptionMode::None,
            true
        );
    }

    #[benchmark]
    fn send_group_message() {
        let caller: T::AccountId = whitelisted_caller();
        let group_id: u64 = 1;
        let content: Vec<u8> = b"Hello group!".to_vec();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            group_id,
            content,
            MessageType::Text
        );
    }

    #[benchmark]
    fn join_group() {
        let caller: T::AccountId = whitelisted_caller();
        let group_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), group_id);
    }

    #[benchmark]
    fn leave_group() {
        let caller: T::AccountId = whitelisted_caller();
        let group_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), group_id);
    }

    #[benchmark]
    fn disband_group() {
        let caller: T::AccountId = whitelisted_caller();
        let group_id: u64 = 1;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), group_id);
    }

    #[benchmark]
    fn handle_group_violation() {
        let group_id: u64 = 1;
        let violation_type: u8 = 1;
        let evidence_cid: Vec<u8> = b"QmEvidence".to_vec();

        #[extrinsic_call]
        _(RawOrigin::Root, group_id, violation_type, evidence_cid);
    }

    #[benchmark]
    fn top_up_group_deposit() {
        let caller: T::AccountId = whitelisted_caller();
        let group_id: u64 = 1;
        let amount: BalanceOf<T> = 1000u32.into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), group_id, amount);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
