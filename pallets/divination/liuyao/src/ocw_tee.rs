//! # 六爻模块 OCW + TEE 集成
//!
//! 实现 `DivinationModule` trait，接入通用 OCW + TEE 架构。
//!
//! ## 功能
//!
//! - 实现 `DivinationModule` trait
//! - 定义六爻专用输入/输出类型
//! - 提供 JSON 清单生成
//! - 支持三种隐私模式

use crate::algorithm;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_divination_ocw_tee::{
    DivinationModule, DivinationType, ModuleError, PrivacyMode,
};
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 六爻输入类型（明文）====================

/// 六爻起卦方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum LiuyaoMethod {
    /// 铜钱起卦
    Coins = 0,
    /// 数字起卦
    Numbers = 1,
    /// 时间起卦
    Time = 2,
    /// 随机起卦
    Random = 3,
    /// 手动指定
    Manual = 4,
}

impl Default for LiuyaoMethod {
    fn default() -> Self {
        Self::Time
    }
}

/// 六爻明文输入
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct LiuyaoInputPlain {
    /// 起卦方式
    pub method: LiuyaoMethod,
    /// 六爻数据（6个值，每个0-3表示阴阳动静）
    /// 0: 老阴（变）, 1: 少阳, 2: 少阴, 3: 老阳（变）
    pub yao_values: Option<[u8; 6]>,
    /// 数字起卦 - 数字
    pub number: Option<u32>,
    /// 时间起卦 - 年
    pub year: Option<u16>,
    /// 时间起卦 - 月
    pub month: Option<u8>,
    /// 时间起卦 - 日
    pub day: Option<u8>,
    /// 时间起卦 - 时
    pub hour: Option<u8>,
    /// 日干（用于六神排布）
    pub day_gan: Option<u8>,
    /// 占问事宜（可选，最大128字节）
    pub question: Option<BoundedVec<u8, ConstU32<128>>>,
}

impl LiuyaoInputPlain {
    /// 验证输入有效性
    pub fn is_valid(&self) -> bool {
        match self.method {
            LiuyaoMethod::Coins | LiuyaoMethod::Manual => {
                // 需要六爻数据
                if let Some(yao) = &self.yao_values {
                    yao.iter().all(|&v| v <= 3)
                } else {
                    false
                }
            }
            LiuyaoMethod::Numbers => self.number.is_some(),
            LiuyaoMethod::Time => {
                self.year.is_some() && self.month.is_some() && 
                self.day.is_some() && self.hour.is_some()
            }
            LiuyaoMethod::Random => true,
        }
    }
}

// ==================== 六爻索引类型 ====================

/// 六爻索引（用于快速查询）
#[derive(Clone, Copy, Debug, Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct LiuyaoIndex {
    /// 本卦上卦索引 (0-7)
    pub ben_shang: u8,
    /// 本卦下卦索引 (0-7)
    pub ben_xia: u8,
    /// 变卦上卦索引 (0-7)
    pub bian_shang: u8,
    /// 变卦下卦索引 (0-7)
    pub bian_xia: u8,
    /// 动爻位置（位掩码，6位）
    pub dong_yao_mask: u8,
    /// 世爻位置 (1-6)
    pub shi_yao: u8,
    /// 应爻位置 (1-6)
    pub ying_yao: u8,
    /// 卦宫索引 (0-7)
    pub gong: u8,
}

// ==================== 六爻计算结果 ====================

/// 六爻计算结果
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct LiuyaoComputeResult {
    /// 索引
    pub index: LiuyaoIndex,
    /// 本卦六十四卦索引
    pub ben_gua_index: u8,
    /// 变卦六十四卦索引
    pub bian_gua_index: u8,
    /// 六爻纳甲数据
    pub yao_data: [[u8; 3]; 6], // [天干, 地支, 六亲]
    /// 六神数据
    pub liu_shen: [u8; 6],
    /// 旬空地支
    pub xun_kong: [u8; 2],
}

// ==================== DivinationModule 实现 ====================

/// 六爻模块 OCW + TEE 处理器
pub struct LiuyaoModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for LiuyaoModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::LiuYao;
    const MODULE_NAME: &'static str = "LiuYao";
    const VERSION: u32 = 1;

    type PlainInput = LiuyaoInputPlain;
    type Index = LiuyaoIndex;
    type Result = LiuyaoComputeResult;

    /// 执行六爻计算
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        // 验证输入
        <Self as DivinationModule<T>>::validate_input(input)?;

        // 根据起卦方式获取六爻数据
        let yao_values = match input.method {
            LiuyaoMethod::Coins | LiuyaoMethod::Manual => {
                input.yao_values.ok_or_else(|| ModuleError::invalid_input(b"Missing yao values"))?
            }
            LiuyaoMethod::Numbers => {
                let num = input.number.ok_or_else(|| ModuleError::invalid_input(b"Missing number"))?;
                algorithm::generate_yao_from_number(num)
            }
            LiuyaoMethod::Time => {
                // 使用时间生成六爻
                let year = input.year.unwrap_or(2024);
                let month = input.month.unwrap_or(1);
                let day = input.day.unwrap_or(1);
                let hour = input.hour.unwrap_or(0);
                algorithm::generate_yao_from_time(year, month, day, hour)
            }
            LiuyaoMethod::Random => {
                // 随机生成（在 OCW 中使用随机源）
                [1, 2, 1, 2, 1, 2] // 默认值，实际应由随机源生成
            }
        };

        // 计算本卦和变卦
        let (ben_shang, ben_xia, bian_shang, bian_xia, dong_mask) = 
            algorithm::calculate_gua_from_yao(&yao_values);

        // 计算世应
        let (shi_yao, ying_yao, gong) = algorithm::calculate_shi_ying(ben_shang, ben_xia);

        // 计算纳甲
        let yao_data = algorithm::calculate_najia_data(ben_shang, ben_xia, gong);

        // 计算六神
        let day_gan = input.day_gan.unwrap_or(0);
        let liu_shen = algorithm::calculate_liu_shen_u8(day_gan);

        // 计算旬空
        let xun_kong = algorithm::calculate_xun_kong_u8(day_gan, 0); // 简化：使用日干

        // 计算六十四卦索引
        let ben_gua_index = ben_shang * 8 + ben_xia;
        let bian_gua_index = bian_shang * 8 + bian_xia;

        Ok(LiuyaoComputeResult {
            index: LiuyaoIndex {
                ben_shang,
                ben_xia,
                bian_shang,
                bian_xia,
                dong_yao_mask: dong_mask,
                shi_yao,
                ying_yao,
                gong,
            },
            ben_gua_index,
            bian_gua_index,
            yao_data,
            liu_shen,
            xun_kong,
        })
    }

    /// 从计算结果提取索引
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Public | PrivacyMode::Encrypted => Some(result.index),
            PrivacyMode::Private => None,
        }
    }

    /// 生成 JSON 清单
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError> {
        // 简化实现：使用 SCALE 编码
        // 实际应使用 JSON 格式
        let manifest = (
            Self::VERSION,
            Self::MODULE_ID as u8,
            privacy_mode as u8,
            &result.index,
            result.ben_gua_index,
            result.bian_gua_index,
        );
        
        Ok(manifest.encode())
    }

    /// 验证输入
    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError> {
        if !input.is_valid() {
            return Err(ModuleError::invalid_input(b"Invalid LiuYao input"));
        }
        Ok(())
    }
}
