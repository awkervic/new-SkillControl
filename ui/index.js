// ==========================================================================
// 1. Tauri Core Invoke API Initialization
// ==========================================================================

const invoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;

// Global Application Reactive State
const state = {
  config: null,
  skills: [],
  activeRepoId: 'all', // 'all', or specific 'repo_id'
  searchQuery: ''
};

// ==========================================================================
// 2. DOM Elements Selection
// ==========================================================================

const dom = {
  repoList: document.getElementById('repo-list'),
  skillsGrid: document.getElementById('skills-grid'),
  emptyState: document.getElementById('empty-state'),
  searchInput: document.getElementById('search-input'),
  themeToggle: document.getElementById('theme-toggle'),
  btnSyncAll: document.getElementById('btn-sync-all'),
  activeRepoTitle: document.getElementById('active-repo-title'),
  activeRepoDesc: document.getElementById('active-repo-desc'),
  statTotalSkills: document.getElementById('stat-total-skills'),
  statActiveAgy: document.getElementById('stat-active-agy'),
  statActiveReasonix: document.getElementById('stat-active-reasonix'),
  toast: document.getElementById('toast'),
  
  // Modals & Forms
  modalAddRepo: document.getElementById('modal-add-repo'),
  btnAddRepo: document.getElementById('btn-add-repo'),
  btnSaveRepo: document.getElementById('btn-save-repo'),
  newRepoName: document.getElementById('new-repo-name'),
  newRepoUrl: document.getElementById('new-repo-url'),
  
  modalSettings: document.getElementById('modal-settings'),
  btnSettings: document.getElementById('btn-settings'),
  btnSaveSettings: document.getElementById('btn-save-settings'),
  webdavUrl: document.getElementById('webdav-url'),
  webdavUser: document.getElementById('webdav-user'),
  webdavPass: document.getElementById('webdav-pass'),
  webdavAutoBackup: document.getElementById('webdav-auto-backup'),
  
  btnCloudBackup: document.getElementById('btn-cloud-backup'),
  btnCloudRestore: document.getElementById('btn-cloud-restore'),
  backupVersionSelect: document.getElementById('backup-version-select'),
  btnFetchBackups: document.getElementById('btn-fetch-backups'),
  btnCloudRestoreVersioned: document.getElementById('btn-cloud-restore-versioned')
};

// ==========================================================================
// 3. UI Notification Toasts
// ==========================================================================

function showToast(message, type = 'success') {
  dom.toast.textContent = message;
  dom.toast.className = 'toast-notification active';
  if (type === 'danger') {
    dom.toast.style.borderColor = 'var(--danger)';
  } else {
    dom.toast.style.borderColor = 'var(--primary)';
  }
  
  setTimeout(() => {
    dom.toast.className = 'toast-notification';
  }, 3500);
}

// ==========================================================================
// 4. Data Layer Loading & Backend syncs
// ==========================================================================

