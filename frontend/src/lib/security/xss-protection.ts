/**
 * 星尘玄鉴 - XSS 防护工具
 * 
 * 提供多层 XSS 防护：
 * 1. 输入验证和清理
 * 2. 输出编码
 * 3. DOM 操作安全封装
 * 4. 敏感数据保护
 */

// ==================== HTML 实体编码 ====================

const HTML_ENTITIES: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#x27;',
  '/': '&#x2F;',
  '`': '&#x60;',
  '=': '&#x3D;',
};

/**
 * HTML 实体编码（防止 XSS）
 */
export function escapeHtml(str: string): string {
  return str.replace(/[&<>"'`=/]/g, (char) => HTML_ENTITIES[char] || char);
}

/**
 * 解码 HTML 实体
 */
export function unescapeHtml(str: string): string {
  const doc = new DOMParser().parseFromString(str, 'text/html');
  return doc.documentElement.textContent || '';
}

// ==================== URL 安全验证 ====================

/**
 * 验证 URL 是否安全（防止 javascript: 协议等）
 */
export function isSafeUrl(url: string): boolean {
  try {
    const parsed = new URL(url, window.location.origin);
    const safeProtocols = ['http:', 'https:', 'mailto:', 'tel:'];
    return safeProtocols.includes(parsed.protocol);
  } catch {
    return false;
  }
}

/**
 * 清理 URL（移除危险协议）
 */
export function sanitizeUrl(url: string): string {
  if (!isSafeUrl(url)) {
    return 'about:blank';
  }
  return url;
}

// ==================== 输入验证 ====================

/**
 * 验证地址格式（Substrate SS58）
 */
export function isValidAddress(address: string): boolean {
  // SS58 地址格式：以数字开头，长度 47-48
  return /^[1-9A-HJ-NP-Za-km-z]{47,48}$/.test(address);
}

/**
 * 验证助记词格式
 */
export function isValidMnemonicFormat(mnemonic: string): boolean {
  const words = mnemonic.trim().split(/\s+/);
  // 只允许字母和空格
  const isClean = /^[a-z\s]+$/i.test(mnemonic);
  const validLength = words.length === 12 || words.length === 24;
  return isClean && validLength;
}

/**
 * 验证金额格式
 */
export function isValidAmount(amount: string): boolean {
  return /^\d+(\.\d{1,12})?$/.test(amount) && parseFloat(amount) > 0;
}

/**
 * 清理用户输入（移除潜在危险字符）
 */
export function sanitizeInput(input: string): string {
  return input
    .replace(/<[^>]*>/g, '') // 移除 HTML 标签
    .replace(/javascript:/gi, '') // 移除 javascript: 协议
    .replace(/on\w+=/gi, '') // 移除事件处理器
    .replace(/data:/gi, '') // 移除 data: 协议
    .trim();
}

// ==================== 敏感数据保护 ====================

/**
 * 掩码地址（显示前6后4）
 */
export function maskAddress(address: string): string {
  if (address.length < 12) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

/**
 * 掩码助记词（只显示首尾词）
 */
export function maskMnemonic(mnemonic: string): string {
  const words = mnemonic.split(' ');
  if (words.length < 3) return '***';
  return `${words[0]} ... ${words[words.length - 1]}`;
}

/**
 * 安全日志（自动掩码敏感数据）
 */
export function secureLog(message: string, data?: unknown): void {
  if (process.env.NODE_ENV === 'production') {
    // 生产环境不输出敏感日志
    return;
  }

  let sanitizedData = data;
  
  if (typeof data === 'object' && data !== null) {
    sanitizedData = JSON.parse(JSON.stringify(data), (key, value) => {
      // 掩码敏感字段
      const sensitiveKeys = ['mnemonic', 'password', 'privateKey', 'secret', 'seed'];
      if (sensitiveKeys.some(k => key.toLowerCase().includes(k))) {
        return '[REDACTED]';
      }
      // 掩码地址
      if (key === 'address' && typeof value === 'string' && value.length > 20) {
        return maskAddress(value);
      }
      return value;
    });
  }

  console.log(`[Secure] ${message}`, sanitizedData);
}

// ==================== DOM 安全操作 ====================

/**
 * 安全设置文本内容（防止 XSS）
 */
export function safeSetTextContent(element: Element, text: string): void {
  element.textContent = text; // textContent 自动转义
}

/**
 * 安全设置 innerHTML（使用 DOMPurify 风格的清理）
 */
export function safeSetInnerHTML(element: Element, html: string): void {
  // 创建临时容器
  const temp = document.createElement('div');
  temp.innerHTML = html;

  // 移除危险元素
  const dangerousTags = ['script', 'iframe', 'object', 'embed', 'form'];
  dangerousTags.forEach(tag => {
    temp.querySelectorAll(tag).forEach(el => el.remove());
  });

  // 移除危险属性
  const dangerousAttrs = ['onclick', 'onerror', 'onload', 'onmouseover', 'onfocus'];
  temp.querySelectorAll('*').forEach(el => {
    dangerousAttrs.forEach(attr => el.removeAttribute(attr));
    // 移除 javascript: 链接
    if (el.getAttribute('href')?.startsWith('javascript:')) {
      el.removeAttribute('href');
    }
    if (el.getAttribute('src')?.startsWith('javascript:')) {
      el.removeAttribute('src');
    }
  });

  element.innerHTML = temp.innerHTML;
}

// ==================== 剪贴板安全 ====================

/**
 * 安全复制到剪贴板（自动清理）
 */
export async function secureCopyToClipboard(
  text: string,
  clearAfterMs: number = 60000
): Promise<void> {
  await navigator.clipboard.writeText(text);
  
  // 设置定时清理
  if (clearAfterMs > 0) {
    setTimeout(async () => {
      try {
        const current = await navigator.clipboard.readText();
        if (current === text) {
          await navigator.clipboard.writeText('');
        }
      } catch {
        // 忽略权限错误
      }
    }, clearAfterMs);
  }
}

// ==================== 防止点击劫持 ====================

/**
 * 检测是否在 iframe 中运行
 */
export function isInIframe(): boolean {
  try {
    return window.self !== window.top;
  } catch {
    return true; // 跨域 iframe 会抛出错误
  }
}

/**
 * 防止点击劫持（如果在 iframe 中则阻止）
 */
export function preventClickjacking(): void {
  if (isInIframe()) {
    // 清空页面内容
    document.body.innerHTML = '<h1>安全错误：不允许在 iframe 中运行</h1>';
    throw new Error('Clickjacking detected: Application cannot run in iframe');
  }
}

// ==================== 导出安全配置 ====================

/**
 * 推荐的 CSP 配置
 */
export const RECOMMENDED_CSP = {
  'default-src': ["'self'"],
  'script-src': ["'self'"],
  'style-src': ["'self'", "'unsafe-inline'"], // React Native Web 需要
  'img-src': ["'self'", 'data:', 'https:'],
  'font-src': ["'self'"],
  'connect-src': ["'self'", 'wss:', 'https:'], // WebSocket 连接
  'frame-ancestors': ["'none'"], // 防止点击劫持
  'form-action': ["'self'"],
  'base-uri': ["'self'"],
  'object-src': ["'none'"],
};

/**
 * 生成 CSP 字符串
 */
export function generateCspString(): string {
  return Object.entries(RECOMMENDED_CSP)
    .map(([directive, values]) => `${directive} ${values.join(' ')}`)
    .join('; ');
}
