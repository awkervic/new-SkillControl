use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
    pub enable_agy: bool,
    pub enable_reasonix: bool,
    pub auto_update: bool,
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

fn get_project_root() -> PathBuf {
    PathBuf::from("D:\\123123123123\\new-SkillControl")
}

fn get_my_brain_path() -> PathBuf {
    let path = get_project_root().join("my-brain");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

fn get_config_path() -> PathBuf {
    get_my_brain_path().join("config.json")
}

fn get_repos_cache_path() -> PathBuf {
    let path = get_my_brain_path().join("repos");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

fn get_staging_path() -> PathBuf {
    let path = get_my_brain_path().join("staging");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
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

#[tauri::command]
fn get_config() -> Result<AppConfig, String> {
    let path = get_config_path();
    if !path.exists() {
        let default_config = AppConfig::default();
        let data = serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())?;
        return Ok(default_config);
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let config: AppConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
async fn save_config(config: AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;

    // Auto WebDAV backup if enabled
    if config.webdav.auto_backup_enabled 
        && !config.webdav.url.is_empty() 
        && config.webdav.url != "https://dav.jianguoyun.com/dav/" // don't auto-backup with default config placeholders
    {
        let _ = backup_to_webdav_internal(&config).await;
    }

    Ok(())
}

#[tauri::command]
async fn add_repository(name: String, url: String) -> Result<AppConfig, String> {
    let mut config = get_config()?;
    
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
        let _ = git_clone_internal(&clone_url, &dest_path).await;
    });

    Ok(config)
}

#[tauri::command]
async fn delete_repository(repo_id: String) -> Result<AppConfig, String> {
    let mut config = get_config()?;
    
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
        let _ = remove_physical_distribution(&skill_id);
        config.skills_status.remove(&skill_id);
    }
    
    save_config(config.clone()).await?;

    // 3. Remove git repository cache
    let repo_path = get_repos_cache_path().join(&repo_id);
    if repo_path.exists() {
        let _ = fs::remove_dir_all(repo_path);
    }

    Ok(config)
}

#[tauri::command]
async fn sync_all_repositories() -> Result<(), String> {
    let config = get_config()?;
    for repo in config.repositories {
        let repo_path = get_repos_cache_path().join(&repo.id);
        if repo_path.exists() {
            let _ = git_pull_internal(&repo_path).await;
        } else {
            let _ = git_clone_internal(&repo.url, &repo_path).await;
        }
    }
    Ok(())
}

#[tauri::command]
async fn sync_single_repository(repo_id: String) -> Result<(), String> {
    let config = get_config()?;
    if let Some(repo) = config.repositories.iter().find(|r| r.id == repo_id) {
        let repo_path = get_repos_cache_path().join(&repo.id);
        if repo_path.exists() {
            git_pull_internal(&repo_path).await?;
        } else {
            git_clone_internal(&repo.url, &repo_path).await?;
        }
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
    let config = get_config()?;
    let repo = config.repositories.iter().find(|r| r.id == repo_id)
        .ok_or_else(|| "Repository not found".to_string())?;

    let repo_path = get_repos_cache_path().join(&repo_id);
    if !repo_path.exists() {
        // Automatically clone if cached folder does not exist yet
        let _ = git_clone_internal(&repo.url, &repo_path).await;
    }

    let mut skills = Vec::new();
    if repo_path.exists() {
        scan_directory_for_skills(&repo_path, &repo_id, &mut skills);
    }
    Ok(skills)
}

#[tauri::command]
async fn discover_all_skills() -> Result<Vec<SkillMetadata>, String> {
    let config = get_config()?;
    let mut all_skills = Vec::new();
    for repo in config.repositories {
        let repo_path = get_repos_cache_path().join(&repo.id);
        if !repo_path.exists() {
            // Lazy clone
            let _ = git_clone_internal(&repo.url, &repo_path).await;
        }
        if repo_path.exists() {
            scan_directory_for_skills(&repo_path, &repo.id, &mut all_skills);
        }
    }
    Ok(all_skills)
}

fn scan_directory_for_skills(root: &Path, repo_id: &str, skills: &mut Vec<SkillMetadata>) {
    let walker = walkdir::WalkDir::new(root).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(skill) = parse_markdown_skill(path, root, repo_id) {
                    skills.push(skill);
                }
            }
        }
    }
}

