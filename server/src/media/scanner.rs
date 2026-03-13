use crate::db::Pool;
use crate::models::{Library, LibraryType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct MediaScanner {
    db: Pool,
}

pub struct ScanResult {
    pub items_found: u32,
    pub items_added: u32,
    pub items_updated: u32,
    pub items_removed: u32,
}

/// Minimal row for checking existing files in DB
#[derive(sqlx::FromRow)]
struct ExistingItem {
    id: Uuid,
    file_path: Option<String>,
    size_bytes: Option<i64>,
    file_mtime: Option<DateTime<Utc>>,
}

impl MediaScanner {
    pub fn new(db: Pool) -> Self {
        Self { db }
    }

    pub async fn scan_library(&self, library: &Library) -> Result<ScanResult> {
        tracing::info!(library_id = %library.id, name = %library.name, "starting library scan");

        // Create a library_scans record
        let scan_id: (Uuid,) = sqlx::query_as(
            "INSERT INTO library_scans (library_id, status, started_at)
             VALUES ($1, 'running'::scan_status, NOW())
             RETURNING id",
        )
        .bind(library.id)
        .fetch_one(&self.db)
        .await?;
        let scan_id = scan_id.0;

        let result = self.do_scan(library).await;

        // Update the scan record with results
        match &result {
            Ok(r) => {
                sqlx::query(
                    "UPDATE library_scans
                     SET status = 'completed'::scan_status, items_found = $1, items_added = $2,
                         items_updated = $3, items_removed = $4, completed_at = NOW()
                     WHERE id = $5",
                )
                .bind(r.items_found as i32)
                .bind(r.items_added as i32)
                .bind(r.items_updated as i32)
                .bind(r.items_removed as i32)
                .bind(scan_id)
                .execute(&self.db)
                .await?;
            }
            Err(e) => {
                sqlx::query(
                    "UPDATE library_scans
                     SET status = 'failed'::scan_status, error_message = $1, completed_at = NOW()
                     WHERE id = $2",
                )
                .bind(e.to_string())
                .bind(scan_id)
                .execute(&self.db)
                .await?;
            }
        }

        result
    }

    async fn do_scan(&self, library: &Library) -> Result<ScanResult> {
        let extensions = extensions_for_type(&library.library_type);
        let mut items_found: u32 = 0;
        let mut items_added: u32 = 0;
        let mut items_updated: u32 = 0;

        // Collect all discovered file paths so we can detect removals
        let mut discovered_paths: HashSet<String> = HashSet::new();

        // Load existing items for this library (that have file_path)
        let existing: Vec<ExistingItem> = sqlx::query_as(
            "SELECT id, file_path, size_bytes, file_mtime
             FROM media_items
             WHERE library_id = $1 AND file_path IS NOT NULL",
        )
        .bind(library.id)
        .fetch_all(&self.db)
        .await?;

        let existing_by_path: std::collections::HashMap<String, ExistingItem> = existing
            .into_iter()
            .filter_map(|item| item.file_path.clone().map(|p| (p, item)))
            .collect();

        for lib_path in &library.paths {
            let root = Path::new(lib_path);
            if !root.exists() {
                tracing::warn!(path = %lib_path, "library path does not exist, skipping");
                continue;
            }

            let mut files = Vec::new();
            walk_dir_recursive(root, &extensions, &mut files).await?;

            for file_path in files {
                items_found += 1;
                let path_str = file_path.to_string_lossy().to_string();
                // Normalize path separators to forward slash for consistency
                let path_str = path_str.replace('\\', "/");
                discovered_paths.insert(path_str.clone());

                let fs_meta = tokio::fs::metadata(&file_path).await?;
                let size_bytes = fs_meta.len() as i64;
                let mtime: DateTime<Utc> = fs_meta.modified()?.into();

                if let Some(existing_item) = existing_by_path.get(&path_str) {
                    // File already indexed — check if changed
                    let size_match = existing_item.size_bytes == Some(size_bytes);
                    let mtime_match = existing_item
                        .file_mtime
                        .map(|t| (t - mtime).num_seconds().abs() < 2)
                        .unwrap_or(false);

                    if size_match && mtime_match {
                        // Unchanged, skip
                        continue;
                    }

                    // File changed — update size/mtime
                    sqlx::query(
                        "UPDATE media_items
                         SET size_bytes = $1, file_mtime = $2, last_scanned_at = NOW(), updated_at = NOW()
                         WHERE id = $3",
                    )
                    .bind(size_bytes)
                    .bind(mtime)
                    .bind(existing_item.id)
                    .execute(&self.db)
                    .await?;
                    items_updated += 1;
                } else {
                    // New file — parse and insert
                    match library.library_type {
                        LibraryType::Movies => {
                            self.index_movie(library.id, &file_path, &path_str, size_bytes, mtime)
                                .await?;
                        }
                        LibraryType::Shows => {
                            self.index_episode(library.id, &file_path, &path_str, size_bytes, mtime)
                                .await?;
                        }
                        LibraryType::Music => {
                            self.index_track(library.id, &file_path, &path_str, size_bytes, mtime)
                                .await?;
                        }
                        LibraryType::Books => {
                            self.index_book(library.id, &file_path, &path_str, size_bytes, mtime)
                                .await?;
                        }
                        LibraryType::Photos => {
                            self.index_photo(library.id, &file_path, &path_str, size_bytes, mtime)
                                .await?;
                        }
                    }
                    items_added += 1;
                }
            }
        }

        // Detect removed files: items in DB that weren't found on disk
        let mut items_removed: u32 = 0;
        for (path, item) in &existing_by_path {
            if !discovered_paths.contains(path) {
                sqlx::query("DELETE FROM media_items WHERE id = $1")
                    .bind(item.id)
                    .execute(&self.db)
                    .await?;
                items_removed += 1;
            }
        }

        tracing::info!(
            items_found, items_added, items_updated, items_removed,
            "scan complete"
        );

        Ok(ScanResult {
            items_found,
            items_added,
            items_updated,
            items_removed,
        })
    }

    // ── Movie indexing ───────────────────────────────────────────────────────

    async fn index_movie(
        &self,
        library_id: Uuid,
        file_path: &Path,
        path_str: &str,
        size_bytes: i64,
        mtime: DateTime<Utc>,
    ) -> Result<()> {
        let (title, year) = parse_title_year(file_path);
        let sort_title = make_sort_title(&title);
        let ext = file_extension(file_path);

        sqlx::query(
            "INSERT INTO media_items (library_id, item_type, title, sort_title, year, file_path, container, size_bytes, file_mtime, last_scanned_at, date_added)
             VALUES ($1, 'movie'::media_item_type, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())",
        )
        .bind(library_id)
        .bind(&title)
        .bind(&sort_title)
        .bind(year)
        .bind(path_str)
        .bind(ext.as_deref())
        .bind(size_bytes)
        .bind(mtime)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ── TV show indexing ─────────────────────────────────────────────────────

    async fn index_episode(
        &self,
        library_id: Uuid,
        file_path: &Path,
        path_str: &str,
        size_bytes: i64,
        mtime: DateTime<Utc>,
    ) -> Result<()> {
        // Parse show structure from path
        // Expected: .../Show Name/Season 01/S01E01 - Episode Title.mkv
        // Or:       .../Show Name/S01E01 - Episode Title.mkv
        let components: Vec<&str> = path_str.split('/').collect();

        let (show_name, season_num, episode_num, episode_title) =
            parse_show_structure(&components, file_path);

        let show_sort_title = make_sort_title(&show_name);

        // Find or create the Series item
        let series_id = self
            .find_or_create_series(library_id, &show_name, &show_sort_title)
            .await?;

        // Find or create the Season item
        let season_id = if let Some(sn) = season_num {
            Some(
                self.find_or_create_season(library_id, series_id, sn)
                    .await?,
            )
        } else {
            None
        };

        let ext = file_extension(file_path);

        sqlx::query(
            "INSERT INTO media_items (library_id, parent_id, item_type, title, sort_title, file_path, container, size_bytes, file_mtime, season_number, episode_number, last_scanned_at, date_added)
             VALUES ($1, $2, 'episode'::media_item_type, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())",
        )
        .bind(library_id)
        .bind(season_id.or(Some(series_id)))
        .bind(&episode_title)
        .bind(&make_sort_title(&episode_title))
        .bind(path_str)
        .bind(ext.as_deref())
        .bind(size_bytes)
        .bind(mtime)
        .bind(season_num)
        .bind(episode_num)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn find_or_create_series(
        &self,
        library_id: Uuid,
        name: &str,
        sort_title: &str,
    ) -> Result<Uuid> {
        // Look for existing series with this title in this library
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM media_items
             WHERE library_id = $1 AND item_type = 'series'::media_item_type AND title = $2
             LIMIT 1",
        )
        .bind(library_id)
        .bind(name)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = existing {
            return Ok(row.0);
        }

        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO media_items (library_id, item_type, title, sort_title, date_added)
             VALUES ($1, 'series'::media_item_type, $2, $3, NOW())
             RETURNING id",
        )
        .bind(library_id)
        .bind(name)
        .bind(sort_title)
        .fetch_one(&self.db)
        .await?;

        Ok(row.0)
    }

    async fn find_or_create_season(
        &self,
        library_id: Uuid,
        series_id: Uuid,
        season_number: i32,
    ) -> Result<Uuid> {
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM media_items
             WHERE library_id = $1 AND parent_id = $2 AND item_type = 'season'::media_item_type AND season_number = $3
             LIMIT 1",
        )
        .bind(library_id)
        .bind(series_id)
        .bind(season_number)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = existing {
            return Ok(row.0);
        }

        let title = format!("Season {season_number:02}");
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO media_items (library_id, parent_id, item_type, title, sort_title, season_number, date_added)
             VALUES ($1, $2, 'season'::media_item_type, $3, $4, $5, NOW())
             RETURNING id",
        )
        .bind(library_id)
        .bind(series_id)
        .bind(&title)
        .bind(&title)
        .bind(season_number)
        .fetch_one(&self.db)
        .await?;

        Ok(row.0)
    }

    // ── Music indexing ───────────────────────────────────────────────────────

    async fn index_track(
        &self,
        library_id: Uuid,
        file_path: &Path,
        path_str: &str,
        size_bytes: i64,
        mtime: DateTime<Utc>,
    ) -> Result<()> {
        let (title, _year) = parse_title_year(file_path);
        let sort_title = make_sort_title(&title);
        let ext = file_extension(file_path);

        // Try to parse track number from filename: "01 - Song Title.flac" or "01. Song Title.flac"
        let track_number = parse_track_number(file_path);

        // Try to get album from parent directory
        let album_name = file_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string());

        let parent_id = if let Some(ref album) = album_name {
            let album_sort = make_sort_title(album);
            Some(self.find_or_create_album(library_id, album, &album_sort).await?)
        } else {
            None
        };

        sqlx::query(
            "INSERT INTO media_items (library_id, parent_id, item_type, title, sort_title, file_path, container, size_bytes, file_mtime, track_number, last_scanned_at, date_added)
             VALUES ($1, $2, 'track'::media_item_type, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())",
        )
        .bind(library_id)
        .bind(parent_id)
        .bind(&title)
        .bind(&sort_title)
        .bind(path_str)
        .bind(ext.as_deref())
        .bind(size_bytes)
        .bind(mtime)
        .bind(track_number)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn find_or_create_album(
        &self,
        library_id: Uuid,
        name: &str,
        sort_title: &str,
    ) -> Result<Uuid> {
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM media_items
             WHERE library_id = $1 AND item_type = 'album'::media_item_type AND title = $2
             LIMIT 1",
        )
        .bind(library_id)
        .bind(name)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = existing {
            return Ok(row.0);
        }

        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO media_items (library_id, item_type, title, sort_title, date_added)
             VALUES ($1, 'album'::media_item_type, $2, $3, NOW())
             RETURNING id",
        )
        .bind(library_id)
        .bind(name)
        .bind(sort_title)
        .fetch_one(&self.db)
        .await?;

        Ok(row.0)
    }

    // ── Book indexing ────────────────────────────────────────────────────────

    async fn index_book(
        &self,
        library_id: Uuid,
        file_path: &Path,
        path_str: &str,
        size_bytes: i64,
        mtime: DateTime<Utc>,
    ) -> Result<()> {
        let (title, year) = parse_title_year(file_path);
        let sort_title = make_sort_title(&title);
        let ext = file_extension(file_path);

        sqlx::query(
            "INSERT INTO media_items (library_id, item_type, title, sort_title, year, file_path, container, size_bytes, file_mtime, last_scanned_at, date_added)
             VALUES ($1, 'book'::media_item_type, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())",
        )
        .bind(library_id)
        .bind(&title)
        .bind(&sort_title)
        .bind(year)
        .bind(path_str)
        .bind(ext.as_deref())
        .bind(size_bytes)
        .bind(mtime)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ── Photo indexing ───────────────────────────────────────────────────────

    async fn index_photo(
        &self,
        library_id: Uuid,
        file_path: &Path,
        path_str: &str,
        size_bytes: i64,
        mtime: DateTime<Utc>,
    ) -> Result<()> {
        let (title, _year) = parse_title_year(file_path);
        let sort_title = make_sort_title(&title);

        sqlx::query(
            "INSERT INTO media_items (library_id, item_type, title, sort_title, file_path, size_bytes, file_mtime, last_scanned_at, date_added)
             VALUES ($1, 'photo'::media_item_type, $2, $3, $4, $5, $6, NOW(), NOW())",
        )
        .bind(library_id)
        .bind(&title)
        .bind(&sort_title)
        .bind(path_str)
        .bind(size_bytes)
        .bind(mtime)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

fn extensions_for_type(library_type: &LibraryType) -> HashSet<String> {
    let exts: &[&str] = match library_type {
        LibraryType::Movies | LibraryType::Shows => {
            &["mkv", "mp4", "avi", "m4v", "webm", "ts", "mov"]
        }
        LibraryType::Music => &["flac", "mp3", "m4a", "ogg", "opus", "wav"],
        LibraryType::Books => &["epub", "pdf", "cbz", "cbr"],
        LibraryType::Photos => &["jpg", "jpeg", "png", "webp", "heic"],
    };
    exts.iter().map(|e| e.to_string()).collect()
}

/// Recursively walk a directory, collecting files matching the given extensions.
async fn walk_dir_recursive(
    dir: &Path,
    extensions: &HashSet<String>,
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    let mut read_dir = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let path = entry.path();
        let ft = entry.file_type().await?;

        if ft.is_dir() {
            // Skip hidden directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
            Box::pin(walk_dir_recursive(&path, extensions, out)).await?;
        } else if ft.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext.to_lowercase()) {
                    out.push(path);
                }
            }
        }
    }
    Ok(())
}