async function loadApp() {
  try {
    // === Phase 1: 本地配置读取 — 带 3 秒超时保护 ===
    // 如果 get_config 卡死，自动使用默认配置让界面先亮起来
    const configOrTimeout = await Promise.race([
      invoke('get_config'),
      new Promise((_, reject) => setTimeout(() => reject(new Error('get_config timeout after 3s')), 3000))
    ]);
    state.config = configOrTimeout;
    
    // Set persisted theme
    document.documentElement.setAttribute('data-theme', state.config.theme);
    
    // Fill WebDAV settings input
    dom.webdavUrl.value = state.config.webdav.url;
    dom.webdavUser.value = state.config.webdav.user;
    dom.webdavPass.value = state.config.webdav.pass;
    dom.webdavAutoBackup.checked = state.config.webdav.auto_backup_enabled;

    // === Phase 2: 立即渲染界面骨架（零等待亮起主看盘）===
    renderSidebar();
    renderStats();
    // 骨架屏：加载中状态
    dom.skillsGrid.innerHTML = `
      <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;gap:16px;padding:80px 0;width:100%;grid-column:1/-1;">
        <span style="display:inline-block;animation:spin 1s linear infinite;font-size:32px;">🔄</span>
        <span style="color:var(--text-secondary);font-size:13.5px;letter-spacing:0.5px;font-weight:500;">正在后台扫描技能清单...</span>
      </div>
    `;
    dom.skillsGrid.style.display = 'flex';
    dom.emptyState.style.display = 'none';

    // === Phase 3: 非阻塞后台拉取技能清单（不 await，不卡界面）===
    invoke('discover_all_skills')
      .then(skills => {
        state.skills = skills;
        renderStats();
        renderSkillsGrid();
        console.log('[SkillControl] 技能清单后台加载完成:', skills.length, '个技能');
      })
      .catch(err => {
        console.warn('[SkillControl] 技能后台拉取异常（非致命）:', err);
        // 即使网络失败也不让界面空白 — 保留骨架但显示提示
        dom.skillsGrid.innerHTML = `
          <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;padding:60px 0;width:100%;grid-column:1/-1;">
            <span style="font-size:40px;">📡</span>
            <span style="color:var(--text-secondary);font-size:13px;">技能清单加载失败，请点击「同步云端」重试</span>
          </div>
        `;
      });

    // === Phase 4: 开机后台同步物理分发（fire-and-forget，不阻塞任何东西）===
    invoke('startup_sync_distributions').catch(err => {
      console.warn('[SkillControl] Startup sync partial error (non-fatal):', err);
    });
  } catch (error) {
    console.error('[SkillControl] 初始化失败:', error);
    // 即使初始化失败也显示基本骨架，不让界面完全白屏
    dom.skillsGrid.innerHTML = `
      <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;padding:60px 0;width:100%;grid-column:1/-1;">
        <span style="font-size:40px;">⚠️</span>
        <span style="color:var(--text-secondary);font-size:13px;">初始化异常: ${error}，请重启软件</span>
      </div>
    `;
    dom.skillsGrid.style.display = 'flex';
    dom.emptyState.style.display = 'none';
  }
}

async function refreshSkillsList() {
  try {
    state.skills = await invoke('discover_all_skills');
    renderStats();
    renderSkillsGrid();
  } catch (error) {
    showToast(`拉取技能清单失败: ${error}`, 'danger');
  }
}

// ==========================================================================
// 5. Render Templates
// ==========================================================================

let currentRequestId = 0;
let debounceTimeout = null;

function selectRepository(repoId, repoName, repoDesc) {
  state.activeRepoId = repoId;
  dom.activeRepoTitle.textContent = repoName;
  dom.activeRepoDesc.textContent = repoDesc;
  renderSidebar();

  if (debounceTimeout) {
    clearTimeout(debounceTimeout);
  }

  debounceTimeout = setTimeout(async () => {
    const requestId = ++currentRequestId;
    
    // Show premium visual loading state inside skills grid
    dom.skillsGrid.innerHTML = `
      <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;gap:16px;padding:80px 0;width:100%;grid-column:1/-1;">
        <span style="display:inline-block;animation:spin 1s linear infinite;font-size:32px;">🔄</span>
        <span style="color:var(--text-secondary);font-size:13.5px;letter-spacing:0.5px;font-weight:500;">正在为您全速扫描磁盘中...</span>
      </div>
    `;
    dom.skillsGrid.style.display = 'flex';
    dom.emptyState.style.display = 'none';

    try {
      let skills = [];
      if (repoId === 'all' || repoId === 'downloaded') {
        skills = await invoke('discover_all_skills');
      } else {
        skills = await invoke('discover_skills', { repoId });
      }

      if (requestId !== currentRequestId) {
        console.log(`[Request ID Override] Ignored stale request ID: ${requestId} (current: ${currentRequestId})`);
        return;
      }

      state.skills = skills;
      renderStats();
      renderSkillsGrid();
    } catch (error) {
      if (requestId === currentRequestId) {
        showToast(`扫描目录失败: ${error}`, 'danger');
        dom.skillsGrid.style.display = 'none';
        dom.emptyState.style.display = 'flex';
      }
    }
  }, 150);
}

