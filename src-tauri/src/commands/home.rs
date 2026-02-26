use crate::commands::bangumi::{get_bangumi_calendar_data, BangumiCalendarData, Items};
use crate::commands::douban_client::{
    get_douban_categories, DoubanCategoriesParams, DoubanItem, DoubanResult, Kind,
};
use crate::db::db_client::Db;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomePageData {
    pub hot_movies: Vec<DoubanItem>,
    pub hot_tv_shows: Vec<DoubanItem>,
    pub hot_variety_shows: Vec<DoubanItem>,
    pub today_bangumi: Vec<Items>,
}

#[derive(Debug, Serialize)]
pub struct FavoriteCard {
    pub id: String,
    pub source: String,
    pub title: String,
    pub year: String,
    pub poster: String,
    pub episodes: i32,
    pub source_name: String,
    #[serde(rename = "currentEpisode")]
    pub current_episode: Option<i32>,
    pub search_title: String,
}

#[derive(Debug, Serialize)]
pub struct ContinueWatchingItem {
    pub key: String,
    pub title: String,
    pub source_name: String,
    pub year: String,
    pub cover: String,
    pub episode_index: i32,
    pub total_episodes: i32,
    pub play_time: i32,
    pub total_time: i32,
    pub save_time: i32,
    pub search_title: String,
}

#[derive(Debug, Clone)]
struct FavoriteRecord {
    key: String,
    title: String,
    source_name: String,
    year: String,
    cover: String,
    total_episodes: i32,
    save_time: i32,
    search_title: String,
}

#[derive(Debug, Clone)]
struct PlayRecordMeta {
    key: String,
    episode_index: i32,
}

fn split_storage_key(key: &str) -> (String, String) {
    if let Some((source, id)) = key.split_once('+') {
        (source.to_string(), id.to_string())
    } else {
        (key.to_string(), String::new())
    }
}

fn build_favorite_cards(
    mut favorites: Vec<FavoriteRecord>,
    play_records: Vec<PlayRecordMeta>,
) -> Vec<FavoriteCard> {
    favorites.sort_by(|a, b| b.save_time.cmp(&a.save_time));

    let mut play_map = std::collections::HashMap::new();
    for record in play_records {
        play_map.insert(record.key, record.episode_index);
    }

    favorites
        .into_iter()
        .map(|fav| {
            let (source, id) = split_storage_key(&fav.key);
            FavoriteCard {
                id,
                source,
                title: fav.title,
                year: fav.year,
                poster: fav.cover,
                episodes: fav.total_episodes,
                source_name: fav.source_name,
                current_episode: play_map.get(&fav.key).copied(),
                search_title: fav.search_title,
            }
        })
        .collect()
}

fn sort_continue_watching(mut records: Vec<ContinueWatchingItem>) -> Vec<ContinueWatchingItem> {
    records.sort_by(|a, b| b.save_time.cmp(&a.save_time));
    records
}

fn select_bangumi_for_weekday(
    calendar: &[BangumiCalendarData],
    weekday_en: &str,
) -> Vec<Items> {
    calendar
        .iter()
        .find(|item| {
            item.weekday
                .as_ref()
                .map(|w| w.en.as_str() == weekday_en)
                .unwrap_or(false)
        })
        .and_then(|item| item.items.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|item| item.id != 0)
        .collect()
}

fn current_weekday_en() -> &'static str {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400;
    let index = ((now + 4) % 7) as usize;
    WEEKDAYS[index]
}

fn resolve_weekday_input(selected: Option<&str>, fallback: &str) -> String {
    selected
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .unwrap_or(fallback)
        .to_string()
}

