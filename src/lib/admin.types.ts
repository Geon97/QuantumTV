export interface AdminConfig {
  ConfigSubscribtion: {
    URL: string;
    AutoUpdate: boolean;
    LastCheck: string;
  };
  ConfigFile: string;
  UserPreferences: {
    // 应用基础设置
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
  };
  UserConfig: {
    Users: {
      username: string;
      role: 'user' | 'admin' | 'owner';
      banned?: boolean;
      enabledApis?: string[]; // 优先级高于tags限制
      tags?: string[]; // 多 tags 取并集限制
    }[];
    Tags?: {
      name: string;
      enabledApis: string[];
    }[];
  };
  SourceConfig: {
    key: string;
    name: string;
    api: string;
    detail?: string;
    from: 'config' | 'custom';
    disabled?: boolean;
    is_adult?: boolean; // 标记是否为成人资源
  }[];
  CustomCategories: {
    name?: string;
    type: 'movie' | 'tv';
    query: string;
    from: 'config' | 'custom';
    disabled?: boolean;
  }[];
}
