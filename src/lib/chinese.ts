/* eslint-disable no-console */

import { Converter } from 'opencc-js';

// 提供繁体 -> 简体 的转换函数。
// 使用 opencc-js (纯 JS 实现) 替代原生 opencc，避免 .node 模块加载问题。
// 支持港台繁体到大陆简体的精准转换。

// 创建转换器实例 (tw -> cn)
// 使用 Taiwan 变体，因为它覆盖了更广泛的繁体中文习惯
const converter = Converter({ from: 'tw', to: 'cn' });

export async function toSimplified(text: string): Promise<string> {
  if (!text) return text;
  try {
    return converter(text);
  } catch (e) {
    console.warn('繁体转简体失败:', e);
    return text;
  }
}

export default toSimplified;
