//! # 验证函数模块
//!
//! 提供 TRON 地址验证


/// 函数级详细中文注释：验证 TRON 地址格式
///
/// # 规则
/// - 长度：34 字符
/// - 开头：'T'
/// - 编码：Base58（字符集：1-9, A-H, J-N, P-Z, a-k, m-z）
///
/// # 参数
/// - address: TRON 地址字节数组
///
/// # 返回
/// - bool: 有效返回 true，无效返回 false
pub fn is_valid_tron_address(address: &[u8]) -> bool {
    // 长度检查
    if address.len() != 34 {
        return false;
    }

    // 开头检查
    if address[0] != b'T' {
        return false;
    }

    // Base58 字符集检查
    const BASE58_CHARS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    for &byte in address {
        if !BASE58_CHARS.contains(&byte) {
            return false;
        }
    }

    true
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_tron_address() {
        assert!(is_valid_tron_address(b"TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS"));
        assert!(!is_valid_tron_address(b"TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHL")); // 长度不对（33字符）
        assert!(!is_valid_tron_address(b"AYASr5UV6HEcXatwdFQfmLVUqQQQMUxHLS")); // 不是T开头
        assert!(!is_valid_tron_address(b"TYASr5UV6HEcXatwdFQfmLVUqQQQMUxHL0")); // 包含0（非Base58）
    }
}
