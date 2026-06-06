use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use chrono::prelude::*;
use std::time::Duration;
use tauri::Manager;
use reqwest::header;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Global memory cache for discovered skills
static SKILLS_CACHE: std::sync::OnceLock<std::sync::Mutex<Option<Vec<SkillMetadata>>>> = std::sync::OnceLock::new();

fn get_skills_cache() -> &'static std::sync::Mutex<Option<Vec<SkillMetadata>>> {
    SKILLS_CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

fn invalidate_skills_cache() {
    if let Some(cache) = SKILLS_CACHE.get() {
        if let Ok(mut lock) = cache.lock() {
            *lock = None;
            println!("DEBUG: [invalidate_skills_cache] Cache invalidated.");
        }
    }
}

// ==========================================
// 1. Data Structures & Configurations
// ==========================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebDavConfig {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub auto_backup_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub url: String,
    pub is_official: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkillStatus {
    pub repo_id: String,
    /// Distribution scope: "global" (user-wide) or "project" (current project tree)
    #[serde(default = "default_scope")]
    pub scope: String,
    pub enable_agy: bool,
    #[serde(default)]
    pub enable_agy2: bool,
    pub enable_reasonix: bool,
    pub auto_update: bool,
}

fn default_scope() -> String {
    "global".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub theme: String,
    pub webdav: WebDavConfig,
    pub repositories: Vec<Repository>,
    pub skills_status: HashMap<String, SkillStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub repo_id: String,
    pub relative_path: String,
    pub is_installed: bool,
    pub is_downloaded: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            webdav: WebDavConfig {
                url: "https://dav.jianguoyun.com/dav/".to_string(),
                user: "your_email@qq.com".to_string(),
                pass: "your_app_password".to_string(),
                auto_backup_enabled: true,
            },
            repositories: vec![
                Repository {
                    id: "official-public-repo".to_string(),
                    name: "公共筛选库".to_string(),
                    url: "https://github.com/ComposioHQ/awesome-claude-skills.git".to_string(),
                    is_official: true,
                }
            ],
            skills_status: HashMap::new(),
        }
    }
}

// ==========================================
// Helper functions
// ==========================================

static APP_DATA_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

fn get_app_data_path() -> PathBuf {
    APP_DATA_PATH.get_or_init(|| {
        let p = home::home_dir()
            .map(|p| p.join("AppData").join("Roaming").join("new-SkillControl"))
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming\\new-SkillControl"));
        println!("DEBUG: [get_app_data_path] initialized -> {:?}", p);
        p
    }).clone()
}

fn get_project_root() -> PathBuf {
    PathBuf::from("D:\\123123123123\\new-SkillControl")
}

fn get_my_brain_path() -> PathBuf {
    let path = get_app_data_path().join("my-brain");
    println!("DEBUG: [get_my_brain_path] {:?}", path);
    if !path.exists() {
        println!("DEBUG: [get_my_brain_path] creating directory...");
        let _ = fs::create_dir_all(&path);
    }
    path
}

fn get_config_path() -> PathBuf {
    let p = get_my_brain_path().join("config.json");
    println!("DEBUG: [get_config_path] {:?}", p);
    p
}

fn get_repos_cache_path() -> PathBuf {
    let path = get_my_brain_path().join("repos");
    println!("DEBUG: [get_repos_cache_path] {:?}", path);
    if !path.exists() {
        println!("DEBUG: [get_repos_cache_path] creating directory...");
        let _ = fs::create_dir_all(&path);
    }
    path
}

fn get_staging_path() -> PathBuf {
    let path = get_my_brain_path().join("staging");
    println!("DEBUG: [get_staging_path] {:?}", path);
    if !path.exists() {
        println!("DEBUG: [get_staging_path] creating directory...");
        let _ = fs::create_dir_all(&path);
    }
    path
}