function renderSidebar() {
  dom.repoList.innerHTML = '';

  // "All Repositories" Tab
  const allItem = document.createElement('li');
  allItem.className = `repo-item ${state.activeRepoId === 'all' ? 'active' : ''}`;
  allItem.innerHTML = `
    <div class="repo-info-row">
      <span>📦</span>
      <span class="repo-name-text">全部技能列表</span>
    </div>
  `;
  allItem.addEventListener('click', () => {
    selectRepository('all', '全部技能', '正在查看所有绑定的技能仓库中的 AI 技能卡片。支持热更新与物理分发。');
  });
  dom.repoList.appendChild(allItem);

  // "Downloaded Skills" Tab
  const downloadedItem = document.createElement('li');
  downloadedItem.className = `repo-item ${state.activeRepoId === 'downloaded' ? 'active' : ''}`;
  downloadedItem.innerHTML = `
    <div class="repo-info-row">
      <span>📥</span>
      <span class="repo-name-text">已下载技能</span>
    </div>
  `;
  downloadedItem.addEventListener('click', () => {
    selectRepository('downloaded', '已下载技能', '正在查看所有已成功下载并安装到本地 Staging 暂存池的 AI 技能。');
  });
  dom.repoList.appendChild(downloadedItem);

  // Individual Repositories
  state.config.repositories.forEach(repo => {
    const item = document.createElement('li');
    item.className = `repo-item ${state.activeRepoId === repo.id ? 'active' : ''}`;
    
    item.innerHTML = `
      <div class="repo-info-row" style="flex: 1;">
        <span>📂</span>
        <span class="repo-name-text" title="${repo.name}">${repo.name}</span>
      </div>
      ${!repo.is_official ? `
        <button class="btn-delete-repo" title="删除仓库并解除其技能">
          <svg class="icon" viewBox="0 0 24 24"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round"/></svg>
        </button>
      ` : ''}
    `;

    // Click to select Repository Filter
    item.addEventListener('click', (e) => {
      if (e.target.closest('.btn-delete-repo')) {
        handleDeleteRepo(repo.id, repo.name);
        return;
      }
      selectRepository(repo.id, repo.name, `仓库链接: ${repo.url}。点击“Sync Now”或“同步云端”更新本地物理镜像。`);
    });

    dom.repoList.appendChild(item);
  });
}

function renderStats() {
  dom.statTotalSkills.textContent = state.skills.length;
  
  let agyCount = 0;
  let reasonixCount = 0;
  
  Object.values(state.config.skills_status).forEach(status => {
    if (status.enable_agy) agyCount++;
    if (status.enable_reasonix) reasonixCount++;
  });
  
  dom.statActiveAgy.textContent = agyCount;
  dom.statActiveReasonix.textContent = reasonixCount;
}