/// Parse a title and optional year from a filename.
/// "Movie Title (2024).mkv" -> ("Movie Title", Some(2024))
/// "just a file.mp4" -> ("just a file", None)
fn parse_title_year(path: &Path) -> (String, Option<i32>) {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // Try to extract year from parentheses: "Title (2024)"
    if let Some(paren_start) = stem.rfind('(') {
        if let Some(paren_end) = stem[paren_start..].find(')') {
            let inside = &stem[paren_start + 1..paren_start + paren_end];
            if let Ok(year) = inside.trim().parse::<i32>() {
                if (1800..=2100).contains(&year) {
                    let title = stem[..paren_start].trim().to_string();
                    if !title.is_empty() {
                        return (title, Some(year));
                    }
                }
            }
        }
    }

    (stem, None)
}

/// Generate a sort title: lowercase, strip leading "the ", "a ", "an ".
fn make_sort_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let stripped = if lower.starts_with("the ") {
        &lower[4..]
    } else if lower.starts_with("a ") {
        &lower[2..]
    } else if lower.starts_with("an ") {
        &lower[3..]
    } else {
        &lower
    };
    stripped.trim().to_string()
}

/// Extract the file extension (lowercase) without the dot.
fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

/// Parse TV show structure from path components.
/// Returns (show_name, season_number, episode_number, episode_title).
fn parse_show_structure(
    components: &[&str],
    file_path: &Path,
) -> (String, Option<i32>, Option<i32>, String) {
    let filename_stem = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // Parse S01E01 or 1x01 pattern from filename
    let (season_from_file, episode_num, ep_title) = parse_episode_filename(&filename_stem);

    // Try to find season folder and show name from path components
    // Typical structures:
    //   .../Show Name/Season 01/S01E01.mkv  (3+ components from the media file)
    //   .../Show Name/S01E01.mkv            (2+ components)
    let len = components.len();

    let mut show_name = String::new();
    let mut season_from_folder: Option<i32> = None;

    if len >= 3 {
        // Check if second-to-last dir is a season folder
        let potential_season = components[len - 2];
        if let Some(sn) = parse_season_folder(potential_season) {
            season_from_folder = Some(sn);
            // Show name is the component before the season folder
            if len >= 3 {
                show_name = components[len - 3].to_string();
            }
        } else {
            // No season folder — the parent dir is the show name
            show_name = components[len - 2].to_string();
        }
    } else if len >= 2 {
        show_name = components[len - 2].to_string();
    }

    if show_name.is_empty() {
        show_name = "Unknown Show".to_string();
    }

    // Prefer season from folder, fall back to season from filename
    let season_num = season_from_folder.or(season_from_file);

    let episode_title = if ep_title.is_empty() {
        filename_stem.clone()
    } else {
        ep_title
    };

    (show_name, season_num, episode_num, episode_title)
}

