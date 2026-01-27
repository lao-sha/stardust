//! # 脱敏函数模块
//!
//! 提供姓名、身份证、生日的脱敏处理

use sp_std::prelude::*;

/// 函数级详细中文注释：姓名脱敏
///
/// # 规则
/// - 0字：返回空
/// - 1字：返回 "×"
/// - 2字：前×，保留后，示例："张三" -> "×三"
/// - 3字：前后保留，中间×，示例："李四五" -> "李×五"
/// - 4字及以上：前1后1，中间×，示例:"王二麻子" -> "王×子"
///
/// # 参数
/// - full_name: 完整姓名（UTF-8字符串切片）
///
/// # 返回
/// - 脱敏后的姓名字节数组
pub fn mask_name(full_name: &str) -> Vec<u8> {
    extern crate alloc;
    use alloc::string::String;

    let chars: Vec<char> = full_name.chars().collect();
    let len = chars.len();

    let mut masked = String::new();
    match len {
        0 => {},
        1 => masked.push('×'),
        2 => {
            masked.push('×');
            masked.push(chars[1]);
        },
        3 => {
            masked.push(chars[0]);
            masked.push('×');
            masked.push(chars[2]);
        },
        _ => {
            masked.push(chars[0]);
            masked.push('×');
            masked.push(chars[len - 1]);
        },
    }

    masked.as_bytes().to_vec()
}

/// 函数级详细中文注释：身份证号脱敏
///
/// # 规则
/// - 18位：前4位 + 10个星号 + 后4位
/// - 15位：前4位 + 7个星号 + 后4位
/// - 少于8位：全部用星号替换
///
/// # 参数
/// - id_card: 完整身份证号（ASCII字符串切片）
///
/// # 返回
/// - 脱敏后的身份证号字节数组
pub fn mask_id_card(id_card: &str) -> Vec<u8> {
    extern crate alloc;
    use alloc::string::String;

    let len = id_card.len();

    if len < 8 {
        let masked: String = (0..len).map(|_| '*').collect();
        return masked.as_bytes().to_vec();
    }

    let front = &id_card[0..4];
    let back = &id_card[len - 4..];
    let middle_count = len - 8;

    let mut masked = String::new();
    masked.push_str(front);
    for _ in 0..middle_count {
        masked.push('*');
    }
    masked.push_str(back);

    masked.as_bytes().to_vec()
}

/// 函数级详细中文注释：生日脱敏
///
/// # 规则
/// - 标准格式（YYYY-MM-DD）：保留年份，月日用xx替换
/// - 示例："1990-01-01" -> "1990-xx-xx"
/// - 少于4字符：全部用****-xx-xx替换
///
/// # 参数
/// - birthday: 完整生日（ASCII字符串切片，格式 YYYY-MM-DD）
///
/// # 返回
/// - 脱敏后的生日字节数组
pub fn mask_birthday(birthday: &str) -> Vec<u8> {
    extern crate alloc;

    if birthday.len() >= 4 {
        let year = &birthday[0..4];
        let masked = alloc::format!("{}-xx-xx", year);
        masked.as_bytes().to_vec()
    } else {
        b"****-xx-xx".to_vec()
    }
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_name() {
        assert_eq!(mask_name(""), b"");
        assert_eq!(mask_name("李"), "×".as_bytes());
        assert_eq!(mask_name("张三"), "×三".as_bytes());
        assert_eq!(mask_name("李四五"), "李×五".as_bytes());
        assert_eq!(mask_name("王二麻子"), "王×子".as_bytes());
    }

    #[test]
    fn test_mask_id_card() {
        assert_eq!(mask_id_card("110101199001011234"), b"1101**********1234");
        assert_eq!(mask_id_card("110101900101123"), b"1101*******1123"); // 15位身份证：前4+7星+后4
        assert_eq!(mask_id_card("1234567"), b"*******");
    }

    #[test]
    fn test_mask_birthday() {
        assert_eq!(mask_birthday("1990-01-01"), b"1990-xx-xx");
        assert_eq!(mask_birthday("123"), b"****-xx-xx");
    }
}