function renderSkillsGrid() {
  dom.skillsGrid.innerHTML = '';
  
  // 1. Filter by Active Repository
  let filtered = state.skills;
  if (state.activeRepoId === 'downloaded') {
    filtered = filtered.filter(s => s.is_downloaded === true);
  } else if (state.activeRepoId !== 'all') {
    filtered = filtered.filter(s => s.repo_id === state.activeRepoId);
  }
  
  // 2. Filter by Fuzzy Search Query
  if (state.searchQuery) {
    const q = state.searchQuery.toLowerCase();
    filtered = filtered.filter(s => 
      s.name.toLowerCase().includes(q) || 
      s.id.toLowerCase().includes(q) || 
      s.description.toLowerCase().includes(q)
    );
  }

  if (filtered.length === 0) {
    dom.skillsGrid.style.display = 'none';
    dom.emptyState.style.display = 'flex';
    return;
  }

  dom.skillsGrid.style.display = 'grid';
  dom.emptyState.style.display = 'none';

  filtered.forEach(skill => {
    const card = document.createElement('div');
    card.className = 'skill-card';

    // Retrieve active switches status from config
    const status = state.config.skills_status[skill.id] || {
      repo_id: skill.repo_id,
      scope: 'global',
      enable_agy: false,
      enable_reasonix: false,
      auto_update: true
    };

    const repoName = state.config.repositories.find(r => r.id === skill.repo_id)?.name || '未知仓库';

    card.innerHTML = `
      <div class="skill-header">
        <div class="skill-title-area">
          <h3 class="skill-title">${skill.name}</h3>
          <span class="skill-repo-badge">📦 隶属仓库: ${repoName}</span>
        </div>
        ${state.activeRepoId === 'downloaded' ? `
          <button class="btn-crush-skill" data-skill="${skill.id}" title="彻底物理粉碎并抹除此技能">
            🗑️ 彻底删除
          </button>
        ` : ''}
      </div>
      <p class="skill-desc">${skill.description || '暂无对该智能体技能的详细描述文本。'}</p>
      
      <!-- Core Switch Row (Ironclad controls) -->
      <div class="skill-switches">
        <div class="control-switch-row">
          <div class="switch-label-group">
            <span class="switch-label-name label-agy">⚡ AGY 技能分发</span>
            <span class="switch-label-desc">在 .gemini 隐藏沙箱创建物理 SKILL.md</span>
          </div>
          <label class="switch-container">
            <input type="checkbox" class="toggle-switch" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-type="agy" ${status.enable_agy ? 'checked' : ''} ${!skill.is_installed ? 'disabled' : ''}>
            <span class="switch-slider"></span>
          </label>
        </div>

        <div class="control-switch-row">
          <div class="switch-label-group">
            <span class="switch-label-name label-reasonix">🌀 Reasonix 播放剧本</span>
            <span class="switch-label-desc">在 .reasonix 隐藏剧本池映射独立剧本</span>
          </div>
          <label class="switch-container">
            <input type="checkbox" class="toggle-switch" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-type="reasonix" ${status.enable_reasonix ? 'checked' : ''} ${!skill.is_installed ? 'disabled' : ''}>
            <span class="switch-slider"></span>
          </label>
        </div>

        <div class="control-switch-row">
          <div class="switch-label-group">
            <span class="switch-label-name label-auto-update">🔄 开机自动同步更新</span>
            <span class="switch-label-desc">系统启动时自动拉取最新 Git 并重新分发</span>
          </div>
          <label class="switch-container">
            <input type="checkbox" class="toggle-switch" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-type="auto_update" ${status.auto_update ? 'checked' : ''}>
            <span class="switch-slider"></span>
          </label>
        </div>
      </div>

      <!-- Scope Selector (Global / Project / Shared) -->
      <div class="scope-selector-row">
        <div class="switch-label-group">
          <span class="switch-label-name" style="color:var(--text-secondary);font-size:11px;">📡 分发范围</span>
          <span class="switch-label-desc">Global（全局级）| Project（项目级）| Shared（共享级）</span>
        </div>
        <div class="scope-toggle-group">
          <button class="btn-scope btn-scope-global ${status.scope === 'global' ? 'active' : ''}" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-scope="global">Global</button>
          <button class="btn-scope btn-scope-project ${status.scope === 'project' ? 'active' : ''}" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-scope="project">Project</button>
          <button class="btn-scope btn-scope-shared ${status.scope === 'shared' ? 'active' : ''}" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-scope="shared">Shared</button>
        </div>
      </div>

      <!-- Action Card Buttons Footer -->
      <div class="skill-card-footer">
        ${!skill.is_installed ? `
          <button class="btn-card-action btn-cloud-download" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-path="${skill.relative_path}">
            📥 下载并安装到本地
          </button>
        ` : `
          <button class="btn-card-action btn-sync-now" data-skill="${skill.id}" data-repo="${skill.repo_id}" data-path="${skill.relative_path}">
            🔄 Sync Now
          </button>
        `}
      </div>
    `;

    // Listen to changes on switches
    card.querySelectorAll('.toggle-switch').forEach(sw => {
      sw.addEventListener('change', async (e) => {
        const skillId = e.target.dataset.skill;
        const repoId = e.target.dataset.repo;
        const type = e.target.dataset.type;
        const checked = e.target.checked;
        
        try {
          // Read current scope from the status in config
          const scope = (state.config.skills_status[skillId] || {}).scope || 'global';
          state.config = await invoke('toggle_skill_switch', {
            skillId,
            repoId,
            switchType: type,
            status: checked,
            scope: scope
          });
          renderStats();
          showToast(`技能 [${skillId}] 已${checked ? '点亮启动' : '注销关闭'} [${type.toUpperCase()}]`);
          
          // After any AGY/Reasonix toggle, notify Reasonix to reload its playbook index
          // and update AGY's config.json installed_skills registry
          if (type === 'reasonix' || type === 'agy') {
            invoke('notify_reasonix_reload').catch(err => {
              console.warn('[SkillControl] Reasonix reload note (non-blocking):', err);
            });
          }
        } catch (error) {
          e.target.checked = !checked; // revert
          showToast(`开关操作失败: ${error}`, 'danger');
        }
      });
    });

    // Scope selector click
    card.querySelectorAll('.btn-scope').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        const target = e.currentTarget;
        const skillId = target.dataset.skill;
        const repoId = target.dataset.repo;
        const scope = target.dataset.scope;
        
        try {
          state.config = await invoke('update_skill_scope', {
            skillId, repoId, scope
          });
          // Update active visual
          card.querySelectorAll('.btn-scope').forEach(b => b.classList.remove('active'));
          target.classList.add('active');
          showToast(`技能 [${skillId}] 分发范围切换为 [${scope.toUpperCase()}]`);
        } catch (error) {
          showToast(`范围切换失败: ${error}`, 'danger');
        }
      });
    });

    // Download/Install click
    const btnDownload = card.querySelector('.btn-cloud-download');
    if (btnDownload) {
      btnDownload.addEventListener('click', async (e) => {
        const btn = e.currentTarget;
        const skillId = btn.dataset.skill;
        const repoId = btn.dataset.repo;
        const path = btn.dataset.path;
        
        btn.disabled = true;
        btn.textContent = '📥 正在拉取中...';
        
        try {
          await invoke('install_skill', { repoId, relativePath: path, skillId });
          showToast(`技能拉取至本地暂存池成功！`);
          await refreshSkillsList();
        } catch (error) {
          btn.disabled = false;
          btn.textContent = '📥 下载并安装到本地';
          showToast(`拉取失败: ${error}`, 'danger');
        }
      });
    }

    // Sync Now Click
    const btnSync = card.querySelector('.btn-sync-now');
    if (btnSync) {
      btnSync.addEventListener('click', async (e) => {
        const btn = e.currentTarget;
        const skillId = btn.dataset.skill;
        const repoId = btn.dataset.repo;
        const path = btn.dataset.path;
        
        btn.disabled = true;
        btn.innerHTML = `<span style="display:inline-block;animation:spin 1s linear infinite">🔄</span> 正在同步...`;
        
        try {
          await invoke('sync_skill_now', { skillId, repoId, relativePath: path });
          showToast(`技能 [${skillId}] 手动物理重组分发同步完成！`);
          
          // Notify Reasonix to reload playbooks after sync
          invoke('notify_reasonix_reload').catch(err => {
            console.warn('[SkillControl] Reasonix reload note (non-blocking):', err);
          });
          
          await refreshSkillsList();
        } catch (error) {
          btn.disabled = false;
          btn.innerHTML = `🔄 Sync Now`;
          showToast(`同步失败: ${error}`, 'danger');
        }
      });
    }

    // Crush/Uninstall Skill Click
    const btnCrush = card.querySelector('.btn-crush-skill');
    if (btnCrush) {
      btnCrush.addEventListener('click', async (e) => {
        const btn = e.currentTarget;
        const skillId = btn.dataset.skill;
        if (confirm(`【铁血物理粉碎警告】\n您确认要彻底粉碎并删除技能 [${skillId}] 吗？\n这将直接物理删除暂存池（my-brain 及 staging 目录下的源文件）、AGY/Reasonix 对应的所有物理分发，并彻底从本系统的 config.json 账本中注销！`)) {
          btn.disabled = true;
          btn.textContent = '🗑️ 正在粉碎...';
          
          try {
            state.config = await invoke('uninstall_skill', { skillId });
            showToast(`技能 [${skillId}] 已被彻底粉碎，从系统里完全灰飞烟灭！`);
            
            // Notify Reasonix to reload playbooks after deletion
            invoke('notify_reasonix_reload').catch(err => {
              console.warn('[SkillControl] Reasonix reload note:', err);
            });
            
            await refreshSkillsList();
          } catch (error) {
            btn.disabled = false;
            btn.textContent = '🗑️ 彻底删除';
            showToast(`粉碎删除失败: ${error}`, 'danger');
          }
        }
      });
    }

    dom.skillsGrid.appendChild(card);
  });
}