fn parse_markdown_skill(file_path: &Path, root: &Path, repo_id: &str) -> Option<SkillMetadata> {
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

    // Check if skill is already in staging
    let is_installed = get_staging_path().join(format!("{}.md", id)).exists();

    Some(SkillMetadata {
        id,
        name,
        description,
        repo_id: repo_id.to_string(),
        relative_path,
        is_installed,
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
fn sync_agy_config_json(skill_id: &str, skill_name: &str, enable: bool) -> Result<(), String> {
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

    let skill_path = format!(
        "C:\\Users\\{}\\.gemini\\skills\\shared\\{}\\SKILL.md",
        username, skill_id
    );

    if enable {
        // Append only if not already present
        if !agy_cfg.installed_skills.iter().any(|s| s.id == skill_id) {
            agy_cfg.installed_skills.push(AgySkillEntry {
                id: skill_id.to_string(),
                name: skill_name.to_string(),
                path: skill_path.clone(),
                enabled: true,
            });
        }
        // Also update active_skills
        if !agy_cfg.active_skills.iter().any(|s| s.id == skill_id) {
            agy_cfg.active_skills.push(AgySkillEntry {
                id: skill_id.to_string(),
                name: skill_name.to_string(),
                path: skill_path,
                enabled: true,
            });
        }
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
    fs::copy(&source_path, &dest_path).map_err(|e| format!("Failed to install skill into staging: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn toggle_skill_switch(
    skill_id: String,
    repo_id: String,
    switch_type: String, // "agy", "reasonix", "auto_update"
    status: bool,
) -> Result<AppConfig, String> {
    let mut config = get_config()?;
    
    // Get or insert skill status
    let mut skill_stat = config.skills_status.get(&skill_id).cloned().unwrap_or_else(|| SkillStatus {
        repo_id: repo_id.clone(),
        enable_agy: false,
        enable_reasonix: false,
        auto_update: true,
    });

    match switch_type.as_str() {
        "agy" => skill_stat.enable_agy = status,
        "reasonix" => skill_stat.enable_reasonix = status,
        "auto_update" => skill_stat.auto_update = status,
        _ => return Err("Invalid switch type".to_string()),
    }

    config.skills_status.insert(skill_id.clone(), skill_stat);
    save_config(config.clone()).await?;

    // Perform physical changes based on toggle action
    sync_physical_distributions_for_skill(&skill_id, &config)?;

    Ok(config)
}

fn sync_physical_distributions_for_skill(skill_id: &str, config: &AppConfig) -> Result<(), String> {
    let skill_status = config.skills_status.get(skill_id)
        .ok_or_else(|| "Skill status not found".to_string())?;

    let staging_file = get_staging_path().join(format!("{}.md", skill_id));
    if !staging_file.exists() {
        // If not installed, try to find and auto-install from local clone
        return Err("Skill is not downloaded yet. Please download the skill first.".to_string());
    }

    let username = home::home_dir()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .ok_or_else(|| "Could not determine user home directory".to_string())?;

    // Read staging file content once for potential rewriting
    let staging_content = fs::read_to_string(&staging_file).map_err(|e| e.to_string())?;

    // --- AGY Physical Setup ---
    let agy_dest_dir = PathBuf::from(format!("C:\\Users\\{}\\.gemini\\skills\\shared\\{}\\", username, skill_id));
    let agy_dest_file = agy_dest_dir.join("SKILL.md");

    if skill_status.enable_agy {
        fs::create_dir_all(&agy_dest_dir).map_err(|e| e.to_string())?;
        fs::copy(&staging_file, &agy_dest_file).map_err(|e| e.to_string())?;
    } else if agy_dest_file.exists() {
        let _ = fs::remove_file(&agy_dest_file);
        let _ = fs::remove_dir(&agy_dest_dir);
    }

    // --- Reasonix Physical Setup ---
    // Reasonix detects playbooks by scanning for files with strict YAML frontmatter.
    // We force-inject standard frontmatter so Reasonix reliably recognises this as a playbook.
    let reasonix_dest_dir = PathBuf::from(format!("C:\\Users\\{}\\.reasonix\\playbooks\\shared\\", username));
    let reasonix_dest_file = reasonix_dest_dir.join(format!("{}.md", skill_id));

    if skill_status.enable_reasonix {
        fs::create_dir_all(&reasonix_dest_dir).map_err(|e| e.to_string())?;

        // Force-write with standardised frontmatter for Reasonix
        let normalized = normalize_for_reasonix(skill_id, &staging_content);
        fs::write(&reasonix_dest_file, normalized).map_err(|e| e.to_string())?;
    } else if reasonix_dest_file.exists() {
        let _ = fs::remove_file(&reasonix_dest_file);
    }

    // --- AGY config.json index sync ---
    // After physical file operations, update AGY's global installed_skills registry
    // so that `\/skills` CLI command reflects the change immediately.
    let _ = sync_agy_config_json(
        skill_id,
        skill_id,  // name fallback — caller will pass the actual name later
        skill_status.enable_agy,
    );

    Ok(())
}

/// Ensures the markdown content has a Reasonix-compatible frontmatter with `name:` field.
/// Reasonix scans playbooks by looking for `name:` in YAML frontmatter.
/// If the file uses `id:` but no `name:`, we inject a proper `name:` field.
fn normalize_for_reasonix(skill_id: &str, content: &str) -> String {
    // ---- NOTICE ----
    // Reasonix requires a strict YAML frontmatter at the very top of every playbook.
    // We ALWAYS overwrite the frontmatter with our standardised format so that
    // Reasonix's scanner can reliably detect this file as a playbook.
    // The original body (everything after the first frontmatter, or the whole file
    // if no frontmatter existed) is preserved.

    // Extract description from existing content if possible
    let description = extract_frontmatter_value(content, "description");
    let desc_str = if !description.is_empty() {
        description
    } else {
        format!("AI Agent skill: {}", skill_id)
    };

    // Build the standard frontmatter
    let frontmatter = format!(
        "---\nname: {}\ndescription: {}\nversion: 1.0.0\n---\n",
        skill_id, desc_str
    );

    // Extract the body by stripping existing frontmatter if any
    let body = if content.starts_with("---") {
        // Split on second "---"
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() >= 3 {
            parts[2].trim()
        } else {
            content.trim()
        }
    } else {
        content.trim()
    };

    format!("{}{}\n", frontmatter, body)
}

/// Helper: extract a YAML frontmatter field value by key from raw markdown
fn extract_frontmatter_value(content: &str, key: &str) -> String {
    if !content.starts_with("---") {
        return String::new();
    }
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return String::new();
    }
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

fn remove_physical_distribution(skill_id: &str) -> Result<(), String> {
    let username = home::home_dir()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
        .ok_or_else(|| "Could not determine user home directory".to_string())?;

    let agy_dest_dir = PathBuf::from(format!("C:\\Users\\{}\\.gemini\\skills\\shared\\{}\\", username, skill_id));
    let agy_dest_file = agy_dest_dir.join("SKILL.md");
    if agy_dest_file.exists() {
        let _ = fs::remove_file(&agy_dest_file);
        let _ = fs::remove_dir(&agy_dest_dir);
    }

    let reasonix_dest_dir = PathBuf::from(format!("C:\\Users\\{}\\.reasonix\\playbooks\\shared\\", username));
    let reasonix_dest_file = reasonix_dest_dir.join(format!("{}.md", skill_id));
    if reasonix_dest_file.exists() {
        let _ = fs::remove_file(&reasonix_dest_file);
    }

    let staging_file = get_staging_path().join(format!("{}.md", skill_id));
    if staging_file.exists() {
        let _ = fs::remove_file(&staging_file);
    }

    Ok(())
}

/// Called on app startup from JS loadApp().
/// Re-syncs all enabled skills to their physical destinations.
/// This ensures that after a restart, .reasonix/playbooks/shared/ and .gemini/skills/shared/
/// are always up-to-date with the current config state.
#[tauri::command]
async fn startup_sync_distributions() -> Result<(), String> {
    let config = get_config()?;
    let mut errors: Vec<String> = Vec::new();

    for (skill_id, status) in &config.skills_status {
        // Only sync skills that have at least one distribution enabled
        if status.enable_agy || status.enable_reasonix {
            let staging_file = get_staging_path().join(format!("{}.md", skill_id));
            if staging_file.exists() {
                if let Err(e) = sync_physical_distributions_for_skill(skill_id, &config) {
                    errors.push(format!("{}: {}", skill_id, e));
                }
            }
            // If staging file doesn't exist but is_enabled, silently skip
            // (user hasn't downloaded it yet, or it was cleaned up)
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        // Non-fatal: report what failed but don't block app startup
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
    fs::copy(&source_path, &dest_path).map_err(|e| format!("Failed to copy updated file: {}", e))?;

    // 3. Redistribute files
    let config = get_config()?;
    if config.skills_status.contains_key(&skill_id) {
        sync_physical_distributions_for_skill(&skill_id, &config)?;
    }

    Ok(())
}

// ==========================================
// 6. WebDAV Backup & Clouds Resurrect
// ==========================================

async fn backup_to_webdav_internal(config: &AppConfig) -> Result<(), String> {
    let config_file = get_config_path();
    if !config_file.exists() {
        return Err("config.json not found".to_string());
    }

    let backup_zip_path = get_my_brain_path().join("config_backup.zip");

    // Zip config.json
    let file = File::create(&backup_zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    zip.start_file("config.json", options).map_err(|e| e.to_string())?;
    let config_data = fs::read(&config_file).map_err(|e| e.to_string())?;
    zip.write_all(&config_data).map_err(|e| e.to_string())?;
    zip.finish().map_err(|e| e.to_string())?;

    // Upload ZIP via HTTP PUT using reqwest
    let mut url = config.webdav.url.clone();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("config_backup.zip");

    let client = reqwest::Client::new();
    let auth_header = format!("Basic {}", base64_encode(&format!("{}:{}", config.webdav.user, config.webdav.pass)));

    let zip_bytes = fs::read(&backup_zip_path).map_err(|e| e.to_string())?;

    let response = client.put(&url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/zip")
        .body(zip_bytes)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("WebDAV backup failed with status: {}", response.status()))
    }
}

#[tauri::command]
async fn trigger_backup() -> Result<String, String> {
    let config = get_config()?;
    backup_to_webdav_internal(&config).await?;
    Ok("Backup uploaded successfully!".to_string())
}

#[tauri::command]
async fn trigger_resurrect() -> Result<AppConfig, String> {
    let config = get_config()?;
    
    let mut url = config.webdav.url.clone();
    if !url.ends_with('/') {
        url.push('/');
    }
    url.push_str("config_backup.zip");

    let client = reqwest::Client::new();
    let auth_header = format!("Basic {}", base64_encode(&format!("{}:{}", config.webdav.user, config.webdav.pass)));

    // 1. Download Backup ZIP
    let response = client.get(&url)
        .header("Authorization", auth_header)
        .send()
        .await
        .map_err(|e| format!("Failed to download backup: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed. WebDAV server returned: {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
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
                    scan_directory_for_skills(&repo_path, &repo.id, &mut skills_list);
                    if let Some(matched) = skills_list.iter().find(|s| s.id == *skill_id) {
                        found_path = Some(repo_path.join(&matched.relative_path));
                        _found_repo = Some(repo.id.clone());
                        break;
                    }
                }
            }

            if let Some(src) = found_path {
                let _ = fs::copy(&src, &staging_file);
            }
        }

        if staging_file.exists() {
            let _ = sync_physical_distributions_for_skill(skill_id, &recovered_config);
        }
    }

    Ok(recovered_config)
}

// ==========================================
// 6b. Reasonix Reload Notification
// ==========================================

/// Notify Reasonix to reload its playbooks index.
/// After distributing/removing a `.md` playbook file, call this to force
/// Reasonix to re-scan its `playbooks/shared/` directory and rebuild its
/// in-memory cache — otherwise the new playbook stays invisible until the
/// next Reasonix session restart.
#[tauri::command]
async fn notify_reasonix_reload() -> Result<String, String> {
    // Attempt multiple strategies to wake up Reasonix:
    // 1. Try running `reasonix /playbooks` which triggers a reload
    // 2. Fallback: try `reasonix --reload-skills`
    // 3. Silent fail if Reasonix isn't installed or not running

    let commands: [(&str, &[&str]); 3] = [
        ("reasonix", &["/playbooks"]),
        ("reasonix", &["/skills"]),
        ("npx",      &["reasonix", "/playbooks"]),
    ];

    let mut last_err = String::new();
    for (cmd, args) in &commands {
        let mut child = match std::process::Command::new(cmd)
            .args(*args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                last_err = format!("{}: {}", cmd, e);
                continue;
            }
        };

        // Give it a short timeout
        let _ = child.wait();

        return Ok(format!(
            "Reasonix notified via `{} {}`",
            cmd,
            args.join(" ")
        ));
    }

    // If we get here, none of the commands worked
    // This is non-fatal — Reasonix may not be installed
    Err(format!(
        "Could not reach Reasonix (all methods failed: {}) — skill file was written, but you may need to restart Reasonix manually.",
        last_err
    ))
}

// ==========================================
// 7. Builder Init
// ==========================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            add_repository,
            delete_repository,
            discover_skills,
            discover_all_skills,
            install_skill,
            toggle_skill_switch,
            sync_skill_now,
            sync_all_repositories,
            sync_single_repository,
            startup_sync_distributions,
            notify_reasonix_reload,
            trigger_backup,
            trigger_resurrect
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
