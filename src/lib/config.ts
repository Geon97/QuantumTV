import { AdminConfig } from './admin.types';

export interface ApiSite {
  key: string;
  api: string;
  name: string;
  detail?: string;
  is_adult?: boolean;
}

export interface LiveCfg {
  name: string;
  url: string;
  ua?: string;
  epg?: string;
}

export const API_CONFIG = {
  search: {
    path: '?ac=videolist&wd=',
    pagePath: '?ac=videolist&wd={query}&pg={page}',
    headers: {
      'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
      Accept: 'application/json',
    },
  },
  detail: {
    path: '?ac=videolist&ids=',
    headers: {
      'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
      Accept: 'application/json',
    },
  },
};

// 内存缓存
let cachedConfig: AdminConfig | null = null;

/**
 * 获取本地模式的默认配置（Tauri 模式使用）
 */
export function getLocalModeConfig(): AdminConfig {
  const adminConfig: AdminConfig = {
    ConfigFile: '',
    ConfigSubscribtion: {
      URL: '',
      AutoUpdate: false,
      LastCheck: '',
    },
    SiteConfig: {
      SiteName: 'QuantumTV',
      Announcement: '本应用仅提供影视信息搜索服务，所有内容均来自第三方网站。',
      SearchDownstreamMaxPage: 5,
      SiteInterfaceCacheTime: 7200,
      DoubanProxyType: 'cmliussss-cdn-tencent',
      DoubanProxy: '',
      DoubanImageProxyType: 'cmliussss-cdn-tencent',
      DoubanImageProxy: '',
      DisableYellowFilter: false,
      FluidSearch: true,
    },
    UserConfig: {
      Users: [
        {
          username: 'admin',
          role: 'owner',
          banned: false,
        },
      ],
    },
    SourceConfig: [],
    CustomCategories: []
  };
  return adminConfig;
}

/**
 * 获取配置（Tauri 模式）
 */
export function getConfig(): AdminConfig {
  if (cachedConfig) {
    return cachedConfig;
  }
  cachedConfig = getLocalModeConfig();
  return cachedConfig;
}


/**
 * 获取可用的 API 站点
 */
export function getAvailableApiSites(): ApiSite[] {
  const config = getConfig();
  return config.SourceConfig.filter((s) => !s.disabled);
}