// ==========================================
// 6. Action Handlers (Modal popups & saves)
// ==========================================

async function handleDeleteRepo(repoId, repoName) {
  if (confirm(`警告：您确认要物理删除仓库 [${repoName}] 吗？\n删除将注销其下所有的技能启用状态并移除本地克隆缓存！`)) {
    try {
      state.config = await invoke('delete_repository', { repoId });
      showToast(`仓库 [${repoName}] 已删除，关联的本地物理分发已下线！`);
      state.activeRepoId = 'all';
      await loadApp();
    } catch (error) {
      showToast(`删除仓库失败: ${error}`, 'danger');
    }
  }
}

// ==========================================
// 7. Event Listeners Setup
// ==========================================

// Global sync buttons
dom.btnSyncAll.addEventListener('click', async () => {
  dom.btnSyncAll.disabled = true;
  dom.btnSyncAll.innerHTML = `<svg class="icon animate-spin-hover" viewBox="0 0 24 24" style="animation:spin 1.5s linear infinite"><path d="M20 4v5h-5M4 20v-5h5M4 12a8 8 0 0114.93-4.07L20 9m-16 6l1.07 1.07A8 8 0 0020 12" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round"/></svg> 正在全面克隆拉取...`;
  
  try {
    await invoke('sync_all_repositories');
    showToast('所有 Git 技能仓库本地缓存拉取刷新完毕！');
    await refreshSkillsList();
  } catch (error) {
    showToast(`云端全面拉取异常: ${error}`, 'danger');
  } finally {
    dom.btnSyncAll.disabled = false;
    dom.btnSyncAll.innerHTML = `<svg class="icon animate-spin-hover" viewBox="0 0 24 24"><path d="M20 4v5h-5M4 20v-5h5M4 12a8 8 0 0114.93-4.07L20 9m-16 6l1.07 1.07A8 8 0 0020 12" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round"/></svg> 同步云端`;
  }
});