/// Parse a season folder name.
/// "Season 01", "Season 1", "S01", "s01" -> Some(1)
fn parse_season_folder(name: &str) -> Option<i32> {
    let lower = name.to_lowercase();

    if let Some(rest) = lower.strip_prefix("season ") {
        return rest.trim().parse::<i32>().ok();
    }

    if lower.len() >= 2 && lower.starts_with('s') {
        let rest = &lower[1..];
        if rest.chars().all(|c| c.is_ascii_digit()) {
            return rest.parse::<i32>().ok();
        }
    }

    None
}

/// Parse episode info from a filename stem.
/// "S01E01 - Episode Title" -> (Some(1), Some(1), "Episode Title")
/// "1x01 - Title" -> (Some(1), Some(1), "Title")
/// "S02E13" -> (Some(2), Some(13), "")
fn parse_episode_filename(stem: &str) -> (Option<i32>, Option<i32>, String) {
    let upper = stem.to_uppercase();

    // Try S01E01 pattern
    if let Some(s_pos) = upper.find('S') {
        let after_s = &upper[s_pos + 1..];
        if let Some(e_pos) = after_s.find('E') {
            let season_str = &after_s[..e_pos];
            let rest = &after_s[e_pos + 1..];

            // Extract episode number (digits only)
            let ep_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();

            if let (Ok(season), Ok(episode)) = (season_str.parse::<i32>(), ep_str.parse::<i32>()) {
                // Extract title after the episode number
                let original_offset = s_pos + 1 + e_pos + 1 + ep_str.len();
                let title = if original_offset < stem.len() {
                    stem[original_offset..]
                        .trim_start_matches(|c: char| c == '-' || c == ' ' || c == '.')
                        .to_string()
                } else {
                    String::new()
                };

                return (Some(season), Some(episode), title);
            }
        }
    }

    // Try 1x01 pattern
    if let Some(x_pos) = upper.find('X') {
        if x_pos > 0 {
            let before_x: String = upper[..x_pos]
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let after_x: String = upper[x_pos + 1..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();

            if let (Ok(season), Ok(episode)) = (before_x.parse::<i32>(), after_x.parse::<i32>()) {
                let digits_end = x_pos + 1 + after_x.len();
                let title = if digits_end < stem.len() {
                    stem[digits_end..]
                        .trim_start_matches(|c: char| c == '-' || c == ' ' || c == '.')
                        .to_string()
                } else {
                    String::new()
                };

                return (Some(season), Some(episode), title);
            }
        }
    }

    // No pattern found
    (None, None, stem.to_string())
}

/// Parse track number from a music filename.
/// "01 - Song Title.flac" -> Some(1)
/// "01. Song Title.flac" -> Some(1)
/// "Song Title.flac" -> None
fn parse_track_number(path: &Path) -> Option<i32> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())?;

    // Check if filename starts with digits followed by separator
    let digit_prefix: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digit_prefix.is_empty() {
        return None;
    }

    // Verify there's a separator after the digits (space, dash, dot)
    let rest = &stem[digit_prefix.len()..];
    if rest.starts_with(' ') || rest.starts_with('-') || rest.starts_with('.') || rest.starts_with(" -") {
        digit_prefix.parse::<i32>().ok()
    } else {
        None
    }
}
