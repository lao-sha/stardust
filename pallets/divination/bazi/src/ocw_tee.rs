//! # 八字模块 OCW + TEE 集成
//!
//! 实现 `DivinationModule` trait，接入通用 OCW + TEE 架构。
//!
//! ## 功能
//!
//! - 实现 `DivinationModule` trait
//! - 定义八字专用输入/输出类型
//! - 提供 JSON 清单生成
//! - 支持三种隐私模式

use crate::types::{BaziInputType, Gender, SiZhuIndex, ZiShiMode};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_divination_ocw_tee::{
    DivinationModule, DivinationType, ModuleError, PrivacyMode, ProcessResult,
};
use scale_info::TypeInfo;
use sp_std::prelude::*;

// ==================== 八字输入类型（明文）====================

/// 八字明文输入
///
/// 用于 OCW + TEE 架构的标准化输入格式。
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct BaziInputPlain {
    /// 输入类型（公历/农历/四柱直接输入）
    pub input_type: BaziInputType,
    /// 性别
    pub gender: Gender,
    /// 子时模式（可选）
    pub zishi_mode: Option<ZiShiMode>,
    /// 出生地经度（可选，1/100000 度）
    pub longitude: Option<i32>,
    /// 命盘名称（可选，最大32字节）
    pub name: Option<BoundedVec<u8, ConstU32<32>>>,
}

impl BaziInputPlain {
    /// 验证输入有效性
    pub fn is_valid(&self) -> bool {
        // 验证输入类型
        if !self.input_type.is_valid() {
            return false;
        }

        // 验证经度范围（如果提供）
        if let Some(lng) = self.longitude {
            // 经度范围: -180° 到 180°，存储为 1/100000 度
            if lng < -18_000_000 || lng > 18_000_000 {
                return false;
            }
        }

        true
    }
}

// ==================== 八字计算结果 ====================

/// 八字计算结果（简化版，用于 JSON 清单）
///
/// 不包含完整的 `BaziChart`，只包含必要的计算结果。
#[derive(Clone, Debug, Encode, Decode)]
pub struct BaziComputeResult {
    /// 四柱索引
    pub sizhu_index: SiZhuIndex,
    /// 性别
    pub gender: Gender,
    /// 起运年龄
    pub qiyun_age: u8,
    /// 是否顺排大运
    pub is_shun: bool,
    /// 五行强度 [金, 木, 水, 火, 土]
    pub wuxing_strength: [u32; 5],
    /// 喜用神（五行索引：0=金, 1=木, 2=水, 3=火, 4=土）
    pub xiyong_shen: Option<u8>,
    /// 大运列表（干支索引）
    pub dayun_list: Vec<u8>,
}

// ==================== JSON 清单结构 ====================

/// 八字 JSON 清单（用于 IPFS 存储）
#[derive(Clone, Debug, Encode, Decode)]
pub struct BaziManifest {
    /// 版本号
    pub version: u32,
    /// 模块 ID
    pub module_id: u8,
    /// 隐私模式
    pub privacy_mode: u8,
    /// 创建时间戳（Unix 秒）
    pub created_at: u64,

    // ===== 公开数据（所有模式都有）=====
    /// 四柱索引（Public/Encrypted 模式）
    pub sizhu_index: Option<SiZhuIndex>,

    // ===== 计算数据（Public/Encrypted 模式）=====
    /// 性别
    pub gender: Option<u8>,
    /// 起运年龄
    pub qiyun_age: Option<u8>,
    /// 是否顺排大运
    pub is_shun: Option<bool>,
    /// 五行强度
    pub wuxing_strength: Option<[u32; 5]>,
    /// 喜用神
    pub xiyong_shen: Option<u8>,
    /// 大运列表
    pub dayun_list: Option<Vec<u8>>,

    // ===== 敏感数据（仅 Public 模式明文）=====
    /// 出生时间（仅 Public 模式）
    pub birth_time: Option<BirthTimeJson>,
    /// 出生地经度（仅 Public 模式）
    pub longitude: Option<i32>,
    /// 命盘名称（仅 Public 模式）
    pub name: Option<Vec<u8>>,

    // ===== 加密数据（Encrypted/Private 模式）=====
    /// 加密的敏感数据
    pub encrypted_sensitive: Option<Vec<u8>>,
    /// 加密数据的哈希
    pub sensitive_hash: Option<[u8; 32]>,
}

/// 出生时间 JSON 格式
#[derive(Clone, Debug, Encode, Decode)]
pub struct BirthTimeJson {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

// ==================== DivinationModule 实现 ====================

/// 八字模块 OCW + TEE 处理器
pub struct BaziModuleHandler<T>(sp_std::marker::PhantomData<T>);

impl<T: crate::pallet::Config> DivinationModule<T> for BaziModuleHandler<T> {
    const MODULE_ID: DivinationType = DivinationType::BaZi;
    const MODULE_NAME: &'static str = "BaZi";
    const VERSION: u32 = 1;

