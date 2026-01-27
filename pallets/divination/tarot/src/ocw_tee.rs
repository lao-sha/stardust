//! # 塔罗牌模块 OCW + TEE 集成
//!
//! 实现 `DivinationModule` trait，接入通用 OCW + TEE 架构。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_divination_ocw_tee::{
    DivinationModule, DivinationType, ModuleError, PrivacyMode,
};
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 塔罗输入类型 ====================

/// 塔罗起卦方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum TarotMethod {
    Random = 0,
    Time = 1,
    Numbers = 2,
    Manual = 3,
    RandomWithCut = 4,
}

impl Default for TarotMethod {
    fn default() -> Self {
        Self::Random
    }
}

/// 塔罗明文输入
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TarotInputPlain {
    /// 起卦方式
    pub method: TarotMethod,
    /// 牌阵类型
    pub spread_type: u8,
    /// 手动指定的牌（牌ID列表）
    pub cards: Option<BoundedVec<u8, ConstU32<12>>>,
    /// 手动指定的正逆位
    pub reversed: Option<BoundedVec<bool, ConstU32<12>>>,
    /// 数字起卦用的数字
    pub number: Option<u32>,
    /// 切牌位置
    pub cut_position: Option<u8>,
    /// 占问事宜
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
}

impl TarotInputPlain {
    pub fn is_valid(&self) -> bool {
        match self.method {
            TarotMethod::Manual => self.cards.is_some(),
            TarotMethod::Numbers => self.number.is_some(),
            _ => true,
        }
    }
}

// ==================== 塔罗索引类型 ====================

/// 塔罗索引
#[derive(Clone, Copy, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TarotIndex {
    /// 牌阵类型
    pub spread_type: u8,
    /// 牌数量
    pub card_count: u8,
    /// 第一张牌ID（用于快速索引）
    pub first_card: u8,
}

// ==================== 塔罗计算结果 ====================

/// 塔罗计算结果
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct TarotComputeResult {
    pub index: TarotIndex,
    /// 抽到的牌ID列表
    pub cards: Vec<u8>,
    /// 正逆位列表
    pub reversed: Vec<bool>,
    /// 牌阵类型
    pub spread_type: u8,
}

// ==================== DivinationModule 实现 ====================

pub struct TarotModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for TarotModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::Tarot;
    const MODULE_NAME: &'static str = "Tarot";
    const VERSION: u32 = 1;

    type PlainInput = TarotInputPlain;
    type Index = TarotIndex;
    type Result = TarotComputeResult;

    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        <Self as DivinationModule<T>>::validate_input(input)?;

        let (cards, reversed) = match input.method {
            TarotMethod::Manual => {
                let c = input.cards.as_ref()
                    .ok_or_else(|| ModuleError::invalid_input(b"Missing cards"))?;
                let r = input.reversed.as_ref()
                    .map(|v| v.to_vec())
                    .unwrap_or_else(|| vec![false; c.len()]);
                (c.to_vec(), r)
            }
            TarotMethod::Numbers => {
                let num = input.number.unwrap_or(0);
                let card_count = crate::algorithm::get_spread_card_count(input.spread_type);
                let cards = crate::algorithm::generate_cards_from_number(num, card_count);
                let reversed = crate::algorithm::generate_reversed_from_number(num, card_count);
                (cards, reversed)
            }
            _ => {
                // Random/Time/RandomWithCut - 需要随机源
                let card_count = crate::algorithm::get_spread_card_count(input.spread_type);
                let cards = (0..card_count).map(|i| i as u8).collect();
                let reversed = vec![false; card_count as usize];
                (cards, reversed)
            }
        };

        let first_card = cards.first().copied().unwrap_or(0);

        Ok(TarotComputeResult {
            index: TarotIndex {
                spread_type: input.spread_type,
                card_count: cards.len() as u8,
                first_card,
            },
            cards,
            reversed,
            spread_type: input.spread_type,
        })
    }

    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Public | PrivacyMode::Encrypted => Some(result.index),
            PrivacyMode::Private => None,
        }
    }

    fn generate_manifest(
        _input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError> {
        let manifest = (
            Self::VERSION,
            Self::MODULE_ID as u8,
            privacy_mode as u8,
            &result.index,
            &result.cards,
            &result.reversed,
        );
        Ok(manifest.encode())
    }

    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError> {
        if !input.is_valid() {
            return Err(ModuleError::invalid_input(b"Invalid Tarot input"));
        }
        Ok(())
    }
}
