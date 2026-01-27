/**
 * 星尘玄鉴 - CSP (Content Security Policy) 管理
 * 
 * 提供 CSP nonce 生成和管理功能
 */

/**
 * 生成加密安全的 nonce
 */
export function generateNonce(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return Array.from(array)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * 获取或生成 nonce（单例模式）
 */
let cachedNonce: string | null = null;

export function getNonce(): string {
  if (!cachedNonce) {
    cachedNonce = generateNonce();
  }
  return cachedNonce;
}

/**
 * 重置 nonce（用于测试或重新生成）
 */
export function resetNonce(): void {
  cachedNonce = null;
}

/**
 * CSP 配置（生产环境 - 严格）
 */
export const PRODUCTION_CSP = {
  'default-src': ["'self'"],
  'script-src': ["'self'"], // 使用 nonce，不包含 'unsafe-inline'
  'style-src': ["'self'"], // 使用 nonce，不包含 'unsafe-inline'
  'img-src': ["'self'", 'data:', 'https:'],
  'font-src': ["'self'", 'data:'],
  'connect-src': ["'self'", 'ws:', 'wss:', 'https:'],
  'frame-ancestors': ["'none'"],
  'base-uri': ["'self'"],
  'form-action': ["'self'"],
  'object-src': ["'none'"],
  'upgrade-insecure-requests': [],
} as const;

/**
 * CSP 配置（开发环境 - 宽松）
 * 
 * 注意：开发环境可能需要 'unsafe-eval' 用于热重载
 */
export const DEVELOPMENT_CSP = {
  ...PRODUCTION_CSP,
  'script-src': ["'self'", "'unsafe-eval'"], // 开发环境允许 eval（热重载）
  'style-src': ["'self'", "'unsafe-inline'"], // 开发环境允许内联样式
} as const;

/**
 * 生成 CSP 字符串
 */
export function generateCspString(
  config: typeof PRODUCTION_CSP | typeof DEVELOPMENT_CSP,
  nonce?: string
): string {
  const directives: string[] = [];

  for (const [directive, values] of Object.entries(config)) {
    if (values.length === 0 && directive === 'upgrade-insecure-requests') {
      directives.push(directive);
    } else if (values.length > 0) {
      let valueString = values.join(' ');
      
      // 添加 nonce（如果提供）
      if (nonce && (directive === 'script-src' || directive === 'style-src')) {
        valueString = `'nonce-${nonce}' ${valueString}`;
      }
      
      directives.push(`${directive} ${valueString}`);
    }
  }

  return directives.join('; ');
}

/**
 * 获取当前环境的 CSP 配置
 */
export function getCspConfig(): typeof PRODUCTION_CSP | typeof DEVELOPMENT_CSP {
  const isDev = process.env.NODE_ENV === 'development' || __DEV__;
  return isDev ? DEVELOPMENT_CSP : PRODUCTION_CSP;
}

/**
 * 生成完整的 CSP 字符串（包含 nonce）
 */
export function getCspString(): string {
  const config = getCspConfig();
  const nonce = getNonce();
  return generateCspString(config, nonce);
}

/**
 * 验证 CSP 配置
 */
export function validateCsp(cspString: string): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  // 检查是否包含 'unsafe-inline'（生产环境不应包含）
  if (process.env.NODE_ENV === 'production') {
    if (cspString.includes("'unsafe-inline'")) {
      errors.push("生产环境不应包含 'unsafe-inline'");
    }
    if (cspString.includes("'unsafe-eval'")) {
      errors.push("生产环境不应包含 'unsafe-eval'");
    }
  }

  // 检查是否包含 nonce
  if (!cspString.includes("'nonce-")) {
    errors.push("CSP 应包含 nonce 以替代 'unsafe-inline'");
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