// Search input keyup
dom.searchInput.addEventListener('input', (e) => {
  state.searchQuery = e.target.value.trim();
  renderSkillsGrid();
});

// Theme switcher
dom.themeToggle.addEventListener('click', async () => {
  const currentTheme = document.documentElement.getAttribute('data-theme');
  const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
  
  document.documentElement.setAttribute('data-theme', newTheme);
  state.config.theme = newTheme;
  
  try {
    await invoke('save_config', { config: state.config });
  } catch (error) {
    console.error('Failed to save theme state', error);
  }
});

// --- Modal Add Repo Controllers ---
dom.btnAddRepo.addEventListener('click', () => {
  dom.newRepoName.value = '';
  dom.newRepoUrl.value = '';
  dom.modalAddRepo.classList.add('active');
});

dom.modalAddRepo.querySelectorAll('.btn-close-modal, .btn-cancel').forEach(btn => {
  btn.addEventListener('click', () => dom.modalAddRepo.classList.remove('active'));
});

dom.btnSaveRepo.addEventListener('click', async () => {
  const name = dom.newRepoName.value.trim();
  const url = dom.newRepoUrl.value.trim();
  
  if (!name || !url) {
    showToast('名字与 Git 链接均为必填项！', 'danger');
    return;
  }
  
  try {
    state.config = await invoke('add_repository', { name, url });
    dom.modalAddRepo.classList.remove('active');
    showToast(`自定义技能仓库 [${name}] 已成功绑定！正在后台触发 Git clone 克隆。`);
    await loadApp();
  } catch (error) {
    showToast(`绑定失败: ${error}`, 'danger');
  }
});

// --- Modal Settings & Backup Controllers ---
dom.btnSettings.addEventListener('click', () => {
  dom.modalSettings.classList.add('active');
  // Non-blocking background fetch — settings panel opens instantly
  dom.backupVersionSelect.innerHTML = '<option value="">正在获取云端列表...</option>';
  setTimeout(() => fetchBackupList(), 100); // 100ms delay so the modal renders first
});

dom.modalSettings.querySelector('.btn-close-modal').addEventListener('click', () => {
  dom.modalSettings.classList.remove('active');
});

dom.btnSaveSettings.addEventListener('click', async () => {
  state.config.webdav.url = dom.webdavUrl.value.trim();
  state.config.webdav.user = dom.webdavUser.value.trim();
  state.config.webdav.pass = dom.webdavPass.value.trim();
  state.config.webdav.auto_backup_enabled = dom.webdavAutoBackup.checked;
  
  try {
    await invoke('save_config', { config: state.config });
    dom.modalSettings.classList.remove('active');
    showToast('WebDAV 远程账户与全局备份偏好配置保存成功！');
  } catch (error) {
    showToast(`配置保存异常: ${error}`, 'danger');
  }
});

