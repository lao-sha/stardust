/**
 * CSP 服务器中间件示例
 * 
 * 用于在服务器端动态生成和注入 CSP nonce
 * 
 * 支持：
 * - Express.js
 * - Next.js
 * - Koa.js
 * - 其他 Node.js 框架
 */

const crypto = require('crypto');

/**
 * 生成 nonce
 */
function generateNonce() {
  return crypto.randomBytes(16).toString('base64');
}

/**
 * Express.js 中间件
 */
function expressCspMiddleware(req, res, next) {
  const nonce = generateNonce();
  
  // 将 nonce 附加到请求对象
  res.locals.cspNonce = nonce;
  req.cspNonce = nonce;
  
  // 设置 CSP 响应头
  const csp = buildCspString(nonce, req);
  res.setHeader('Content-Security-Policy', csp);
  
  next();
}

/**
 * Next.js 中间件
 */
function nextjsCspMiddleware(req, res, next) {
  const nonce = generateNonce();
  
  // 设置 CSP 响应头
  const csp = buildCspString(nonce, req);
  res.setHeader('Content-Security-Policy', csp);
  
  // 将 nonce 注入到 HTML（通过 _document.js 或 _app.js）
  res.locals.cspNonce = nonce;
  
  next();
}

/**
 * Koa.js 中间件
 */
async function koaCspMiddleware(ctx, next) {
  const nonce = generateNonce();
  
  // 设置 CSP 响应头
  const csp = buildCspString(nonce, ctx.request);
  ctx.set('Content-Security-Policy', csp);
  
  // 将 nonce 附加到上下文
  ctx.state.cspNonce = nonce;
  
  await next();
}

/**
 * 构建 CSP 字符串
 */
function buildCspString(nonce, req) {
  const isDev = process.env.NODE_ENV === 'development';
  
  const directives = [
    "default-src 'self'",
    `script-src 'self' 'nonce-${nonce}'${isDev ? " 'unsafe-eval'" : ''}`,
    `style-src 'self' 'nonce-${nonce}'${isDev ? " 'unsafe-inline'" : ''}`,
    "img-src 'self' data: https:",
    "font-src 'self' data:",
    "connect-src 'self' ws: wss: https:",
    "frame-ancestors 'none'",
    "base-uri 'self'",
    "form-action 'self'",
    "object-src 'none'",
    "upgrade-insecure-requests",
  ];
  
  return directives.join('; ');
}

/**
 * HTML 模板辅助函数（用于服务器端渲染）
 */
function injectNonceToHtml(html, nonce) {
  return html
    .replace(/{NONCE}/g, nonce)
    .replace(
      /<script(?![^>]*nonce=)([^>]*)>/gi,
      `<script nonce="${nonce}"$1>`
    )
    .replace(
      /<style(?![^>]*nonce=)([^>]*)>/gi,
      `<style nonce="${nonce}"$1>`
    );
}

module.exports = {
  generateNonce,
  expressCspMiddleware,
  nextjsCspMiddleware,
  koaCspMiddleware,
  buildCspString,
  injectNonceToHtml,
};