/// Create a reqwest::Client with short timeouts for metadata operations (PROPFIND, MKCOL).
/// Injects standard browser User-Agent to bypass 403 on restrictive WebDAV servers.
fn create_webdav_client() -> reqwest::Client {
    let mut default_headers = header::HeaderMap::new();
    default_headers.insert(header::USER_AGENT,
        header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    default_headers.insert(header::ACCEPT,
        header::HeaderValue::from_static("*/*"));
    default_headers.insert(header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"));

    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(default_headers)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Create a reqwest::Client with longer timeout for file upload/download.
/// ZIP downloads can exceed 15s on slow connections.
/// Injects browser User-Agent + full header set for 403 bypass.
fn create_webdav_download_client() -> reqwest::Client {
    let mut default_headers = header::HeaderMap::new();
    default_headers.insert(header::USER_AGENT,
        header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
    default_headers.insert(header::ACCEPT,
        header::HeaderValue::from_static("*/*"));
    default_headers.insert(header::ACCEPT_LANGUAGE,
        header::HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"));
    default_headers.insert(header::ACCEPT_ENCODING,
        header::HeaderValue::from_static("gzip, deflate"));
    default_headers.insert(header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-cache"));

    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .default_headers(default_headers)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Generates Authorization header. If user or password is empty, returns None for anonymous access.
fn get_auth_header(config: &AppConfig) -> Option<String> {
    if config.webdav.user.trim().is_empty() || config.webdav.pass.trim().is_empty() {
        None
    } else {
        Some(format!("Basic {}", base64_encode(&format!("{}:{}", config.webdav.user.trim(), config.webdav.pass.trim()))))
    }
}

/// Helper function to perform a WebDAV request with retry policy (up to 3 times, 500ms delay)
/// and dual loopback support (swapping 127.0.0.1 <-> localhost on connection failure).
async fn send_webdav_request(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    auth_header: &Option<String>,
    depth: Option<&str>,
    translate: Option<&str>,
    content_type: Option<&str>,
    body: Option<Vec<u8>>,
) -> Result<reqwest::Response, String> {
    let mut current_url = url.to_string();
    let mut attempt = 0;
    let mut redirect_count = 0;
    const MAX_RETRIES: usize = 3;
    let original_parsed = reqwest::Url::parse(url).ok();

    loop {
        let mut req = client.request(method.clone(), &current_url);
        if let Some(ref auth) = auth_header {
            req = req.header("Authorization", auth);
        }
        if let Some(d) = depth {
            req = req.header("Depth", d);
        }
        if let Some(t) = translate {
            req = req.header("Translate", t);
        }
        if let Some(ct) = content_type {
            req = req.header("Content-Type", ct);
        }
        if let Some(ref b) = body {
            req = req.body(b.clone());
        }

        println!("DEBUG: [send_webdav_request] Attempt {} to {}", attempt + 1, current_url);
        match req.send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_redirection() {
                    redirect_count += 1;
                    if redirect_count > 5 {
                        return Err("重定向次数过多，已终止".to_string());
                    }
                    if let Some(loc_val) = resp.headers().get(reqwest::header::LOCATION) {
                        if let Ok(loc_str) = loc_val.to_str() {
                            let base_url = reqwest::Url::parse(&current_url).map_err(|e| e.to_string())?;
                            let mut redirect_url = base_url.join(loc_str).map_err(|e| e.to_string())?;
                            
                            // 校验并替换 Host
                            if let Some(ref orig) = original_parsed {
                                let orig_host = orig.host_str().unwrap_or("");
                                if orig_host == "127.0.0.1" || orig_host == "localhost" {
                                    let redir_host = redirect_url.host_str().unwrap_or("");
                                    if redir_host != "127.0.0.1" && redir_host != "localhost" {
                                        // 仅当重定向目标是局域网私有 IP 时才进行本地环回纠错，以防干扰后续重定向到公网 CDN 下载直链
                                        let is_private_ip = if let Ok(ip) = redir_host.parse::<std::net::IpAddr>() {
                                            match ip {
                                                std::net::IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_link_local(),
                                                std::net::IpAddr::V6(_) => false,
                                            }
                                        } else {
                                            false
                                        };

                                        if is_private_ip {
                                            println!("DEBUG: [send_webdav_request] Correcting redirect host from '{}' to '{}'", redir_host, orig_host);
                                            let _ = redirect_url.set_host(Some(orig_host));
                                        }
                                    }
                                }
                            }
                            
                            current_url = redirect_url.to_string();
                            println!("DEBUG: [send_webdav_request] Following redirect to: {}", current_url);
                            continue;
                        }
                    }
                }
                return Ok(resp);
            }
            Err(e) => {
                attempt += 1;
                if attempt >= MAX_RETRIES {
                    return Err(format!("连接 WebDAV 失败（已尝试 {} 次）: {}", MAX_RETRIES, e));
                }
                
                // Swap local address formats
                if current_url.contains("127.0.0.1") {
                    current_url = current_url.replace("127.0.0.1", "localhost");
                } else if current_url.contains("localhost") {
                    current_url = current_url.replace("localhost", "127.0.0.1");
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

/// Robust namespace-insensitive XML extractor to parse href values from WebDAV PROPFIND responses.
fn extract_hrefs_from_xml(xml: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let lower_xml = xml.to_lowercase();
    let mut pos = 0;
    
    while let Some(href_tag_end) = lower_xml[pos..].find("href>") {
        let tag_start_candidate = pos + href_tag_end;
        if let Some(open_bracket) = lower_xml[..tag_start_candidate].rfind('<') {
            if !lower_xml[open_bracket..tag_start_candidate].contains('>') {
                let content_start = tag_start_candidate + "href>".len();
                if let Some(close_tag_start) = lower_xml[content_start..].find("href>") {
                    let end_pos = content_start + close_tag_start;
                    if let Some(close_bracket) = lower_xml[..end_pos].rfind("</") {
                        if close_bracket >= content_start {
                            let href_val = &xml[content_start..close_bracket];
                            hrefs.push(href_val.trim().to_string());
                        }
                    }
                }
            }
        }
        pos += href_tag_end + 5;
    }
    hrefs
}

// Base64 helper for WebDAV basic authorization
fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] } else { 0 };
        
        let enc1 = b0 >> 2;
        let enc2 = ((b0 & 3) << 4) | (b1 >> 4);
        let enc3 = ((b1 & 15) << 2) | (b2 >> 6);
        let enc4 = b2 & 63;
        
        result.push(CHARS[enc1 as usize] as char);
        result.push(CHARS[enc2 as usize] as char);
        if i + 1 < bytes.len() {
            result.push(CHARS[enc3 as usize] as char);
        } else {
            result.push('=');
        }
        if i + 2 < bytes.len() {
            result.push(CHARS[enc4 as usize] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

// ==========================================
// 2. Command Implementations
// ==========================================

async fn get_config_internal() -> Result<AppConfig, String> {
    println!("DEBUG: [get_config_internal] START");
    let path = get_config_path();
    println!("DEBUG: [get_config_internal] path.exists() = {:?}", path.exists());
    if !path.exists() {
        println!("DEBUG: [get_config_internal] writing default config...");
        let default_config = AppConfig::default();
        let data = serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?;
        tokio::fs::write(&path, data).await.map_err(|e| e.to_string())?;
        println!("DEBUG: [get_config_internal] default config written");
        return Ok(default_config);
    }
    println!("DEBUG: [get_config_internal] reading config file...");
    let content = tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    println!("DEBUG: [get_config_internal] OK (theme={}, repos={})", config.theme, config.repositories.len());
    Ok(config)
}

#[tauri::command]
async fn get_config() -> Result<AppConfig, String> {
    get_config_internal().await
}

#[tauri::command]
async fn save_config(config: AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, data).await.map_err(|e| e.to_string())?;

    // Auto WebDAV backup if enabled — FIRE-AND-FORGET via tokio::spawn,
    // NEVER block the IPC response waiting for a network request.
    if config.webdav.auto_backup_enabled 
        && !config.webdav.url.is_empty() 
        && config.webdav.url != "https://dav.jianguoyun.com/dav/" // don't auto-backup with default config placeholders
    {
        let webdav_cfg = config.webdav.clone();
        tokio::spawn(async move {
            // Reconstruct a minimal AppConfig just for backup
            let backup_cfg = AppConfig {
                webdav: webdav_cfg,
                ..AppConfig::default()
            };
            let _ = backup_to_webdav_internal(&backup_cfg).await;
        });
    }

    Ok(())
}

#[tauri::command]
async fn add_repository(name: String, url: String) -> Result<AppConfig, String> {
    let mut config = get_config_internal().await?;
    
    // Generate simple unique ID
    let repo_id = format!("repo-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    let new_repo = Repository {
        id: repo_id.clone(),
        name,
        url,
        is_official: false,
    };
    
    config.repositories.push(new_repo);
    save_config(config.clone()).await?;

    // Trigger initial clone in background
    let clone_url = config.repositories.last().unwrap().url.clone();
    let dest_path = get_repos_cache_path().join(&repo_id);
    tokio::spawn(async move {
        if git_clone_internal(&clone_url, &dest_path).await.is_ok() {
            invalidate_skills_cache();
        }
    });

    invalidate_skills_cache();
    Ok(config)
}

#[tauri::command]
async fn delete_repository(repo_id: String) -> Result<AppConfig, String> {
    let mut config = get_config_internal().await?;
    
    // 1. Remove repo from configuration
    config.repositories.retain(|r| r.id != repo_id);
    
    // 2. Unbind all skills of this repository
    let mut skills_to_remove = Vec::new();
    for (skill_id, status) in config.skills_status.iter() {
        if status.repo_id == repo_id {
            skills_to_remove.push(skill_id.clone());
        }
    }
    for skill_id in skills_to_remove {
        // Physical deletion before unbinding
        let _ = remove_physical_distribution(&skill_id).await;
        config.skills_status.remove(&skill_id);
    }
    
    save_config(config.clone()).await?;

    // 3. Remove git repository cache
    let repo_path = get_repos_cache_path().join(&repo_id);
    if repo_path.exists() {
        let _ = tokio::fs::remove_dir_all(repo_path).await;
    }

    invalidate_skills_cache();
    Ok(config)
}

#[tauri::command]
async fn sync_all_repositories() -> Result<(), String> {
    let config = get_config_internal().await?;
    for repo in &config.repositories {
        let repo_path = get_repos_cache_path().join(&repo.id);
        if repo_path.exists() {
            let _ = git_pull_internal(&repo_path).await;
        } else {
            let _ = git_clone_internal(&repo.url, &repo_path).await;
        }
        let _ = auto_update_and_cleanup_repo(&repo.id).await;
    }
    invalidate_skills_cache();
    Ok(())
}

#[tauri::command]
async fn sync_single_repository(repo_id: String) -> Result<(), String> {
    let config = get_config_internal().await?;
    if let Some(repo) = config.repositories.iter().find(|r| r.id == repo_id) {
        let repo_path = get_repos_cache_path().join(&repo.id);
        if repo_path.exists() {
            let _ = git_pull_internal(&repo_path).await;
        } else {
            let _ = git_clone_internal(&repo.url, &repo_path).await;
        }
        let _ = auto_update_and_cleanup_repo(&repo_id).await;
    }
    invalidate_skills_cache();
    Ok(())
}

async fn auto_update_and_cleanup_repo(repo_id: &str) -> Result<(), String> {
    let mut config = get_config_internal().await?;
    let repo_path = get_repos_cache_path().join(repo_id);
    if !repo_path.exists() {
        return Ok(());
    }

    let repo_id_str = repo_id.to_string();
    let repo_path_clone = repo_path.clone();
    let skills_status_map = config.skills_status.clone();
    let discovered = tokio::task::spawn_blocking(move || {
        let mut skills = Vec::new();
        scan_directory_for_skills(&repo_path_clone, &repo_id_str, &mut skills, &skills_status_map);
        skills
    }).await.map_err(|e| e.to_string())?;

    let discovered_ids: std::collections::HashSet<String> = discovered.iter().map(|s| s.id.clone()).collect();

    // 1. Auto update existing installed skills in staging/distribution if they have auto_update enabled
    for skill in &discovered {
        if skill.is_installed {
            let auto_update = config.skills_status.get(&skill.id)
                .map(|status| status.auto_update)
                .unwrap_or(true);
            
            if auto_update {
                println!("DEBUG: [auto_update] Syncing skill {}...", skill.id);
                let source_path = repo_path.join(&skill.relative_path);
                let dest_path = get_staging_path().join(format!("{}.md", skill.id));
                if source_path.exists() {
                    let _ = tokio::fs::copy(&source_path, &dest_path).await;
                    // Re-distribute physical files
                    if config.skills_status.contains_key(&skill.id) {
                        let _ = sync_physical_distributions_for_skill(&skill.id, &config).await;
                    }
                }
            }
        }
    }

    // 2. Remove skills that are in status but no longer present in this repo (deleted/renamed)
    let mut skills_to_remove = Vec::new();
    for (skill_id, status) in &config.skills_status {
        if status.repo_id == repo_id && !discovered_ids.contains(skill_id) {
            skills_to_remove.push(skill_id.clone());
        }
    }

    if !skills_to_remove.is_empty() {
        println!("DEBUG: [cleanup] Removing orphaned skills: {:?}", skills_to_remove);
        for skill_id in &skills_to_remove {
            let _ = remove_physical_distribution(skill_id).await;
            config.skills_status.remove(skill_id);
        }
        save_config(config).await?;
    }

    Ok(())
}

// ==========================================
// 3. Git Operations Implementations
// ==========================================

async fn git_clone_internal(url: &str, dest: &Path) -> Result<(), String> {
    let mut cmd = TokioCommand::new("git");
    cmd.arg("clone")
       .arg(url)
       .arg(dest)
       .env("GIT_TERMINAL_PROMPT", "0")
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    cmd.as_std_mut().creation_flags(0x08000000);

    let mut child = cmd.spawn().map_err(|e| format!("Failed to start git clone: {}", e))?;
    let status = child.wait().await.map_err(|e| format!("Git clone failed to run: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("Git clone failed (check repository URL or network).".to_string())
    }
}

async fn git_pull_internal(repo_path: &Path) -> Result<(), String> {
    let mut cmd = TokioCommand::new("git");
    cmd.current_dir(repo_path)
       .arg("pull")
       .env("GIT_TERMINAL_PROMPT", "0")
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    cmd.as_std_mut().creation_flags(0x08000000);

    let mut child = cmd.spawn().map_err(|e| format!("Failed to start git pull: {}", e))?;
    let status = child.wait().await.map_err(|e| format!("Git pull failed to run: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("Git pull failed.".to_string())
    }
}

// ==========================================
// 4. Skills Discovery & Frontmatter Parser
// ==========================================

#[tauri::command]
async fn discover_skills(repo_id: String) -> Result<Vec<SkillMetadata>, String> {
    let all_skills = discover_all_skills().await?;
    let filtered = all_skills.into_iter().filter(|s| s.repo_id == repo_id).collect();
    Ok(filtered)
}

#[tauri::command]
async fn discover_all_skills() -> Result<Vec<SkillMetadata>, String> {
    println!("DEBUG: [discover_all_skills] START");
    
    // Check memory cache first
    {
        if let Ok(cache) = get_skills_cache().lock() {
            if let Some(ref cached_skills) = *cache {
                println!("DEBUG: [discover_all_skills] Return from cache ({} skills)", cached_skills.len());
                return Ok(cached_skills.clone());
            }
        }
    }

    let config = get_config_internal().await?;
    let mut repos_to_scan = Vec::new();
    for repo in &config.repositories {
        let repo_path = get_repos_cache_path().join(&repo.id);
        println!("DEBUG: [discover_all_skills] repo={} exists={}", repo.name, repo_path.exists());
        if !repo_path.exists() {
            let clone_url = repo.url.clone();
            let dest_path = repo_path.clone();
            println!("DEBUG: [discover_all_skills] spawning bg clone for {}", repo.name);
            tokio::spawn(async move {
                if git_clone_internal(&clone_url, &dest_path).await.is_ok() {
                    invalidate_skills_cache();
                }
            });
        } else {
            repos_to_scan.push((repo_path, repo.id.clone()));
        }
    }

    println!("DEBUG: [discover_all_skills] scanning {} repos", repos_to_scan.len());
    let skills_status_map = config.skills_status.clone();
    let all_skills = tokio::task::spawn_blocking(move || {
        let mut skills = Vec::new();
        for (repo_path, repo_id) in repos_to_scan {
            scan_directory_for_skills(&repo_path, &repo_id, &mut skills, &skills_status_map);
        }
        skills
    }).await.map_err(|e| e.to_string())?;

    // Update memory cache
    {
        if let Ok(mut cache) = get_skills_cache().lock() {
            *cache = Some(all_skills.clone());
        }
    }
    
    println!("DEBUG: [discover_all_skills] complete, {} skills found", all_skills.len());
    Ok(all_skills)
}

fn scan_directory_for_skills(
    root: &Path,
    repo_id: &str,
    skills: &mut Vec<SkillMetadata>,
    skills_status: &HashMap<String, SkillStatus>,
) {
    let walker = walkdir::WalkDir::new(root).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(skill) = parse_markdown_skill(path, root, repo_id, skills_status) {
                    skills.push(skill);
                }
            }
        }
    }
}

fn parse_markdown_skill(
    file_path: &Path,
    root: &Path,
    repo_id: &str,
    skills_status: &HashMap<String, SkillStatus>,
) -> Option<SkillMetadata> {
    let mut file = File::open(file_path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;

    // Look for YAML frontmatter between "---" and "---" at the very beginning
    if !contents.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = contents.split("---").collect();
    if parts.len() < 3 {
        return None;
    }

    let yaml_content = parts[1];
    let mut id = String::new();
    let mut name = String::new();
    let mut description = String::new();

    // Custom simple line-by-line parser to extract frontmatter tags to avoid heavy dependencies
    for line in yaml_content.lines() {
        let line = line.trim();
        if line.starts_with("id:") {
            id = line.trim_start_matches("id:").trim().to_string();
        } else if line.starts_with("name:") {
            name = line.trim_start_matches("name:").trim().to_string();
        } else if line.starts_with("description:") {
            description = line.trim_start_matches("description:").trim().to_string();
        }
    }

    // Clean up quotes if present in name/description/id
    let clean = |s: String| s.replace('"', "").replace('\'', "");
    id = clean(id);
    name = clean(name);
    description = clean(description);

    if id.is_empty() {
        // Fallback to parent directory name if the filename is generic (e.g., SKILL.md), otherwise use file name stem
        let stem = file_path.file_stem()?.to_string_lossy().into_owned();
        let upper_stem = stem.to_uppercase();
        if upper_stem == "SKILL" || upper_stem == "README" || upper_stem == "CLAUDE" || upper_stem == "GEMINI" {
            if let Some(parent) = file_path.parent() {
                id = parent.file_name()?.to_string_lossy().into_owned();
            } else {
                id = stem;
            }
        } else {
            id = stem;
        }
    }
    if name.is_empty() {
        name = id.clone();
    }

    let relative_path = file_path.strip_prefix(root).ok()?
        .to_string_lossy().into_owned();

    // Check if skill is already in staging (check both staging subfolder and my-brain root folder)
    let is_installed = {
        let staging_exists = get_staging_path().join(format!("{}.md", id)).exists() ||
                             get_my_brain_path().join(format!("{}.md", id)).exists();
        if staging_exists {
            if let Some(status) = skills_status.get(&id) {
                status.repo_id == repo_id
            } else {
                // Fallback: compare file contents
                let get_content = |p: &Path| -> Option<String> {
                    let mut file = File::open(p).ok()?;
                    let mut c = String::new();
                    file.read_to_string(&mut c).ok()?;
                    Some(c)
                };
                let staged_content = get_content(&get_staging_path().join(format!("{}.md", id)))
                    .or_else(|| get_content(&get_my_brain_path().join(format!("{}.md", id))));
                if let Some(staged_c) = staged_content {
                    let mut repo_content = String::new();
                    if let Ok(mut f) = File::open(file_path) {
                        f.read_to_string(&mut repo_content).is_ok() && staged_c == repo_content
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        } else {
            false
        }
    };
    let is_downloaded = is_installed;

    Some(SkillMetadata {
        id,
        name,
        description,
        repo_id: repo_id.to_string(),
        relative_path,
        is_installed,
        is_downloaded,
    })
}

// ==========================================
// 5. AGY Config Integration (installed_skills index)
// ==========================================

/// Structure mirroring AGY's `~/.gemini/config.json` installed_skills entries
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AgyConfig {
    #[serde(default)]
    pub installed_skills: Vec<AgySkillEntry>,
    #[serde(default)]
    pub active_skills: Vec<AgySkillEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgySkillEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub enabled: bool,
}

/// Synchronise AGY's global `config.json` with our skill state.
/// On enable: appends the skill to `installed_skills` (if absent).
/// On disable: removes the skill from `installed_skills`.
fn sync_agy_config_json(skill_id: &str, skill_name: &str, enable_agy: bool, enable_agy2: bool, scope: &str) -> Result<(), String> {
    let username = home::home_dir()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .ok_or_else(|| "Could not determine user home directory".to_string())?;

    let agy_config_path = PathBuf::from(format!(
        "C:\\Users\\{}\\.gemini\\config.json",
        username
    ));

    // Read existing or create default
    let mut agy_cfg: AgyConfig = if agy_config_path.exists() {
        let content = fs::read_to_string(&agy_config_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AgyConfig::default()
    };

    let enable = enable_agy || enable_agy2;

    let skill_path = if scope == "project" || scope == "workspace" {
        get_project_root().join(format!(".agents\\skills\\{}\\SKILL.md", skill_id))
            .to_string_lossy().to_string()
    } else if scope == "shared" {
        format!(
            "C:\\Users\\{}\\.gemini\\skills\\{}\\SKILL.md",
            username, skill_id
        )
    } else if enable_agy2 {
        format!(
            "C:\\Users\\{}\\.gemini\\antigravity\\skills\\{}\\SKILL.md",
            username, skill_id
        )
    } else {
        format!(
            "C:\\Users\\{}\\.gemini\\antigravity-cli\\skills\\{}\\SKILL.md",
            username, skill_id
        )
    };

    if enable {
        // Remove old entry to update path
        agy_cfg.installed_skills.retain(|s| s.id != skill_id);
        agy_cfg.active_skills.retain(|s| s.id != skill_id);

        agy_cfg.installed_skills.push(AgySkillEntry {
            id: skill_id.to_string(),
            name: skill_name.to_string(),
            path: skill_path.clone(),
            enabled: true,
        });
        agy_cfg.active_skills.push(AgySkillEntry {
            id: skill_id.to_string(),
            name: skill_name.to_string(),
            path: skill_path,
            enabled: true,
        });
    } else {
        // Remove from both arrays
        agy_cfg.installed_skills.retain(|s| s.id != skill_id);
        agy_cfg.active_skills.retain(|s| s.id != skill_id);
    }

    // Ensure parent directory exists
    if let Some(parent) = agy_config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let data = serde_json::to_string_pretty(&agy_cfg).map_err(|e| e.to_string())?;
    fs::write(&agy_config_path, data).map_err(|e| format!(
        "Failed to write AGY config at {}: {}",
        agy_config_path.display(), e
    ))?;

    Ok(())
}

// ==========================================
// 6. Skill Execution Switch Controls
// ==========================================

#[tauri::command]
async fn install_skill(repo_id: String, relative_path: String, skill_id: String) -> Result<(), String> {
    let source_path = get_repos_cache_path().join(&repo_id).join(&relative_path);
    if !source_path.exists() {
        return Err("Source skill file not found. Ensure repository is fully synchronized.".to_string());
    }

    let dest_path = get_staging_path().join(format!("{}.md", skill_id));
    tokio::fs::copy(&source_path, &dest_path).await.map_err(|e| format!("Failed to install skill into staging: {}", e))?;

    // Record or update the repository reference in config's skills_status
    let mut config = get_config_internal().await?;
    let mut skill_stat = config.skills_status.get(&skill_id).cloned().unwrap_or_else(|| SkillStatus {
        repo_id: repo_id.clone(),
        scope: "global".to_string(),
        enable_agy: false,
        enable_agy2: false,
        enable_reasonix: false,
        auto_update: true,
    });
    skill_stat.repo_id = repo_id.clone();
    config.skills_status.insert(skill_id.clone(), skill_stat);
    save_config(config).await?;

    invalidate_skills_cache();
    Ok(())
}

#[tauri::command]
async fn toggle_skill_switch(
    skill_id: String,
    repo_id: String,
    switch_type: String, // "agy", "agy2", "reasonix", "auto_update"
    status: bool,
    scope: Option<String>, // "global" | "project", None = keep existing
) -> Result<AppConfig, String> {
    let mut config = get_config_internal().await?;
    
    // Get or insert skill status
    let mut skill_stat = config.skills_status.get(&skill_id).cloned().unwrap_or_else(|| SkillStatus {
        repo_id: repo_id.clone(),
        scope: scope.clone().unwrap_or_else(|| "global".to_string()),
        enable_agy: false,
        enable_agy2: false,
        enable_reasonix: false,
        auto_update: true,
    });

    // If scope was provided, update it
    if let Some(s) = scope {
        skill_stat.scope = s;
    }

    match switch_type.as_str() {
        "agy" => skill_stat.enable_agy = status,
        "agy2" => skill_stat.enable_agy2 = status,
        "reasonix" => skill_stat.enable_reasonix = status,
        "auto_update" => skill_stat.auto_update = status,
        _ => return Err("Invalid switch type".to_string()),
    }

    config.skills_status.insert(skill_id.clone(), skill_stat);
    save_config(config.clone()).await?;

    // Perform physical changes based on toggle action
    sync_physical_distributions_for_skill(&skill_id, &config).await?;
    invalidate_skills_cache();

    Ok(config)
}

#[tauri::command]
async fn update_skill_scope(
    skill_id: String,
    repo_id: String,
    scope: String,
) -> Result<AppConfig, String> {
    let mut config = get_config_internal().await?;
    
    // First, physically clean up existing distribution paths under the old scope
    let _ = remove_physical_distribution(&skill_id).await;

    let mut skill_stat = config.skills_status.get(&skill_id).cloned().unwrap_or_else(|| SkillStatus {
        repo_id: repo_id.clone(),
        scope: scope.clone(),
        enable_agy: false,
        enable_agy2: false,
        enable_reasonix: false,
        auto_update: true,
    });
    
    skill_stat.scope = scope;
    config.skills_status.insert(skill_id.clone(), skill_stat);
    save_config(config.clone()).await?;

    // Now, re-distribute files under the new scope
    let _ = sync_physical_distributions_for_skill(&skill_id, &config).await;
    invalidate_skills_cache();

    Ok(config)
}

/// Resolve the destination path for a skill based on CLI type, scope, and action.
fn resolve_scope_path(skill_id: &str, scope: &str, cli_type: &str) -> Result<PathBuf, String> {
    let username = home::home_dir()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .ok_or_else(|| "Could not determine user home directory".to_string())?;
    let project_root = get_project_root();

    match (cli_type, scope) {
        // Reasonix - Global:  ~/.reasonix/skills/<id>.md
        ("reasonix", "global") => Ok(
            PathBuf::from(format!("C:\\Users\\{}\\.reasonix\\skills", username))
                .join(format!("{}.md", skill_id))
        ),
        // Reasonix - Project:  <project>/.reasonix/skills/<id>.md
        ("reasonix", "project") => Ok(
            project_root.join(".reasonix\\skills").join(format!("{}.md", skill_id))
        ),
        // AGY - Global:  C:\Users\<username>\.gemini\antigravity-cli\skills\<skill_id>\SKILL.md
        ("agy", "global") => Ok(
            PathBuf::from(format!("C:\\Users\\{}\\.gemini\\antigravity-cli\\skills", username))
                .join(skill_id).join("SKILL.md")
        ),
        // AGY 2.0 - Global:  C:\Users\<username>\.gemini\antigravity\skills\<skill_id>\SKILL.md
        ("agy2", "global") => Ok(
            PathBuf::from(format!("C:\\Users\\{}\\.gemini\\antigravity\\skills", username))
                .join(skill_id).join("SKILL.md")
        ),
        // AGY / AGY 2.0 - Project/Workspace:  <project>/.agents/skills/<skill_id>/SKILL.md
        ("agy", "project") | ("agy", "workspace") | ("agy2", "project") | ("agy2", "workspace") => Ok(
            project_root.join(".agents\\skills").join(skill_id).join("SKILL.md")
        ),
        // AGY / AGY 2.0 - Shared:  C:\Users\<username>\.gemini\skills\<skill_id>\SKILL.md
        ("agy", "shared") | ("agy2", "shared") => Ok(
            PathBuf::from(format!("C:\\Users\\{}\\.gemini\\skills", username))
                .join(skill_id).join("SKILL.md")
        ),
        _ => Err(format!("Unknown CLI type '{}' or scope '{}'", cli_type, scope)),
    }
}

async fn sync_physical_distributions_for_skill(skill_id: &str, config: &AppConfig) -> Result<(), String> {
    let skill_status = config.skills_status.get(skill_id)
        .ok_or_else(|| "Skill status not found".to_string())?;

    let scope = &skill_status.scope;

    let staging_file = get_staging_path().join(format!("{}.md", skill_id));
    if !staging_file.exists() {
        return Err("Skill is not downloaded yet. Please download the skill first.".to_string());
    }

    let staging_content = tokio::fs::read_to_string(&staging_file).await.map_err(|e| e.to_string())?;

    // ==================================================================
    // AGY — copy raw markdown into <scope>/<skill_id>/SKILL.md folder
    // ==================================================================
    let agy_path = resolve_scope_path(skill_id, scope, "agy")?;
    if skill_status.enable_agy {
        let agy_dir = agy_path.parent()
            .ok_or_else(|| "Invalid AGY path".to_string())?;
        tokio::fs::create_dir_all(agy_dir).await.map_err(|e| e.to_string())?;
        tokio::fs::write(&agy_path, &staging_content).await.map_err(|e| format!(
            "Failed to write AGY file {}: {}", agy_path.display(), e
        ))?;
    } else {
        if let Some(parent) = agy_path.parent() {
            if parent.exists() {
                let _ = tokio::fs::remove_dir_all(parent).await;
            }
        }
    }

    // ==================================================================
    // AGY 2.0 — copy raw markdown into <scope>/<skill_id>/SKILL.md folder
    // ==================================================================
    let agy2_path = resolve_scope_path(skill_id, scope, "agy2")?;
    if skill_status.enable_agy2 {
        let agy2_dir = agy2_path.parent()
            .ok_or_else(|| "Invalid AGY2 path".to_string())?;
        tokio::fs::create_dir_all(agy2_dir).await.map_err(|e| e.to_string())?;
        tokio::fs::write(&agy2_path, &staging_content).await.map_err(|e| format!(
            "Failed to write AGY2 file {}: {}", agy2_path.display(), e
        ))?;
    } else {
        if let Some(parent) = agy2_path.parent() {
            if parent.exists() {
                let _ = tokio::fs::remove_dir_all(parent).await;
            }
        }
    }

    // ==================================================================
    // Reasonix — inject standardised frontmatter then write .md file
    // ==================================================================
    let reasonix_path = resolve_scope_path(skill_id, scope, "reasonix")?;
    if skill_status.enable_reasonix {
        let reasonix_dir = reasonix_path.parent()
            .ok_or_else(|| "Invalid Reasonix path".to_string())?;
        tokio::fs::create_dir_all(reasonix_dir).await.map_err(|e| e.to_string())?;

        // Force Reasonix-compatible frontmatter with official fields
        let normalized = normalize_for_reasonix(skill_id, &staging_content);
        tokio::fs::write(&reasonix_path, normalized).await.map_err(|e| format!(
            "Failed to write Reasonix file {}: {}", reasonix_path.display(), e
        ))?;
    } else if reasonix_path.exists() {
        let _ = tokio::fs::remove_file(&reasonix_path).await;
    }

    // ==================================================================
    // AGY config.json index sync
    // ==================================================================
    let skill_id_clone = skill_id.to_string();
    let skill_name_clone = skill_id.to_string();
    let enable_agy = skill_status.enable_agy;
    let enable_agy2 = skill_status.enable_agy2;
    let scope_clone = scope.clone();
    // Fire-and-forget: NEVER await blocking tasks on the async runtime
    tokio::spawn(async move {
        let _ = tokio::task::spawn_blocking(move || {
            let _ = sync_agy_config_json(&skill_id_clone, &skill_name_clone, enable_agy, enable_agy2, &scope_clone);
        }).await;
    });

    Ok(())
}

/// Ensures the markdown content has a Reasonix-compatible frontmatter with `name:` field.
fn normalize_for_reasonix(skill_id: &str, content: &str) -> String {
    let description = extract_frontmatter_value(content, "description");
    let desc_str = if !description.is_empty() {
        description
    } else {
        format!("AI Agent skill: {}", skill_id)
    };

    let frontmatter = format!(
        concat!(
            "---\n",
            "name: {}\n",
            "description: {}\n",
            "runAs: inline\n",
            "allowed-tools: bash,read\n",
            "model: deepseek-chat\n",
            "max-iters: 16\n",
            "---\n"
        ),
        skill_id, desc_str
    );

    let body = if content.starts_with("---") {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3 { parts[2].trim() } else { content.trim() }
    } else {
        content.trim()
    };

    format!("{}{}\n", frontmatter, body)
}

/// Helper: 从原始 Markdown 中提取 YAML Frontmatter 字段值
fn extract_frontmatter_value(content: &str, key: &str) -> String {
    if !content.starts_with("---") {
        return String::new();
    }
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 { return String::new(); }
    let yaml = parts[1];
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}:", key)) {
            let val = trimmed.trim_start_matches(&format!("{}:", key)).trim();
            return val.trim_matches('"').trim_matches('\'').to_string();
        }
    }
    String::new()
}

async fn remove_physical_distribution(skill_id: &str) -> Result<(), String> {
    let username = home::home_dir()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .ok_or_else(|| "Could not determine user home directory".to_string())?;
    let project_root = get_project_root();

    // Clean Reasonix files
    let reasonix_paths = [
        PathBuf::from(format!("C:\\Users\\{}\\.reasonix\\skills\\{}.md", username, skill_id)),
        project_root.join(format!(".reasonix\\skills\\{}.md", skill_id)),
    ];
    for path in &reasonix_paths {
        if path.exists() {
            let _ = tokio::fs::remove_file(path).await;
        }
    }

    // Clean AGY (both files and folders recursively to prevent residue)
    let agy_dirs = [
        // Global
        PathBuf::from(format!("C:\\Users\\{}\\.gemini\\antigravity-cli\\skills\\{}", username, skill_id)),
        // Project
        project_root.join(format!(".agents\\skills\\{}", skill_id)),
        // Shared
        PathBuf::from(format!("C:\\Users\\{}\\.gemini\\skills\\{}", username, skill_id)),
        // Legacy (incorrect path without -cli)
        PathBuf::from(format!("C:\\Users\\{}\\.gemini\\antigravity\\skills\\{}", username, skill_id)),
    ];

    for dir in &agy_dirs {
        if dir.exists() {
            let _ = tokio::fs::remove_dir_all(dir).await;
        }
    }

    // Clean staging
    let staging_file = get_staging_path().join(format!("{}.md", skill_id));
    if staging_file.exists() {
        let _ = tokio::fs::remove_file(&staging_file).await;
    }

    // Clean my-brain root staging too (Patch A requirement)
    let my_brain_file = get_my_brain_path().join(format!("{}.md", skill_id));
    if my_brain_file.exists() {
        let _ = tokio::fs::remove_file(&my_brain_file).await;
    }

    // Clean AGY config.json indices
    let skill_id_clone1 = skill_id.to_string();
    let skill_id_clone2 = skill_id.to_string();
    let skill_id_clone3 = skill_id.to_string();
    // Fire-and-forget: NEVER await blocking tasks on the async runtime
    tokio::spawn(async move {
        let _ = tokio::task::spawn_blocking(move || {
            let _ = sync_agy_config_json(&skill_id_clone1, &skill_id_clone1, false, false, "global");
            let _ = sync_agy_config_json(&skill_id_clone2, &skill_id_clone2, false, false, "project");
            let _ = sync_agy_config_json(&skill_id_clone3, &skill_id_clone3, false, false, "shared");
        }).await;
    });

    Ok(())
}

/// Called on app startup from JS loadApp().
#[tauri::command]
async fn startup_sync_distributions() -> Result<(), String> {
    println!("DEBUG: [startup_sync_distributions] START");
    let config = get_config_internal().await?;
    
    // First, sync/cleanup each repository based on local cached files
    for repo in &config.repositories {
        let _ = auto_update_and_cleanup_repo(&repo.id).await;
    }
    
    // Now re-read config
    let config = get_config_internal().await?;
    
    println!("DEBUG: [startup_sync_distributions] {} skills in status", config.skills_status.len());
    let mut errors: Vec<String> = Vec::new();

    for (skill_id, status) in &config.skills_status {
        if status.enable_agy || status.enable_agy2 || status.enable_reasonix {
            let staging_file = get_staging_path().join(format!("{}.md", skill_id));
            println!("DEBUG: [startup_sync_distributions] skill={} staging_exists={}", skill_id, staging_file.exists());
            if staging_file.exists() {
                println!("DEBUG: [startup_sync_distributions] syncing skill {}...", skill_id);
                if let Err(e) = sync_physical_distributions_for_skill(skill_id, &config).await {
                    errors.push(format!("{}: {}", skill_id, e));
                    println!("DEBUG: [startup_sync_distributions] skill {} error: {}", skill_id, e);
                }
            }
        }
    }

    invalidate_skills_cache();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("Partial sync errors: {}", errors.join("; ")))
    }
}

#[tauri::command]
async fn sync_skill_now(skill_id: String, repo_id: String, relative_path: String) -> Result<(), String> {
    // 1. Pull the repository first
    let repo_path = get_repos_cache_path().join(&repo_id);
    if repo_path.exists() {
        let _ = git_pull_internal(&repo_path).await;
    }

    // 2. Refresh staging
    let source_path = repo_path.join(&relative_path);
    if !source_path.exists() {
        return Err("Skill file not found in repository cache.".to_string());
    }

    let dest_path = get_staging_path().join(format!("{}.md", skill_id));
    tokio::fs::copy(&source_path, &dest_path).await.map_err(|e| format!("Failed to copy updated file: {}", e))?;

    // 3. Redistribute files
    let mut config = get_config_internal().await?;
    let mut skill_stat = config.skills_status.get(&skill_id).cloned().unwrap_or_else(|| SkillStatus {
        repo_id: repo_id.clone(),
        scope: "global".to_string(),
        enable_agy: false,
        enable_agy2: false,
        enable_reasonix: false,
        auto_update: true,
    });
    skill_stat.repo_id = repo_id.clone();
    config.skills_status.insert(skill_id.clone(), skill_stat);
    save_config(config.clone()).await?;

    sync_physical_distributions_for_skill(&skill_id, &config).await?;

    invalidate_skills_cache();
    Ok(())
}

// ==========================================
// 6. WebDAV Backup & Clouds Resurrect
// ==========================================

fn zip_my_brain(zip_file_path: &Path, my_brain_path: &Path) -> Result<(), String> {
    let file = File::create(zip_file_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in walkdir::WalkDir::new(my_brain_path) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        // Skip the repos cache folder and the target zip file itself!
        if path.starts_with(my_brain_path.join("repos")) || path == zip_file_path {
            continue;
        }

        let name = path.strip_prefix(my_brain_path)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .into_owned();

        if path.is_file() {
            zip.start_file(name.replace('\\', "/"), options).map_err(|e| e.to_string())?;
            let mut f = File::open(path).map_err(|e| e.to_string())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            zip.write_all(&buffer).map_err(|e| e.to_string())?;
        } else if !name.is_empty() {
            zip.add_directory(name.replace('\\', "/"), options).map_err(|e| e.to_string())?;
        }
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

async fn backup_to_webdav_internal(config: &AppConfig) -> Result<String, String> {
    let config_file = get_config_path();
    if !config_file.exists() {
        return Err("config.json not found".to_string());
    }

    // Standard zip archive path (temporary location)
    let backup_zip_path = get_my_brain_path().join("config_backup_temp.zip");
    zip_my_brain(&backup_zip_path, &get_my_brain_path())?;

    // Create client with longer timeout for upload
    let client = create_webdav_download_client();
    let auth_header = get_auth_header(config);

    // 1. Create or verify dedicated 'new-SkillControl' WebDAV directory via MKCOL
    let mut folder_url = config.webdav.url.clone();
    if !folder_url.ends_with('/') {
        folder_url.push('/');
    }
    folder_url.push_str("new-SkillControl/");

    let _mkcol_res = send_webdav_request(
        &client,
        reqwest::Method::from_bytes(b"MKCOL").unwrap(),
        &folder_url,
        &auth_header,
        None,
        None,
        None,
        None,
    ).await;

    // 2. Generate chronological backup name: backup-YYYY-M-D-H-Min-S.zip
    let now = chrono::Local::now();
    let filename = format!("backup-{}-{}-{}-{}-{}-{}.zip", 
        now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());

    let mut upload_url = folder_url;
    upload_url.push_str(&filename);

    let zip_bytes = fs::read(&backup_zip_path).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&backup_zip_path); // clean up temp zip file

    let response = send_webdav_request(
        &client,
        reqwest::Method::PUT,
        &upload_url,
        &auth_header,
        None,
        None,
        Some("application/zip"),
        Some(zip_bytes),
    ).await?;

    if response.status().is_success() {
        Ok(filename)
    } else {
        Err(format!("WebDAV backup failed with status: {}", response.status()))
    }
}

#[tauri::command]
async fn trigger_backup() -> Result<String, String> {
    let config = get_config_internal().await?;
    let filename = backup_to_webdav_internal(&config).await?;
    Ok(format!("备份成功！已打包并上传至云端文件：{}", filename))
}

#[tauri::command]
async fn trigger_resurrect() -> Result<AppConfig, String> {
    // Legacy generic resurrection for compatibility, downloads standard config_backup.zip from root
    let config = get_config_internal().await?;
    
    let mut url = config.webdav.url.clone();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("config_backup.zip");
    println!("DEBUG: [trigger_resurrect] downloading from URL: {}", url);

    // Use download client with 120s timeout for ZIP download
    let client = create_webdav_download_client();
    let auth_header = get_auth_header(&config);

    // 1. Download Backup ZIP with WebDAV headers
    let response = send_webdav_request(
        &client,
        reqwest::Method::GET,
        &url,
        &auth_header,
        Some("0"),
        Some("f"),
        None,
        None,
    ).await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        println!("DEBUG: [trigger_resurrect] GET failed with {} — body: {}", status, &body[..body.len().min(200)]);
        return Err(format!("下载失败: 服务器返回 {} — 请确认 config_backup.zip 存在且有读取权限", status));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    println!("DEBUG: [trigger_resurrect] downloaded {} bytes", bytes.len());
    let zip_path = get_my_brain_path().join("downloaded_backup.zip");
    fs::write(&zip_path, bytes).map_err(|e| e.to_string())?;

    // 2. Unzip & Recover config.json
    let recovered_contents = {
        let file = File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let mut config_file_in_zip = archive.by_name("config.json").map_err(|e| e.to_string())?;
        let mut recovered_contents = String::new();
        config_file_in_zip.read_to_string(&mut recovered_contents).map_err(|e| e.to_string())?;
        recovered_contents
    };

    // Save recovered config
    let restored_config_path = get_config_path();
    fs::write(&restored_config_path, &recovered_contents).map_err(|e| e.to_string())?;

    // Parse recovered config
    let recovered_config: AppConfig = serde_json::from_str(&recovered_contents).map_err(|e| e.to_string())?;

    // 3. Automated Git clones & physical files reconstruction
    for repo in &recovered_config.repositories {
        let repo_path = get_repos_cache_path().join(&repo.id);
        if !repo_path.exists() {
            let _ = git_clone_internal(&repo.url, &repo_path).await;
        }
    }

    // 4. Redispatch all active skills that have cached md files
    for (skill_id, _status) in &recovered_config.skills_status {
        // Search locally in all cloned repositories for this skill
        let staging_file = get_staging_path().join(format!("{}.md", skill_id));
        if !staging_file.exists() {
            // Find in repositories
            let mut found_path = None;
            let mut _found_repo = None;
            for repo in &recovered_config.repositories {
                let repo_path = get_repos_cache_path().join(&repo.id);
                if repo_path.exists() {
                    let mut skills_list = Vec::new();
                    scan_directory_for_skills(&repo_path, &repo.id, &mut skills_list, &recovered_config.skills_status);
                    if let Some(matched) = skills_list.iter().find(|s| s.id == *skill_id) {
                        found_path = Some(repo_path.join(&matched.relative_path));
                        _found_repo = Some(repo.id.clone());
                        break;
                    }
                }
            }

            if let Some(src) = found_path {
                let _ = tokio::fs::copy(&src, &staging_file).await;
            }
        }

        if staging_file.exists() {
            let _ = sync_physical_distributions_for_skill(skill_id, &recovered_config).await;
        }
    }

    invalidate_skills_cache();
    Ok(recovered_config)
}

#[tauri::command]
async fn get_backup_list() -> Result<Vec<HashMap<String, String>>, String> {
    let config = get_config_internal().await?;
    let mut url = config.webdav.url.clone();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("new-SkillControl/");

    let client = create_webdav_client();
    let auth_header = get_auth_header(&config);

    // Send PROPFIND
    let response = send_webdav_request(
        &client,
        reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
        &url,
        &auth_header,
        Some("1"),
        None,
        None,
        None,
    ).await?;

    if !response.status().is_success() {
        return Err(format!("WebDAV returned status: {}", response.status()));
    }

    let text = response.text().await.map_err(|e| e.to_string())?;
    
    // Scan WebDAV PROPFIND hrefs matching backup-*.zip
    let mut backups = Vec::new();
    let mut search_str = &text[..];
    while let Some(start_idx) = search_str.find("backup-") {
        let rest = &search_str[start_idx..];
        if let Some(end_idx) = rest.find(".zip") {
            let filename = &rest[..end_idx + 4];
            if filename.chars().all(|c| c.is_numeric() || c == '-' || c == '.' || c == 'b' || c == 'a' || c == 'c' || c == 'k' || c == 'u' || c == 'p' || c == 'z' || c == 'i') {
                if !backups.contains(&filename.to_string()) {
                    backups.push(filename.to_string());
                }
            }
            search_str = &rest[end_idx + 4..];
        } else {
            break;
        }
    }

    // Parse backup filenames to ensure correct descending sorting chronological sort
    let mut parsed_backups: Vec<(Vec<u32>, String)> = backups.into_iter().filter_map(|name| {
        let parts_str = name.strip_prefix("backup-")?.strip_suffix(".zip")?;
        let parts: Vec<u32> = parts_str.split('-').filter_map(|p| p.parse::<u32>().ok()).collect();
        if parts.len() >= 6 {
            Some((parts, name))
        } else {
            None
        }
    }).collect();

    parsed_backups.sort_by(|a, b| b.0.cmp(&a.0));

    let result = parsed_backups.into_iter().map(|(parts, name)| {
        let mut map = HashMap::new();
        map.insert("filename".to_string(), name);
        let formatted = format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} 备份版本", 
            parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]);
        map.insert("display".to_string(), formatted);
        map
    }).collect();

    Ok(result)
}

#[tauri::command]
async fn trigger_restore_version(filename: String) -> Result<AppConfig, String> {
    let config = get_config_internal().await?;
    let client = create_webdav_download_client();
    let auth_header = get_auth_header(&config);

    // Step 1: PROPFIND the new-SkillControl/ folder to get the SERVER's exact href for this file.
    // Some WebDAV servers (e.g. 115 drive) use different internal paths than what we construct.
    let mut folder_url = config.webdav.url.clone();
    if !folder_url.ends_with('/') { folder_url.push('/'); }
    folder_url.push_str("new-SkillControl/");

    println!("DEBUG: [trigger_restore_version] PROPFIND folder: {}", folder_url);
    let propfind = send_webdav_request(
        &client,
        reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
        &folder_url,
        &auth_header,
        Some("1"),
        None,
        None,
        None,
    ).await?;

    if !propfind.status().is_success() {
        return Err(format!("文件夹查询失败: 服务器返回 {}", propfind.status()));
    }

    let xml = propfind.text().await.map_err(|e| e.to_string())?;

    // Extract the href from PROPFIND XML that contains our filename
    let search_for = &filename; // e.g. "backup-2026-5-27-18-00-00.zip"
    let download_url = {
        let mut best = None;
        let lower_search = search_for.to_lowercase();
        
        let hrefs = extract_hrefs_from_xml(&xml);
        for href in hrefs {
            let lower_href = href.to_lowercase();
            if lower_href.contains(&lower_search) && lower_href.ends_with(".zip") && lower_href.contains("backup-") {
                best = Some(href);
                break;
            }
        }
        best.ok_or_else(|| format!("在 WebDAV 响应中未找到匹配的备份文件: {}", filename))?
    };

    println!("DEBUG: [trigger_restore_version] server href: {}", download_url);

    // Ensure download_url is absolute
    let final_url = if download_url.starts_with("http://") || download_url.starts_with("https://") {
        download_url.clone()
    } else if download_url.starts_with('/') {
        // Relative to server root — extract scheme+host from config.webdav.url
        let base = config.webdav.url.trim_end_matches('/');
        // Find the server root (scheme://host:port)
        if let Some(slash_pos) = base.find("://") {
            if let Some(next_slash) = base[slash_pos + 3..].find('/') {
                let server_root = &base[..slash_pos + 3 + next_slash];
                format!("{}{}", server_root, download_url)
            } else {
                format!("{}{}", base, download_url)
            }
        } else {
            format!("{}{}", base, download_url)
        }
    } else {
        // Relative to the folder — unlikely but handle it
        format!("{}{}", folder_url, download_url)
    };

    println!("DEBUG: [trigger_restore_version] final download URL: {}", final_url);

    // Step 2: Download the ZIP from the server's canonical URL
    let response = send_webdav_request(
        &client,
        reqwest::Method::GET,
        &final_url,
        &auth_header,
        Some("0"),
        Some("f"),
        None,
        None,
    ).await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        println!("DEBUG: [trigger_restore_version] GET failed with {} — body: {}", status, &body[..body.len().min(200)]);
        return Err(format!("下载失败: 服务器返回 {} — 请确认备份文件存在且有读取权限", status));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    println!("DEBUG: [trigger_restore_version] downloaded {} bytes", bytes.len());
    let my_brain_path = get_my_brain_path();
    let zip_path = my_brain_path.join("downloaded_backup_temp.zip");
    fs::write(&zip_path, bytes).map_err(|e| e.to_string())?;

    // 2. Wipe config.json, staging folder, and repos cache folder
    let config_path = get_config_path();
    if config_path.exists() {
        let _ = fs::remove_file(&config_path);
    }
    let staging_path = get_staging_path();
    if staging_path.exists() {
        let _ = fs::remove_dir_all(&staging_path);
    }
    let repos_path = get_repos_cache_path();
    if repos_path.exists() {
        let _ = fs::remove_dir_all(&repos_path);
    }

    // Recreate clean folders
    let _ = fs::create_dir_all(&staging_path);
    let _ = fs::create_dir_all(&repos_path);

    // 3. Unzip files back into my-brain folder
    {
        let file = File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let enclosed = file.enclosed_name();
            let outpath = match enclosed {
                Some(path) => my_brain_path.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }
    }

    let _ = fs::remove_file(&zip_path);

    // 4. Load the recovered config
    let recovered_contents = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let recovered_config: AppConfig = serde_json::from_str(&recovered_contents).map_err(|e| e.to_string())?;

    // 5. Re-clone all configured Git repositories
    for repo in &recovered_config.repositories {
        let repo_path = get_repos_cache_path().join(&repo.id);
        if !repo_path.exists() {
            let _ = git_clone_internal(&repo.url, &repo_path).await;
        }
    }

    // 6. Redistribute active skills
    for (skill_id, _status) in &recovered_config.skills_status {
        let staging_file = get_staging_path().join(format!("{}.md", skill_id));
        if !staging_file.exists() {
            let mut found_path = None;
            for repo in &recovered_config.repositories {
                let repo_path = get_repos_cache_path().join(&repo.id);
                if repo_path.exists() {
                    let mut skills_list = Vec::new();
                    scan_directory_for_skills(&repo_path, &repo.id, &mut skills_list, &recovered_config.skills_status);
                    if let Some(matched) = skills_list.iter().find(|s| s.id == *skill_id) {
                        found_path = Some(repo_path.join(&matched.relative_path));
                        break;
                    }
                }
            }
            if let Some(src) = found_path {
                let _ = tokio::fs::copy(&src, &staging_file).await;
            }
        }

        if staging_file.exists() {
            let _ = sync_physical_distributions_for_skill(skill_id, &recovered_config).await;
        }
    }

    invalidate_skills_cache();
    Ok(recovered_config)
}

// ==========================================
// 6b. Reasonix Reload Notification
// ==========================================

/// Notify Reasonix to reload its playbooks index.
/// WARNING: This command blocks the tokio runtime with synchronous process::wait().
/// It is NOT called on cold start, but is included here for completeness.
#[tauri::command]
async fn notify_reasonix_reload() -> Result<String, String> {
    let commands: [(&str, &[&str]); 3] = [
        ("reasonix", &["/playbooks"]),
        ("reasonix", &["/skills"]),
        ("npx",      &["reasonix", "/playbooks"]),
    ];

    for (cmd, args) in &commands {
        let result = TokioCommand::new(cmd)
            .args(*args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(_) => {
                return Ok(format!(
                    "Reasonix notified via `{} {}`",
                    cmd,
                    args.join(" ")
                ));
            }
            Err(e) => {
                println!("DEBUG: [notify_reasonix_reload] {} failed: {}", cmd, e);
            }
        }
    }

    Err(format!(
        "Could not reach Reasonix (all methods failed)"
    ))
}

// ==========================================
// 6c. Skill Uninstall & Physical Crushing
// ==========================================

#[tauri::command]
async fn uninstall_skill(skill_id: String) -> Result<AppConfig, String> {
    remove_physical_distribution(&skill_id).await?;

    let mut config = get_config_internal().await?;
    config.skills_status.remove(&skill_id);
    save_config(config.clone()).await?;

    invalidate_skills_cache();
    Ok(config)
}

#[tauri::command]
async fn get_skill_diff(skill_id: String, repo_id: String, relative_path: String) -> Result<HashMap<String, String>, String> {
    let staged_path = get_staging_path().join(format!("{}.md", skill_id));
    let repo_path = get_repos_cache_path().join(&repo_id).join(&relative_path);

    let staged_content = if staged_path.exists() {
        tokio::fs::read_to_string(&staged_path).await.unwrap_or_default()
    } else {
        String::new()
    };

    let repo_content = if repo_path.exists() {
        tokio::fs::read_to_string(&repo_path).await.unwrap_or_default()
    } else {
        String::new()
    };

    let mut result = HashMap::new();
    result.insert("staged".to_string(), staged_content);
    result.insert("repo".to_string(), repo_content);
    Ok(result)
}

// ==========================================
// 7. Builder Init
// ==========================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = app.path().app_data_dir().unwrap_or_else(|_| {
                app.path().app_config_dir().unwrap_or_else(|_| {
                    home::home_dir()
                        .map(|p| p.join("AppData").join("Roaming").join("new-SkillControl"))
                        .unwrap()
                })
            });
            // Force the exact requested path: C:\Users\<username>\AppData\Roaming\new-SkillControl
            let _target_path = home::home_dir()
                .map(|p| p.join("AppData").join("Roaming").join("new-SkillControl"))
                .unwrap_or(path);

            // Ensure cold start folders exist
            // NOTE: OnceLock is initialized lazily by the first call to get_app_data_path()
            let app_data = get_app_data_path();
            let my_brain = app_data.join("my-brain");
            if !my_brain.exists() {
                let _ = fs::create_dir_all(&my_brain);
            }
            let config_path = my_brain.join("config.json");
            if !config_path.exists() {
                let default_config = AppConfig::default();
                if let Ok(data) = serde_json::to_string_pretty(&default_config) {
                    let _ = fs::write(&config_path, data);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            add_repository,
            delete_repository,
            discover_skills,
            discover_all_skills,
            install_skill,
            toggle_skill_switch,
            update_skill_scope,
            sync_skill_now,
            sync_all_repositories,
            sync_single_repository,
            startup_sync_distributions,
            notify_reasonix_reload,
            trigger_backup,
            trigger_resurrect,
            get_backup_list,
            trigger_restore_version,
            uninstall_skill,
            get_skill_diff
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