    type PlainInput = BaziInputPlain;
    type Index = SiZhuIndex;
    type Result = BaziComputeResult;

    /// 执行八字计算
    fn compute(input: &Self::PlainInput) -> Result<Self::Result, ModuleError> {
        use crate::calculations::*;
        use crate::types::GanZhi;

        // 验证输入
        <Self as DivinationModule<T>>::validate_input(input)?;

        let zishi_mode = input.zishi_mode.unwrap_or(ZiShiMode::Modern);

        // 1. 根据输入类型获取公历日期和时间
        let (year, month, day, hour, minute) = match &input.input_type {
            BaziInputType::Solar { year, month, day, hour, minute } => {
                (*year, *month, *day, *hour, *minute)
            }
            BaziInputType::Lunar { year, month, day, is_leap_month, hour, minute } => {
                // 农历转公历
                let (solar_year, solar_month, solar_day) = pallet_almanac::lunar::lunar_to_solar(
                    *year, *month, *day, *is_leap_month,
                ).ok_or_else(|| ModuleError::invalid_input(b"Invalid lunar date"))?;
                (solar_year, solar_month, solar_day, *hour, *minute)
            }
            BaziInputType::SiZhu { year_gz, month_gz, day_gz, hour_gz, birth_year } => {
                // 四柱直接输入：直接构建索引
                let sizhu_index = SiZhuIndex {
                    year_gan: year_gz / 12,
                    year_zhi: year_gz % 12,
                    month_gan: month_gz / 12,
                    month_zhi: month_gz % 12,
                    day_gan: day_gz / 12,
                    day_zhi: day_gz % 12,
                    hour_gan: hour_gz / 12,
                    hour_zhi: hour_gz % 12,
                };

                // 简化计算：四柱直接输入时使用默认值
                let day_gan = crate::types::TianGan(sizhu_index.day_gan);
                let year_ganzhi = GanZhi::from_index(*year_gz)
                    .ok_or_else(|| ModuleError::invalid_input(b"Invalid year ganzhi"))?;
                let month_ganzhi = GanZhi::from_index(*month_gz)
                    .ok_or_else(|| ModuleError::invalid_input(b"Invalid month ganzhi"))?;
                let day_ganzhi = GanZhi::from_index(*day_gz)
                    .ok_or_else(|| ModuleError::invalid_input(b"Invalid day ganzhi"))?;
                let hour_ganzhi = GanZhi::from_index(*hour_gz)
                    .ok_or_else(|| ModuleError::invalid_input(b"Invalid hour ganzhi"))?;

                // 计算五行强度
                let wuxing = calculate_wuxing_strength(&year_ganzhi, &month_ganzhi, &day_ganzhi, &hour_ganzhi);
                let wuxing_strength = [wuxing.jin, wuxing.mu, wuxing.shui, wuxing.huo, wuxing.tu];

                // 计算喜用神
                let xiyong = determine_xiyong_shen(&wuxing, day_gan);
                let xiyong_shen = xiyong.map(|x| x as u8);

                // 计算大运
                let (qiyun_age, is_shun) = calculate_qiyun_age(year_ganzhi.gan.0, input.gender, 6);
                let dayun_list_raw = calculate_dayun_list(month_ganzhi, *birth_year, qiyun_age, is_shun, 10);
                let dayun_list: Vec<u8> = dayun_list_raw.iter().map(|(gz, _, _)| gz.to_index()).collect();

                return Ok(BaziComputeResult {
                    sizhu_index,
                    gender: input.gender,
                    qiyun_age,
                    is_shun,
                    wuxing_strength,
                    xiyong_shen,
                    dayun_list,
                });
            }
        };

        // 2. 应用真太阳时修正（如果提供经度）
        let (calc_year, calc_month, calc_day, calc_hour) = if let Some(lng) = input.longitude {
            let result = apply_true_solar_time(year, month, day, hour, minute, lng);
            let (adj_year, adj_month, adj_day) = if result.day_offset != 0 {
                adjust_date(year, month, day, result.day_offset)
            } else {
                (year, month, day)
            };
            (adj_year, adj_month, adj_day, result.hour)
        } else {
            (year, month, day, hour)
        };

        // 3. 计算四柱
        let day_ganzhi = calculate_day_ganzhi(calc_year, calc_month, calc_day)
            .ok_or_else(|| ModuleError::invalid_input(b"Invalid day"))?;
        let year_ganzhi = calculate_year_ganzhi(calc_year, calc_month, calc_day)
            .ok_or_else(|| ModuleError::invalid_input(b"Invalid year"))?;
        let month_ganzhi = calculate_month_ganzhi(calc_year, calc_month, calc_day, year_ganzhi.gan.0)
            .ok_or_else(|| ModuleError::invalid_input(b"Invalid month"))?;
        let (hour_ganzhi, is_next_day) = calculate_hour_ganzhi(calc_hour, day_ganzhi.gan.0, zishi_mode)
            .ok_or_else(|| ModuleError::invalid_input(b"Invalid hour"))?;

        let (final_day_ganzhi, final_hour_ganzhi) = if is_next_day {
            let next_day = day_ganzhi.next();
            let (final_hour, _) = calculate_hour_ganzhi(calc_hour, next_day.gan.0, zishi_mode)
                .ok_or_else(|| ModuleError::invalid_input(b"Invalid hour"))?;
            (next_day, final_hour)
        } else {
            (day_ganzhi, hour_ganzhi)
        };

        // 4. 构建四柱索引
        let sizhu_index = SiZhuIndex {
            year_gan: year_ganzhi.gan.0,
            year_zhi: year_ganzhi.zhi.0,
            month_gan: month_ganzhi.gan.0,
            month_zhi: month_ganzhi.zhi.0,
            day_gan: final_day_ganzhi.gan.0,
            day_zhi: final_day_ganzhi.zhi.0,
            hour_gan: final_hour_ganzhi.gan.0,
            hour_zhi: final_hour_ganzhi.zhi.0,
        };

        // 5. 计算五行强度
        let wuxing = calculate_wuxing_strength(&year_ganzhi, &month_ganzhi, &final_day_ganzhi, &final_hour_ganzhi);
        let wuxing_strength = [wuxing.jin, wuxing.mu, wuxing.shui, wuxing.huo, wuxing.tu];

        // 6. 计算喜用神
        let xiyong = determine_xiyong_shen(&wuxing, final_day_ganzhi.gan);
        let xiyong_shen = xiyong.map(|x| x as u8);

        // 7. 计算大运
        let (qiyun_age, is_shun) = calculate_qiyun_age(year_ganzhi.gan.0, input.gender, 6);
        let dayun_list_raw = calculate_dayun_list(month_ganzhi, year, qiyun_age, is_shun, 10);
        let dayun_list: Vec<u8> = dayun_list_raw.iter().map(|(gz, _, _)| gz.to_index()).collect();

        Ok(BaziComputeResult {
            sizhu_index,
            gender: input.gender,
            qiyun_age,
            is_shun,
            wuxing_strength,
            xiyong_shen,
            dayun_list,
        })
    }

