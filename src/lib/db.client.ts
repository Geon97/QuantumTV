/* eslint-disable no-console */
'use client';

/**
 * 仅在浏览器端使用的数据库工具，基于 localStorage 实现。
 *
 * 功能：
 * 1. 获取/保存播放记录
 * 2. 获取/保存收藏
 * 3. 获取/保存搜索历史
 * 4. 获取/保存跳过片头片尾配置
 */

import { SkipConfig } from './types';

// 全局错误触发函数
function triggerGlobalError(message: string) {
  if (typeof window !== 'undefined') {
    window.dispatchEvent(
      new CustomEvent('globalError', {
        detail: { message },
      }),
    );
  }
}

// ---- 类型 ----
export interface PlayRecord {
  title: string;
  source_name: string;
  year: string;
  cover: string;
  index: number; // 第几集
  total_episodes: number; // 总集数
  play_time: number; // 播放进度（秒）
  total_time: number; // 总进度（秒）
  save_time: number; // 记录保存时间（时间戳）
  search_title?: string; // 搜索时使用的标题
}

// ---- 收藏类型 ----
export interface Favorite {
  title: string;
  source_name: string;
  year: string;
  cover: string;
  total_episodes: number;
  save_time: number;
  search_title?: string;
  origin?: 'vod' | 'live';
}

// ---- 常量 ----
const PLAY_RECORDS_KEY = 'quantumtv_play_records';
const FAVORITES_KEY = 'quantumtv_favorites';
const SEARCH_HISTORY_KEY = 'quantumtv_search_history';
const SKIP_CONFIGS_KEY = 'quantumtv_skip_configs';

// 搜索历史最大保存条数
const SEARCH_HISTORY_LIMIT = 20;

/**
 * 生成存储key
 */
export function generateStorageKey(source: string, id: string): string {
  return `${source}+${id}`;
}

// ---- 播放记录 API ----

/**
 * 读取全部播放记录
 */
export async function getAllPlayRecords(): Promise<Record<string, PlayRecord>> {
  if (typeof window === 'undefined') {
    return {};
  }

  try {
    const raw = localStorage.getItem(PLAY_RECORDS_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as Record<string, PlayRecord>;
  } catch (err) {
    console.error('读取播放记录失败:', err);
    triggerGlobalError('读取播放记录失败');
    return {};
  }
}

/**
 * 保存播放记录
 */
export async function savePlayRecord(
  source: string,
  id: string,
  record: PlayRecord,
): Promise<void> {
  if (typeof window === 'undefined') {
    console.warn('无法在服务端保存播放记录到 localStorage');
    return;
  }

  const key = generateStorageKey(source, id);

  try {
    const allRecords = await getAllPlayRecords();
    allRecords[key] = record;
    localStorage.setItem(PLAY_RECORDS_KEY, JSON.stringify(allRecords));
    window.dispatchEvent(
      new CustomEvent('playRecordsUpdated', {
        detail: allRecords,
      }),
    );
  } catch (err) {
    console.error('保存播放记录失败:', err);
    triggerGlobalError('保存播放记录失败');
    throw err;
  }
}

/**
 * 删除播放记录
 */
export async function deletePlayRecord(
  source: string,
  id: string,
): Promise<void> {
  if (typeof window === 'undefined') {
    console.warn('无法在服务端删除播放记录到 localStorage');
    return;
  }

  const key = generateStorageKey(source, id);

  try {
    const allRecords = await getAllPlayRecords();
    delete allRecords[key];
    localStorage.setItem(PLAY_RECORDS_KEY, JSON.stringify(allRecords));
    window.dispatchEvent(
      new CustomEvent('playRecordsUpdated', {
        detail: allRecords,
      }),
    );
  } catch (err) {
    console.error('删除播放记录失败:', err);
    triggerGlobalError('删除播放记录失败');
    throw err;
  }
}

/**
 * 清空全部播放记录
 */
export async function clearAllPlayRecords(): Promise<void> {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(PLAY_RECORDS_KEY);
  window.dispatchEvent(
    new CustomEvent('playRecordsUpdated', {
      detail: {},
    }),
  );
}

// ---- 搜索历史 API ----

/**
 * 获取搜索历史
 */
export async function getSearchHistory(): Promise<string[]> {
  if (typeof window === 'undefined') {
    return [];
  }

  try {
    const raw = localStorage.getItem(SEARCH_HISTORY_KEY);
    if (!raw) return [];
    const arr = JSON.parse(raw) as string[];
    return Array.isArray(arr) ? arr : [];
  } catch (err) {
    console.error('读取搜索历史失败:', err);
    triggerGlobalError('读取搜索历史失败');
    return [];
  }
}

/**
 * 将关键字添加到搜索历史
 */
export async function addSearchHistory(keyword: string): Promise<void> {
  if (typeof window === 'undefined') return;

  const trimmed = keyword.trim();
  if (!trimmed) return;

  try {
    const history = await getSearchHistory();
    const newHistory = [trimmed, ...history.filter((k) => k !== trimmed)];
    if (newHistory.length > SEARCH_HISTORY_LIMIT) {
      newHistory.length = SEARCH_HISTORY_LIMIT;
    }
    localStorage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(newHistory));
    window.dispatchEvent(
      new CustomEvent('searchHistoryUpdated', {
        detail: newHistory,
      }),
    );
  } catch (err) {
    console.error('保存搜索历史失败:', err);
    triggerGlobalError('保存搜索历史失败');
  }
}