fn extract_list_or_empty(result: DoubanResult) -> Vec<DoubanItem> {
    if result.code == 200 {
        result.list
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub async fn get_home_data(weekday: Option<String>) -> Result<HomePageData, String> {
    let movie_params = DoubanCategoriesParams::new(Kind::Movie, "热门", "全部", None, None);
    let tv_params = DoubanCategoriesParams::new(Kind::Tv, "tv", "tv", None, None);
    let show_params = DoubanCategoriesParams::new(Kind::Tv, "show", "show", None, None);

    let (movie_res, tv_res, show_res, bangumi) = tokio::try_join!(
        get_douban_categories(movie_params),
        get_douban_categories(tv_params),
        get_douban_categories(show_params),
        get_bangumi_calendar_data(),
    )?;

    let weekday = resolve_weekday_input(weekday.as_deref(), current_weekday_en());
    let today_bangumi = select_bangumi_for_weekday(&bangumi, &weekday);

    Ok(HomePageData {
        hot_movies: extract_list_or_empty(movie_res),
        hot_tv_shows: extract_list_or_empty(tv_res),
        hot_variety_shows: extract_list_or_empty(show_res),
        today_bangumi,
    })
}

#[tauri::command]
pub fn get_favorite_cards(db: State<'_, Db>) -> Result<Vec<FavoriteCard>, String> {
    let favorites = db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT key, title, source_name, year, cover, total_episodes, save_time, search_title FROM favorites",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FavoriteRecord {
                key: row.get(0)?,
                title: row.get(1)?,
                source_name: row.get(2)?,
                year: row.get(3)?,
                cover: row.get(4)?,
                total_episodes: row.get(5)?,
                save_time: row.get(6)?,
                search_title: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<FavoriteRecord>, _>>()
    })?;

    let play_records = db.with_conn(|conn| {
        let mut stmt =
            conn.prepare("SELECT key, episode_index FROM play_records")?;
        let rows = stmt.query_map([], |row| {
            Ok(PlayRecordMeta {
                key: row.get(0)?,
                episode_index: row.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<PlayRecordMeta>, _>>()
    })?;

    Ok(build_favorite_cards(favorites, play_records))
}

#[tauri::command]
pub fn get_continue_watching(db: State<'_, Db>) -> Result<Vec<ContinueWatchingItem>, String> {
    let records = db.with_conn(|conn| {
        let mut stmt = conn.prepare(
            "SELECT key, title, source_name, year, cover, episode_index, total_episodes, play_time, total_time, save_time, search_title FROM play_records",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ContinueWatchingItem {
                key: row.get(0)?,
                title: row.get(1)?,
                source_name: row.get(2)?,
                year: row.get(3)?,
                cover: row.get(4)?,
                episode_index: row.get(5)?,
                total_episodes: row.get(6)?,
                play_time: row.get(7)?,
                total_time: row.get(8)?,
                save_time: row.get(9)?,
                search_title: row.get(10)?,
            })
        })?;
        rows.collect::<Result<Vec<ContinueWatchingItem>, _>>()
    })?;

    Ok(sort_continue_watching(records))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::bangumi::Weekday;

    #[test]
    fn build_favorite_cards_sorts_and_maps_episode() {
        let favorites = vec![
            FavoriteRecord {
                key: "s1+id1".to_string(),
                title: "A".to_string(),
                source_name: "S1".to_string(),
                year: "2024".to_string(),
                cover: "c1".to_string(),
                total_episodes: 10,
                save_time: 100,
                search_title: "A".to_string(),
            },
            FavoriteRecord {
                key: "s2+id2".to_string(),
                title: "B".to_string(),
                source_name: "S2".to_string(),
                year: "2023".to_string(),
                cover: "c2".to_string(),
                total_episodes: 1,
                save_time: 200,
                search_title: "B".to_string(),
            },
        ];
        let play_records = vec![PlayRecordMeta {
            key: "s1+id1".to_string(),
            episode_index: 3,
        }];

        let cards = build_favorite_cards(favorites, play_records);
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].source, "s2");
        assert_eq!(cards[0].id, "id2");
        assert_eq!(cards[1].current_episode, Some(3));
    }

    #[test]
    fn sort_continue_watching_orders_desc() {
        let records = vec![
            ContinueWatchingItem {
                key: "k1".to_string(),
                title: "A".to_string(),
                source_name: "S".to_string(),
                year: "2020".to_string(),
                cover: "c1".to_string(),
                episode_index: 1,
                total_episodes: 10,
                play_time: 10,
                total_time: 100,
                save_time: 1,
                search_title: "A".to_string(),
            },
            ContinueWatchingItem {
                key: "k2".to_string(),
                title: "B".to_string(),
                source_name: "S".to_string(),
                year: "2021".to_string(),
                cover: "c2".to_string(),
                episode_index: 2,
                total_episodes: 2,
                play_time: 20,
                total_time: 100,
                save_time: 5,
                search_title: "B".to_string(),
            },
        ];

        let sorted = sort_continue_watching(records);
        assert_eq!(sorted[0].key, "k2");
        assert_eq!(sorted[1].key, "k1");
    }

    #[test]
    fn select_bangumi_for_weekday_returns_items() {
        let calendar = vec![
            BangumiCalendarData {
                weekday: Some(Weekday {
                    en: "Mon".to_string(),
                }),
                items: Some(vec![Items {
                    id: 1,
                    name: "A".to_string(),
                    name_cn: "".to_string(),
                    rating: None,
                    air_date: None,
                    images: None,
                }]),
            },
            BangumiCalendarData {
                weekday: Some(Weekday {
                    en: "Tue".to_string(),
                }),
                items: Some(vec![
                    Items {
                        id: 0,
                        name: "Invalid".to_string(),
                        name_cn: "".to_string(),
                        rating: None,
                        air_date: None,
                        images: None,
                    },
                    Items {
                        id: 2,
                        name: "B".to_string(),
                        name_cn: "".to_string(),
                        rating: None,
                        air_date: None,
                        images: None,
                    },
                ]),
            },
        ];

        let items = select_bangumi_for_weekday(&calendar, "Tue");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 2);
    }

    #[test]
    fn resolve_weekday_input_prefers_trimmed_value() {
        let resolved = resolve_weekday_input(Some("  Mon  "), "Thu");
        assert_eq!(resolved, "Mon");
    }

    #[test]
    fn resolve_weekday_input_uses_fallback_for_empty() {
        let resolved = resolve_weekday_input(Some("   "), "Thu");
        assert_eq!(resolved, "Thu");
    }
}
