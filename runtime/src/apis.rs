// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// External crates imports
use alloc::vec::Vec;
use codec::Encode;
use frame_support::{
	genesis_builder_helper::{build_state, get_preset},
	weights::Weight,
};
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	traits::{Block as BlockT, NumberFor},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, ExtrinsicInclusionMode,
};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
	AccountId, Aura, Balance, Bazi, Block, BlockNumber, Executive, Grandpa, InherentDataExt, Livestream, Nonce, Runtime,
	RuntimeCall, RuntimeGenesisConfig, SessionKeys, StorageService, System, TransactionPayment, TeePrivacy, VERSION,
};

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: <Block as BlockT>::LazyBlock) {
			Executive::execute_block(block.into());
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl frame_support::view_functions::runtime_api::RuntimeViewFunction<Block> for Runtime {
		fn execute_view_function(id: frame_support::view_functions::ViewFunctionId, input: Vec<u8>) -> Result<Vec<u8>, frame_support::view_functions::ViewFunctionDispatchError> {
			Runtime::execute_view_function(id, input)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: <Block as BlockT>::LazyBlock,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block.into())
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;
			use baseline::Pallet as BaselineBench;
			use super::*;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		#[allow(non_local_definitions)]
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
			use frame_benchmarking::{baseline, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;
			use baseline::Pallet as BaselineBench;
			use super::*;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, super::configs::RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, crate::genesis_config_presets::get_preset)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			crate::genesis_config_presets::preset_names()
		}
	}

	// ============================================================================
	// TEE Privacy Runtime API
	// ============================================================================

	impl pallet_tee_privacy::runtime_api::TeePrivacyApi<Block, AccountId, BlockNumber> for Runtime {
		fn get_active_nodes() -> Vec<pallet_tee_privacy::types::TeeNodeInfo> {
			TeePrivacy::active_nodes()
				.iter()
				.filter_map(|account| TeePrivacy::tee_nodes(account).map(|node| {
					pallet_tee_privacy::types::TeeNodeInfo {
						account: node.account.encode(),
						enclave_pubkey: node.enclave_pubkey,
						tee_type: node.attestation.tee_type.clone(),
						status: node.status.clone(),
						registered_at: node.registered_at,
						mr_enclave: node.attestation.mr_enclave,
						attestation_timestamp: node.attestation.timestamp,
					}
				}))
				.collect()
		}

		fn get_node_info(account: AccountId) -> Option<pallet_tee_privacy::types::TeeNodeInfo> {
			TeePrivacy::tee_nodes(&account).map(|node| {
				pallet_tee_privacy::types::TeeNodeInfo {
					account: node.account.encode(),
					enclave_pubkey: node.enclave_pubkey,
					tee_type: node.attestation.tee_type.clone(),
					status: node.status.clone(),
					registered_at: node.registered_at,
					mr_enclave: node.attestation.mr_enclave,
					attestation_timestamp: node.attestation.timestamp,
				}
			})
		}

		fn get_enclave_pubkey(account: AccountId) -> Option<[u8; 32]> {
			TeePrivacy::get_enclave_pubkey(&account)
		}

		fn is_node_active(account: AccountId) -> bool {
			TeePrivacy::is_node_active(&account)
		}

		fn get_active_node_count() -> u32 {
			TeePrivacy::active_node_count()
		}

		fn get_node_count() -> u32 {
			TeePrivacy::node_count()
		}

		fn get_request_status(request_id: u64) -> Option<pallet_tee_privacy::types::RequestStatusInfo> {
			TeePrivacy::compute_requests(request_id).map(|req| {
				pallet_tee_privacy::types::RequestStatusInfo {
					request_id: req.id,
					requester: req.requester.encode(),
					compute_type_id: req.compute_type_id,
					input_hash: req.input_hash,
					assigned_node: req.assigned_node.map(|n| n.encode()),
					created_at: req.created_at,
					timeout_at: req.timeout_at,
					status: req.status.clone(),
					failover_count: req.failover_count,
					failure_reason: req.failure_reason.clone(),
				}
			})
		}

		fn get_user_pending_requests(account: AccountId) -> Vec<u64> {
			TeePrivacy::pending_requests()
				.iter()
				.filter(|&id| {
					TeePrivacy::compute_requests(id)
						.map(|req| req.requester == account)
						.unwrap_or(false)
				})
				.copied()
				.collect()
		}

		fn get_node_current_request(node: AccountId) -> Option<u64> {
			TeePrivacy::node_current_request(&node)
		}

		fn get_next_request_id() -> u64 {
			TeePrivacy::next_request_id()
		}

		fn get_pending_request_count() -> u32 {
			TeePrivacy::pending_requests().len() as u32
		}

		fn verify_attestation(
			mr_enclave: [u8; 32],
			mr_signer: [u8; 32],
			timestamp: u64,
		) -> pallet_tee_privacy::types::AttestationVerifyResult {
			let allowed_enclaves = TeePrivacy::allowed_mr_enclaves();
			let allowed_signers = TeePrivacy::allowed_mr_signers();

			let mr_enclave_match = allowed_enclaves.is_empty() || allowed_enclaves.contains(&mr_enclave);
			let mr_signer_match = allowed_signers.is_empty() || allowed_signers.contains(&mr_signer);

			// Check if attestation is expired (24 hours = 86400 seconds)
			let now = pallet_timestamp::Pallet::<Runtime>::get();
			let current_timestamp: u64 = now.try_into().ok().unwrap_or(0);
			let is_expired = timestamp < current_timestamp.saturating_sub(86400);

			let is_valid = mr_enclave_match && mr_signer_match && !is_expired;

			let error_message = if !mr_enclave_match {
				Some(b"MRENCLAVE not in allowed list".to_vec())
			} else if !mr_signer_match {
				Some(b"MRSIGNER not in allowed list".to_vec())
			} else if is_expired {
				Some(b"Attestation expired".to_vec())
			} else {
				None
			};

			pallet_tee_privacy::types::AttestationVerifyResult {
				is_valid,
				mr_enclave_match,
				is_expired,
				error_message,
			}
		}

		fn get_allowed_mr_enclaves() -> Vec<[u8; 32]> {
			TeePrivacy::allowed_mr_enclaves().to_vec()
		}

		fn get_allowed_mr_signers() -> Vec<[u8; 32]> {
			TeePrivacy::allowed_mr_signers().to_vec()
		}

		fn get_node_stake(account: AccountId) -> Option<(u128, Option<BlockNumber>, bool)> {
			TeePrivacy::node_stakes(&account).map(|stake| {
				let amount: u128 = stake.amount.try_into().ok().unwrap_or(0);
				(amount, stake.unlock_at, stake.is_unbonding)
			})
		}

		fn get_minimum_stake() -> u128 {
			use frame_support::traits::Get;
			let stake: Balance = <Runtime as pallet_tee_privacy::Config>::MinimumStake::get();
			stake.try_into().ok().unwrap_or(0)
		}

		fn get_total_slashed() -> u128 {
			TeePrivacy::total_slashed().try_into().ok().unwrap_or(0)
		}

		fn get_reward_pool() -> u128 {
			TeePrivacy::reward_pool().try_into().ok().unwrap_or(0)
		}

		fn is_audit_enabled() -> bool {
			TeePrivacy::audit_enabled()
		}

		fn get_account_audit_log_count(account: AccountId) -> u32 {
			TeePrivacy::account_audit_logs(&account).len() as u32
		}

		fn get_next_audit_log_id() -> u64 {
			TeePrivacy::next_audit_log_id()
		}
	}

	// ============================================================================
	// Livestream Runtime API
	// ============================================================================

	impl pallet_livestream::runtime_api::LivestreamApi<Block, AccountId, Balance> for Runtime {
		fn get_room(room_id: u64) -> Option<pallet_livestream::runtime_api::LiveRoomInfo<AccountId, Balance>> {
			Livestream::get_room_info(room_id)
		}

		fn get_host_room(host: AccountId) -> Option<u64> {
			Livestream::host_room(&host)
		}

		fn get_live_rooms() -> Vec<u64> {
			Livestream::get_live_room_ids()
		}

		fn get_gift(gift_id: u32) -> Option<pallet_livestream::runtime_api::GiftInfo<Balance>> {
			Livestream::get_gift_info(gift_id)
		}

		fn get_enabled_gifts() -> Vec<pallet_livestream::runtime_api::GiftInfo<Balance>> {
			Livestream::get_enabled_gifts()
		}

		fn has_ticket(room_id: u64, user: AccountId) -> bool {
			Livestream::has_ticket(room_id, &user)
		}

		fn is_blacklisted(room_id: u64, user: AccountId) -> bool {
			Livestream::is_blacklisted(room_id, &user)
		}

		fn get_host_earnings(host: AccountId) -> Balance {
			Livestream::host_earnings(&host)
		}

		fn get_user_room_gifts(room_id: u64, user: AccountId) -> Balance {
			Livestream::user_room_gifts(room_id, &user)
		}

		fn get_co_hosts(room_id: u64) -> Vec<AccountId> {
			Livestream::get_co_host_list(room_id)
		}
	}

	// ============================================================================
	// Bazi Chart Runtime API (八字命盘)
	// ============================================================================

	impl pallet_bazi_chart::runtime_api::BaziChartApi<Block, AccountId> for Runtime {
		fn get_interpretation(chart_id: u64) -> Option<pallet_bazi_chart::interpretation::FullInterpretation> {
			Bazi::get_full_interpretation(chart_id)
		}

		fn get_full_bazi_chart(chart_id: u64) -> Option<alloc::string::String> {
			Bazi::get_full_bazi_chart_for_api(chart_id).map(|chart| chart.to_debug_json())
		}

		fn chart_exists(chart_id: u64) -> bool {
			pallet_bazi_chart::ChartById::<Runtime>::contains_key(chart_id)
		}

		fn get_chart_owner(chart_id: u64) -> Option<AccountId> {
			pallet_bazi_chart::ChartById::<Runtime>::get(chart_id).map(|chart| chart.owner)
		}

		fn get_encrypted_chart_interpretation(chart_id: u64) -> Option<pallet_bazi_chart::interpretation::FullInterpretation> {
			Bazi::get_encrypted_chart_interpretation(chart_id)
		}

		fn encrypted_chart_exists(chart_id: u64) -> bool {
			Bazi::encrypted_chart_exists(chart_id)
		}

		fn get_encrypted_chart_owner(chart_id: u64) -> Option<AccountId> {
			Bazi::get_encrypted_chart_owner(chart_id)
		}

		fn calculate_bazi_temp_unified(
			input_type: u8,
			params: Vec<u16>,
			gender: u8,
			zishi_mode: u8,
		) -> Option<alloc::string::String> {
			Bazi::calculate_bazi_temp_unified(input_type, params, gender, zishi_mode)
				.map(|chart| chart.to_debug_json())
		}

		fn get_user_encryption_key(account: AccountId) -> Option<[u8; 32]> {
			use pallet_divination_privacy::UserEncryptionKeys;
			UserEncryptionKeys::<Runtime>::get(&account).map(|info| info.public_key)
		}

		fn get_service_provider(account: AccountId) -> Option<alloc::string::String> {
			use pallet_divination_privacy::ServiceProviders;
			ServiceProviders::<Runtime>::get(&account).map(|provider| {
				alloc::format!(
					r#"{{"provider_type":{},"public_key":"{}","reputation":{},"registered_at":{},"is_active":{}}}"#,
					provider.provider_type as u8,
					sp_core::hexdisplay::HexDisplay::from(&provider.public_key),
					provider.reputation,
					provider.registered_at,
					provider.is_active
				)
			})
		}

		fn get_providers_by_type(provider_type: u8) -> Vec<AccountId> {
			use pallet_divination_privacy::{ServiceProviders, types::ServiceProviderType};
			let provider_type_enum = match provider_type {
				0 => ServiceProviderType::MingLiShi,
				1 => ServiceProviderType::AiService,
				2 => ServiceProviderType::FamilyMember,
				3 => ServiceProviderType::Research,
				_ => return Vec::new(),
			};
			ServiceProviders::<Runtime>::iter()
				.filter(|(_, provider)| provider.provider_type == provider_type_enum && provider.is_active)
				.map(|(account, _)| account)
				.collect()
		}

		fn get_provider_grants(account: AccountId) -> Vec<u64> {
			use pallet_divination_privacy::ProviderGrants;
			// ProviderGrants stores RecordKey which contains (DivinationType, u64)
			// Extract just the u64 result_id from each RecordKey
			ProviderGrants::<Runtime>::get(&account)
				.into_inner()
				.into_iter()
				.map(|key| key.result_id)
				.collect()
		}

		fn get_multi_key_encrypted_chart_info(_chart_id: u64) -> Option<alloc::string::String> {
			// Multi-key encryption is not yet implemented, return None for now
			None
		}

		fn get_multi_key_encrypted_chart_interpretation(chart_id: u64) -> Option<pallet_bazi_chart::interpretation::FullInterpretation> {
			// Multi-key encryption is not yet implemented, use regular encrypted chart interpretation
			Bazi::get_encrypted_chart_interpretation(chart_id)
		}
	}

	impl pallet_storage_service::runtime_api::StorageServiceApi<Block, AccountId, Balance> for Runtime {
		fn get_user_funding_account(user: AccountId) -> AccountId {
			StorageService::derive_user_funding_account(&user)
		}

		fn get_user_funding_balance(user: AccountId) -> Balance {
			let funding_account = StorageService::derive_user_funding_account(&user);
			pallet_balances::Pallet::<Runtime>::free_balance(&funding_account)
		}

		fn get_subject_usage(user: AccountId, domain: u8, subject_id: u64) -> Balance {
			pallet_storage_service::SubjectUsage::<Runtime>::get((user, domain, subject_id))
		}

		fn get_user_all_usage(user: AccountId) -> Vec<(u8, u64, Balance)> {
			// 遍历 SubjectUsage 存储，筛选出该用户的所有记录
			pallet_storage_service::SubjectUsage::<Runtime>::iter()
				.filter_map(|((u, domain, subject_id), amount)| {
					if u == user {
						Some((domain, subject_id, amount))
					} else {
						None
					}
				})
				.collect()
		}
	}
}
