/**
 * CSP Nonce 注入脚本
 * 
 * 在构建时或运行时注入 nonce 到 HTML 和脚本标签
 */

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

/**
 * 生成 nonce
 */
function generateNonce() {
  return crypto.randomBytes(16).toString('base64');
}

/**
 * 读取 HTML 文件
 */
function readHtmlFile(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch (error) {
    console.error(`Failed to read ${filePath}:`, error);
    return null;
  }
}

/**
 * 写入 HTML 文件
 */
function writeHtmlFile(filePath, content) {
  try {
    fs.writeFileSync(filePath, content, 'utf8');
    console.log(`✅ Updated ${filePath}`);
  } catch (error) {
    console.error(`Failed to write ${filePath}:`, error);
  }
}

/**
 * 注入 nonce 到 HTML
 */
function injectNonceToHtml(html, nonce) {
  // 替换 CSP meta 标签中的 nonce 占位符
  let updated = html.replace(
    /<meta\s+http-equiv=["']Content-Security-Policy["']\s+content=["']([^"']+)["']/gi,
    (match, csp) => {
      // 替换 nonce 占位符
      const updatedCsp = csp.replace(/\{NONCE\}/g, nonce);
      return `<meta http-equiv="Content-Security-Policy" content="${updatedCsp}"`;
    }
  );

  // 如果没有找到 CSP 标签，添加一个
  if (!updated.includes('Content-Security-Policy')) {
    const csp = `default-src 'self'; script-src 'self' 'nonce-${nonce}'; style-src 'self' 'nonce-${nonce}';`;
    const cspMeta = `<meta http-equiv="Content-Security-Policy" content="${csp}">`;
    
    // 插入到 head 标签中
    updated = updated.replace(
      /<head[^>]*>/i,
      `$&${cspMeta}`
    );
  }

  // 为所有 script 标签添加 nonce
  updated = updated.replace(
    /<script(?![^>]*nonce=)([^>]*)>/gi,
    `<script nonce="${nonce}"$1>`
  );

  // 为所有 style 标签添加 nonce
  updated = updated.replace(
    /<style(?![^>]*nonce=)([^>]*)>/gi,
    `<style nonce="${nonce}"$1>`
  );

  return updated;
}

/**
 * 主函数
 */
function main() {
  const htmlPath = path.join(__dirname, '../public/index.html');
  const nonce = generateNonce();

  console.log(`🔐 Generating CSP nonce: ${nonce}`);

  const html = readHtmlFile(htmlPath);
  if (!html) {
    console.error('❌ Failed to read HTML file');
    process.exit(1);
  }

  const updatedHtml = injectNonceToHtml(html, nonce);
  writeHtmlFile(htmlPath, updatedHtml);

  // 将 nonce 保存到环境变量文件（可选）
  const envPath = path.join(__dirname, '../.env.local');
  try {
    let envContent = '';
    if (fs.existsSync(envPath)) {
      envContent = fs.readFileSync(envPath, 'utf8');
    }
    
    // 更新或添加 nonce
    if (envContent.includes('EXPO_PUBLIC_CSP_NONCE=')) {
      envContent = envContent.replace(
        /EXPO_PUBLIC_CSP_NONCE=.*/,
        `EXPO_PUBLIC_CSP_NONCE=${nonce}`
      );
    } else {
      envContent += `\nEXPO_PUBLIC_CSP_NONCE=${nonce}\n`;
    }
    
    fs.writeFileSync(envPath, envContent, 'utf8');
    console.log(`✅ Saved nonce to .env.local`);
  } catch (error) {
    console.warn('⚠️  Failed to save nonce to .env.local:', error.message);
  }

  console.log('✅ CSP nonce injection complete');
}

if (require.main === module) {
  main();
}

module.exports = { generateNonce, injectNonceToHtml };