// Fetch Backups from WebDAV — 非阻塞拉取，绝不卡界面
async function fetchBackupList() {
  dom.btnFetchBackups.disabled = true;
  dom.btnFetchBackups.textContent = '🔄 正在获取...';
  dom.backupVersionSelect.innerHTML = '<option value="">正在获取云端列表...</option>';
  
  try {
    // 内置 15s 超时保护（由后端 create_webdav_client 保证）
    const list = await invoke('get_backup_list');
    dom.backupVersionSelect.innerHTML = '';
    
    if (list.length === 0) {
      dom.backupVersionSelect.innerHTML = '<option value="">云端无历史备份</option>';
      dom.btnCloudRestoreVersioned.disabled = true;
    } else {
      list.forEach(item => {
        const option = document.createElement('option');
        option.value = item.filename;
        option.textContent = item.display;
        dom.backupVersionSelect.appendChild(option);
      });
      dom.btnCloudRestoreVersioned.disabled = false;
    }
  } catch (error) {
    dom.backupVersionSelect.innerHTML = '<option value="">⚠️ 读取失败（15s超时），请检查 WebDAV 配置</option>';
    console.warn('[SkillControl] WebDAV fetch failed (non-fatal):', error);
  } finally {
    dom.btnFetchBackups.disabled = false;
    dom.btnFetchBackups.textContent = '🔍 刷新列表';
  }
}

dom.btnFetchBackups.addEventListener('click', fetchBackupList);

dom.backupVersionSelect.addEventListener('change', () => {
  dom.btnCloudRestoreVersioned.disabled = !dom.backupVersionSelect.value;
});

// WebDAV Cloud Backup Trigger
dom.btnCloudBackup.addEventListener('click', async () => {
  dom.btnCloudBackup.disabled = true;
  dom.btnCloudBackup.textContent = '⚡ 正在打包压缩打包并静默上传中...';
  
  try {
    const res = await invoke('trigger_backup');
    showToast(res);
    // Refresh the list immediately after successful backup
    await fetchBackupList();
  } catch (error) {
    showToast(`备份失败: ${error}`, 'danger');
  } finally {
    dom.btnCloudBackup.disabled = false;
    dom.btnCloudBackup.textContent = '立刻压缩备份到云端';
  }
});

// WebDAV Cloud Restore & Resurrect Trigger
dom.btnCloudRestore.addEventListener('click', async () => {
  if (confirm('警告：一键云端复活会强制下载并还原您云端的 `config_backup.zip`！\n这将覆盖本地当前的 `config.json` 设置，并全自动为您克隆缺失的 Git 仓库并物理重组分发文件。确认复活吗？')) {
    dom.btnCloudRestore.disabled = true;
    dom.btnCloudRestore.textContent = '🌀 正在拉取备份包并全自动分发克隆中...';
    
    try {
      state.config = await invoke('trigger_resurrect');
      showToast('🎉 云端工作流已在新电脑上完美复活！所有 Git 技能树已重新生成并完成物理组装分发！');
      dom.modalSettings.classList.remove('active');
      await loadApp();
    } catch (error) {
      showToast(`复活失败: ${error}`, 'danger');
    } finally {
      dom.btnCloudRestore.disabled = false;
      dom.btnCloudRestore.textContent = '从云端拉取一键复活工作流';
    }
  }
});

// WebDAV Versioned Time Machine Restore Trigger
dom.btnCloudRestoreVersioned.addEventListener('click', async () => {
  const filename = dom.backupVersionSelect.value;
  if (!filename) return;
  
  const text = dom.backupVersionSelect.options[dom.backupVersionSelect.selectedIndex].text;
  if (confirm(`警告：确认恢复到选中的历史版本：[${text}] 吗？\n这将清空当前的 AppData 数据目录并全量拉取还原，随后自动重新组装和克隆所有 Git 技能仓库！`)) {
    dom.btnCloudRestoreVersioned.disabled = true;
    dom.backupVersionSelect.disabled = true;
    dom.btnFetchBackups.disabled = true;
    dom.btnCloudRestoreVersioned.textContent = '🌀 Time Machine 正在定点复活并拉取中...';
    
    try {
      state.config = await invoke('trigger_restore_version', { filename });
      showToast(`🎉 成功！Time Machine 备份版本 [${text}] 已经在本地完美定点复活还原！`);
      dom.modalSettings.classList.remove('active');
      await loadApp();
    } catch (error) {
      showToast(`还原失败: ${error}`, 'danger');
    } finally {
      dom.btnCloudRestoreVersioned.disabled = false;
      dom.backupVersionSelect.disabled = false;
      dom.btnFetchBackups.disabled = false;
      dom.btnCloudRestoreVersioned.textContent = '确认定向还原选中版本';
    }
  }
});

// Start the application
document.addEventListener('DOMContentLoaded', loadApp);
