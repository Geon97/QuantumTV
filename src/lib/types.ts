import { AdminConfig } from '@/lib/admin.types';

export interface ApiSite {
  key: string;
  api: string;
  name: string;
  detail?: string;
  is_adult?: boolean;
}
// 播放记录数据结构
export interface PlayRecord {
  title: string;
  source_name: string;
  cover: string;
  year: string;
  index: number; // 第几集
  total_episodes: number; // 总集数
  play_time: number; // 播放进度（秒）
  total_time: number; // 总进度（秒）
  save_time: number; // 记录保存时间（时间戳）
  search_title: string; // 搜索时使用的标题
}

// 播放器配置类型
export interface PlayerConfig {
  block_ad_enabled: boolean;
  optimization_enabled: boolean;
}

// 用户偏好配置类型（统一配置，包含原 SiteConfig 字段）
export interface UserPreferences {
  site_name: string;
  announcement: string;
  search_downstream_max_page: number;
  site_interface_cache_time: number;
  disable_yellow_filter: boolean;

  // 豆瓣设置
  douban_data_source: string;
  douban_proxy_url: string;
  douban_image_proxy_type: string;
  douban_image_proxy_url: string;

  // 用户偏好设置
  enable_optimization: boolean;
  fluid_search: boolean;
  player_buffer_mode: string;
  has_seen_announcement: string;
}

// 播放器初始化状态类型
export interface PlayerInitialState {
  detail: SearchResult;
  other_sources: SearchResult[];
  play_record: {
    episode_index: number;
    play_time: number;
  } | null;
  is_favorited: boolean;
  skip_config: {
    enable: boolean;
    intro_time: number;
    outro_time: number;
  } | null;
  block_ad_enabled: boolean;
  optimization_enabled: boolean;
}

// 收藏数据结构
export interface Favorite {
  source_name: string;
  total_episodes: number; // 总集数
  title: string;
  year: string;
  cover: string;
  save_time: number; // 记录保存时间（时间戳）
  search_title: string; // 搜索时使用的标题
  origin?: 'vod' | 'live';
}

// 存储接口
export interface IStorage {
  // 播放记录相关
  getPlayRecord(userName: string, key: string): Promise<PlayRecord | null>;
  setPlayRecord(
    userName: string,
    key: string,
    record: PlayRecord,
  ): Promise<void>;
  getAllPlayRecords(userName: string): Promise<{ [key: string]: PlayRecord }>;
  deletePlayRecord(userName: string, key: string): Promise<void>;

  // 收藏相关
  getFavorite(userName: string, key: string): Promise<Favorite | null>;
  setFavorite(userName: string, key: string, favorite: Favorite): Promise<void>;
  getAllFavorites(userName: string): Promise<{ [key: string]: Favorite }>;
  deleteFavorite(userName: string, key: string): Promise<void>;

  // 用户相关
  registerUser(userName: string, password: string): Promise<void>;
  verifyUser(userName: string, password: string): Promise<boolean>;
  // 检查用户是否存在（无需密码）
  checkUserExist(userName: string): Promise<boolean>;
  // 修改用户密码
  changePassword(userName: string, newPassword: string): Promise<void>;
  // 删除用户（包括密码、搜索历史、播放记录、收藏夹）
  deleteUser(userName: string): Promise<void>;

  // 搜索历史相关
  getSearchHistory(userName: string): Promise<string[]>;
  addSearchHistory(userName: string, keyword: string): Promise<void>;
  deleteSearchHistory(userName: string, keyword?: string): Promise<void>;

  // 用户列表
  getAllUsers(): Promise<string[]>;

  // 管理员配置相关
  getAdminConfig(): Promise<AdminConfig | null>;
  setAdminConfig(config: AdminConfig): Promise<void>;

  // 跳过片头片尾配置相关
  getSkipConfig(
    userName: string,
    source: string,
    id: string,
  ): Promise<SkipConfig | null>;
  setSkipConfig(
    userName: string,
    source: string,
    id: string,
    config: SkipConfig,
  ): Promise<void>;
  deleteSkipConfig(userName: string, source: string, id: string): Promise<void>;
  getAllSkipConfigs(userName: string): Promise<{ [key: string]: SkipConfig }>;

  // 数据清理相关
  clearAllData(): Promise<void>;
}

// 搜索结果数据结构
export interface SearchResult {
  id: string;
  title: string;
  poster: string;
  episodes: string[];
  episodes_titles: string[];
  source: string;
  source_name: string;
  class?: string;
  year: string;
  desc?: string;
  type_name?: string;
  douban_id?: number;
}

/** 聚合后的分组*/
export interface AggregatedGroup {
  representative: SearchResult;
  episodes: number;
  source_names: string[];
  douban_id?: number;
}

/** 搜索过滤器*/
export interface SearchFilter {
  source: string;
  title: string;
  year: string;
  year_order: 'none' | 'asc' | 'desc';
}

/** 跳过动作*/
export type SkipAction = 'None' | { SkipIntro: number } | 'SkipOutro';

// 豆瓣数据结构
export interface DoubanItem {
  id: string;
  title: string;
  poster: string;
  rate: string;
  year: string;
}

export interface DoubanResult {
  code: number;
  message: string;
  list: DoubanItem[];
}

// 跳过片头片尾配置数据结构
export interface SkipConfig {
  enable: boolean; // 是否启用跳过片头片尾
  intro_time: number; // 片头时间（秒）
  outro_time: number; // 片尾时间（秒）
}

// 本地定义版本状态枚举，不再从外部导入
export enum UpdateStatus {
  CHECKING = 'Checking',
  HAS_UPDATE = 'HasUpdate',
  NO_UPDATE = 'NoUpdate',
  FETCH_FAILED = 'FetchFailed',
}

// Rust 数据结构类型
export interface RustFavorite {
  key: string;
  title: string;
  source_name: string;
  year: string;
  cover: string;
  episode_index: number;
  total_episodes: number;
  save_time: number;
  search_title: string;
}

export interface RustPlayRecord {
  key: string;
  title: string;
  source_name: string;
  year: string;
  cover: string;
  episode_index: number;
  total_episodes: number;
  play_time: number;
  total_time: number;
  save_time: number;
  search_title: string;
}

export interface BangumiCalendarData {
  weekday: {
    en: string;
  };
  items: {
    id: number;
    name: string;
    name_cn: string;
    rating?: {
      score?: number;
    };
    air_date?: string;
    images?: {
      large?: string;
      common?: string;
      medium?: string;
      small?: string;
      grid?: string;
    };
  }[];
}
// Rust 返回类型定义
export interface SourceTestResult {
  quality: string;
  load_speed: string;
  ping_time: number;
  has_error: boolean;
}

export interface PreferBestSourceResponse {
  best_source: SearchResult;
  test_results: Array<[string, SourceTestResult]>;
}