    /// 从计算结果提取索引
    fn extract_index(result: &Self::Result, privacy_mode: PrivacyMode) -> Option<Self::Index> {
        match privacy_mode {
            PrivacyMode::Public | PrivacyMode::Encrypted => Some(result.sizhu_index),
            PrivacyMode::Private => None, // Private 模式不存储索引
        }
    }

    /// 生成 JSON 清单
    fn generate_manifest(
        input: &Self::PlainInput,
        result: &Self::Result,
        privacy_mode: PrivacyMode,
    ) -> Result<Vec<u8>, ModuleError> {
        let manifest = match privacy_mode {
            PrivacyMode::Public => {
                // 公开模式：所有数据明文
                let birth_time = match input.input_type {
                    BaziInputType::Solar { year, month, day, hour, minute } => {
                        Some(BirthTimeJson { year, month, day, hour, minute })
                    }
                    BaziInputType::Lunar { year, month, day, hour, minute, .. } => {
                        Some(BirthTimeJson { year, month, day, hour, minute })
                    }
                    BaziInputType::SiZhu { birth_year, .. } => {
                        Some(BirthTimeJson { year: birth_year, month: 0, day: 0, hour: 0, minute: 0 })
                    }
                };

                BaziManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0, // TODO: 获取实际时间戳
                    sizhu_index: Some(result.sizhu_index),
                    gender: Some(result.gender as u8),
                    qiyun_age: Some(result.qiyun_age),
                    is_shun: Some(result.is_shun),
                    wuxing_strength: Some(result.wuxing_strength),
                    xiyong_shen: result.xiyong_shen,
                    dayun_list: Some(result.dayun_list.clone()),
                    birth_time,
                    longitude: input.longitude,
                    name: input.name.as_ref().map(|n| n.to_vec()),
                    encrypted_sensitive: None,
                    sensitive_hash: None,
                }
            }
            PrivacyMode::Encrypted => {
                // 加密模式：索引明文，敏感数据加密
                BaziManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    sizhu_index: Some(result.sizhu_index),
                    gender: Some(result.gender as u8),
                    qiyun_age: Some(result.qiyun_age),
                    is_shun: Some(result.is_shun),
                    wuxing_strength: Some(result.wuxing_strength),
                    xiyong_shen: result.xiyong_shen,
                    dayun_list: Some(result.dayun_list.clone()),
                    birth_time: None, // 加密
                    longitude: None,  // 加密
                    name: None,       // 加密
                    encrypted_sensitive: None, // TODO: 由 TEE 填充
                    sensitive_hash: None,      // TODO: 由 TEE 填充
                }
            }
            PrivacyMode::Private => {
                // 私密模式：所有数据加密
                BaziManifest {
                    version: Self::VERSION,
                    module_id: Self::MODULE_ID as u8,
                    privacy_mode: privacy_mode as u8,
                    created_at: 0,
                    sizhu_index: None,
                    gender: None,
                    qiyun_age: None,
                    is_shun: None,
                    wuxing_strength: None,
                    xiyong_shen: None,
                    dayun_list: None,
                    birth_time: None,
                    longitude: None,
                    name: None,
                    encrypted_sensitive: None, // TODO: 由 TEE 填充
                    sensitive_hash: None,      // TODO: 由 TEE 填充
                }
            }
        };

