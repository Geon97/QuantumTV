'use client';

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

/**
 * 将 HTTP 图片链接转换为 HTTPS，避免 Mixed Content 警告
 */
function normalizeImageUrl(url?: string): string | undefined {
  if (!url) return undefined;
  return url.replace(/^http:\/\//i, 'https://');
}

/**
 * 规范化 Bangumi 数据，确保图片链接使用 HTTPS
 */
function normalizeBangumiData(
  data: BangumiCalendarData[],
): BangumiCalendarData[] {
  return data.map((day) => ({
    ...day,
    items: day.items.map((item) => ({
      ...item,
      images: item.images
        ? {
            large: normalizeImageUrl(item.images.large),
            common: normalizeImageUrl(item.images.common),
            medium: normalizeImageUrl(item.images.medium),
            small: normalizeImageUrl(item.images.small),
            grid: normalizeImageUrl(item.images.grid),
          }
        : undefined,
    })),
  }));
}

export async function GetBangumiCalendarData(): Promise<BangumiCalendarData[]> {
  try {
    const response = await fetch('https://api.bgm.tv/calendar');
    if (!response.ok) {
      return [];
    }
    const data = await response.json();
    // 确保返回的数据是数组格式，并规范化图片链接
    const calendarData = Array.isArray(data) ? data : [];
    return normalizeBangumiData(calendarData);
  } catch {
    return [];
  }
}