/**
 * 清空搜索历史
 */
export async function clearSearchHistory(): Promise<void> {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(SEARCH_HISTORY_KEY);
  window.dispatchEvent(
    new CustomEvent('searchHistoryUpdated', {
      detail: [],
    }),
  );
}

/**
 * 删除单条搜索历史
 */
export async function deleteSearchHistory(keyword: string): Promise<void> {
  if (typeof window === 'undefined') return;

  const trimmed = keyword.trim();
  if (!trimmed) return;

  try {
    const history = await getSearchHistory();
    const newHistory = history.filter((k) => k !== trimmed);
    localStorage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(newHistory));
    window.dispatchEvent(
      new CustomEvent('searchHistoryUpdated', {
        detail: newHistory,
      }),
    );
  } catch (err) {
    console.error('删除搜索历史失败:', err);
    triggerGlobalError('删除搜索历史失败');
  }
}

// ---- 收藏 API ----

/**
 * 获取全部收藏
 */
export async function getAllFavorites(): Promise<Record<string, Favorite>> {
  if (typeof window === 'undefined') {
    return {};
  }

  try {
    const raw = localStorage.getItem(FAVORITES_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as Record<string, Favorite>;
  } catch (err) {
    console.error('读取收藏失败:', err);
    triggerGlobalError('读取收藏失败');
    return {};
  }
}

/**
 * 保存收藏
 */
export async function saveFavorite(
  source: string,
  id: string,
  favorite: Favorite,
): Promise<void> {
  if (typeof window === 'undefined') {
    console.warn('无法在服务端保存收藏到 localStorage');
    return;
  }

  const key = generateStorageKey(source, id);

  try {
    const allFavorites = await getAllFavorites();
    allFavorites[key] = favorite;
    localStorage.setItem(FAVORITES_KEY, JSON.stringify(allFavorites));
    window.dispatchEvent(
      new CustomEvent('favoritesUpdated', {
        detail: allFavorites,
      }),
    );
  } catch (err) {
    console.error('保存收藏失败:', err);
    triggerGlobalError('保存收藏失败');
    throw err;
  }
}

/**
 * 删除收藏
 */
export async function deleteFavorite(
  source: string,
  id: string,
): Promise<void> {
  if (typeof window === 'undefined') {
    console.warn('无法在服务端删除收藏到 localStorage');
    return;
  }

  const key = generateStorageKey(source, id);

  try {
    const allFavorites = await getAllFavorites();
    delete allFavorites[key];
    localStorage.setItem(FAVORITES_KEY, JSON.stringify(allFavorites));
    window.dispatchEvent(
      new CustomEvent('favoritesUpdated', {
        detail: allFavorites,
      }),
    );
  } catch (err) {
    console.error('删除收藏失败:', err);
    triggerGlobalError('删除收藏失败');
    throw err;
  }
}

/**
 * 判断是否已收藏
 */
export async function isFavorited(
  source: string,
  id: string,
): Promise<boolean> {
  const key = generateStorageKey(source, id);
  const allFavorites = await getAllFavorites();
  return !!allFavorites[key];
}

/**
 * 清空全部收藏
 */
export async function clearAllFavorites(): Promise<void> {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(FAVORITES_KEY);
  window.dispatchEvent(
    new CustomEvent('favoritesUpdated', {
      detail: {},
    }),
  );
}

// ---- 跳过片头片尾配置 API ----

/**
 * 获取跳过片头片尾配置
 */
export async function getSkipConfig(
  source: string,
  id: string,
): Promise<SkipConfig | null> {
  if (typeof window === 'undefined') {
    return null;
  }

  const key = generateStorageKey(source, id);

  try {
    const raw = localStorage.getItem(SKIP_CONFIGS_KEY);
    if (!raw) return null;
    const configs = JSON.parse(raw) as Record<string, SkipConfig>;
    return configs[key] || null;
  } catch (err) {
    console.error('读取跳过片头片尾配置失败:', err);
    triggerGlobalError('读取跳过片头片尾配置失败');
    return null;
  }
}

/**
 * 保存跳过片头片尾配置
 */
export async function saveSkipConfig(
  source: string,
  id: string,
  config: SkipConfig,
): Promise<void> {
  if (typeof window === 'undefined') {
    console.warn('无法在服务端保存跳过片头片尾配置到 localStorage');
    return;
  }

  const key = generateStorageKey(source, id);

  try {
    const raw = localStorage.getItem(SKIP_CONFIGS_KEY);
    const configs = raw ? (JSON.parse(raw) as Record<string, SkipConfig>) : {};
    configs[key] = config;
    localStorage.setItem(SKIP_CONFIGS_KEY, JSON.stringify(configs));
    window.dispatchEvent(
      new CustomEvent('skipConfigsUpdated', {
        detail: configs,
      }),
    );
  } catch (err) {
    console.error('保存跳过片头片尾配置失败:', err);
    triggerGlobalError('保存跳过片头片尾配置失败');
    throw err;
  }
}

/**
 * 获取所有跳过片头片尾配置
 */
export async function getAllSkipConfigs(): Promise<Record<string, SkipConfig>> {
  if (typeof window === 'undefined') {
    return {};
  }

  try {
    const raw = localStorage.getItem(SKIP_CONFIGS_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as Record<string, SkipConfig>;
  } catch (err) {
    console.error('读取跳过片头片尾配置失败:', err);
    triggerGlobalError('读取跳过片头片尾配置失败');
    return {};
  }
}

/**
 * 删除跳过片头片尾配置
 */
export async function deleteSkipConfig(
  source: string,
  id: string,
): Promise<void> {
  if (typeof window === 'undefined') {
    console.warn('无法在服务端删除跳过片头片尾配置到 localStorage');
    return;
  }

  const key = generateStorageKey(source, id);

  try {
    const raw = localStorage.getItem(SKIP_CONFIGS_KEY);
    if (raw) {
      const configs = JSON.parse(raw) as Record<string, SkipConfig>;
      delete configs[key];
      localStorage.setItem(SKIP_CONFIGS_KEY, JSON.stringify(configs));
      window.dispatchEvent(
        new CustomEvent('skipConfigsUpdated', {
          detail: configs,
        }),
      );
    }
  } catch (err) {
    console.error('删除跳过片头片尾配置失败:', err);
    triggerGlobalError('删除跳过片头片尾配置失败');
    throw err;
  }
}

// ---- 事件订阅辅助 ----

export type CacheUpdateEvent =
  | 'playRecordsUpdated'
  | 'favoritesUpdated'
  | 'searchHistoryUpdated'
  | 'skipConfigsUpdated';

/**
 * 用于 React 组件监听数据更新的事件监听器
 */
export function subscribeToDataUpdates<T>(
  eventType: CacheUpdateEvent,
  callback: (data: T) => void,
): () => void {
  if (typeof window === 'undefined') {
    return () => {};
  }

  const handleUpdate = (event: CustomEvent) => {
    callback(event.detail);
  };

  window.addEventListener(eventType, handleUpdate as EventListener);

  return () => {
    window.removeEventListener(eventType, handleUpdate as EventListener);
  };
}