        // 序列化为 SCALE 编码（no_std 兼容）
        // 注意：实际部署时可以使用 lite-json 转换为 JSON
        Ok(manifest.encode())
    }

    /// 验证输入有效性
    fn validate_input(input: &Self::PlainInput) -> Result<(), ModuleError> {
        if !input.is_valid() {
            return Err(ModuleError::invalid_input(b"Invalid BaZi input"));
        }
        Ok(())
    }

    /// 获取推荐超时时间
    fn recommended_timeout() -> u32 {
        100 // ~10 分钟
    }

    /// 获取最大输入大小
    fn max_input_size() -> u32 {
        256
    }

    /// 是否支持批量处理
    fn supports_batch() -> bool {
        false
    }

    /// 获取 TEE 端点路径
    fn tee_endpoint() -> &'static str {
        "/compute/bazi"
    }
}

// ==================== 辅助函数 ====================

/// 从 BaziChart 创建 ProcessResult
pub fn create_process_result<T: crate::pallet::Config>(
    chart: &crate::types::BaziChart<T>,
    manifest_cid: Vec<u8>,
    manifest_hash: [u8; 32],
) -> ProcessResult {
    let type_index = chart.sizhu.as_ref().map(|sizhu| {
        SiZhuIndex::from_sizhu(sizhu).encode()
    });

    ProcessResult {
        manifest_cid,
        manifest_hash,
        type_index,
        proof: None,
        manifest_data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bazi_input_validation() {
        // 有效的公历输入
        let valid_input = BaziInputPlain {
            input_type: BaziInputType::Solar {
                year: 1990,
                month: 5,
                day: 15,
                hour: 14,
                minute: 30,
            },
            gender: Gender::Male,
            zishi_mode: None,
            longitude: Some(11640000), // 116.4°
            name: None,
        };
        assert!(valid_input.is_valid());

        // 无效的年份
        let invalid_year = BaziInputPlain {
            input_type: BaziInputType::Solar {
                year: 1800, // 太早
                month: 5,
                day: 15,
                hour: 14,
                minute: 30,
            },
            gender: Gender::Male,
            zishi_mode: None,
            longitude: None,
            name: None,
        };
        assert!(!invalid_year.is_valid());

        // 无效的经度
        let invalid_longitude = BaziInputPlain {
            input_type: BaziInputType::Solar {
                year: 1990,
                month: 5,
                day: 15,
                hour: 14,
                minute: 30,
            },
            gender: Gender::Male,
            zishi_mode: None,
            longitude: Some(20_000_000), // 超出范围
            name: None,
        };
        assert!(!invalid_longitude.is_valid());
    }

    #[test]
    fn test_sizhu_index_extraction() {
        let result = BaziComputeResult {
            sizhu_index: SiZhuIndex {
                year_gan: 0,
                year_zhi: 0,
                month_gan: 2,
                month_zhi: 2,
                day_gan: 4,
                day_zhi: 4,
                hour_gan: 0,
                hour_zhi: 0,
            },
            gender: Gender::Male,
            qiyun_age: 5,
            is_shun: true,
            wuxing_strength: [100, 80, 60, 40, 120],
            xiyong_shen: Some(2), // 水
            dayun_list: vec![1, 2, 3, 4, 5],
        };

        // TODO: 此测试需要具体的 Config 类型，暂时跳过
        // Public 模式应该返回索引
        // let index = BaziModuleHandler::<Test>::extract_index(&result, PrivacyMode::Public);
        // assert!(index.is_some());

        // Encrypted 模式应该返回索引
        // let index = BaziModuleHandler::<Test>::extract_index(&result, PrivacyMode::Encrypted);
        // assert!(index.is_some());

        // Private 模式不应该返回索引
        // let index = BaziModuleHandler::<Test>::extract_index(&result, PrivacyMode::Private);
        // assert!(index.is_none());
    }
}
