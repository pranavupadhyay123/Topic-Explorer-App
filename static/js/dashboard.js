/* ═══════════════════════════════════════════════════════════════════════════
   Topic Explorer — Dashboard & Sidebar Data Management
   ═══════════════════════════════════════════════════════════════════════════ */

let workspaces = [];
let topics = [];
let currentWorkspaceId = null;

const emojiToLucide = {
  '🧠': 'brain', '💻': 'monitor', '📚': 'book', '🚀': 'rocket', '⚙️': 'settings', 
  '🧪': 'flask-conical', '🔬': 'microscope', '🌍': 'globe', '⚛️': 'atom', '🏛️': 'landmark',
  'W': 'folder'
};

function renderWsIcon(icon) {
  if (!icon) return '<i data-lucide="folder"></i>';
  if (emojiToLucide[icon]) return `<i data-lucide="${emojiToLucide[icon]}"></i>`;
  if (/^[a-zA-Z-]+$/.test(icon)) return `<i data-lucide="${icon}"></i>`;
  return '<i data-lucide="folder"></i>';
}


document.addEventListener('DOMContentLoaded', () => {
  // Try to load last selected workspace from localStorage
  const savedWsId = localStorage.getItem('last_workspace_id');
  if (savedWsId) {
    currentWorkspaceId = savedWsId;
  }
  
  loadWorkspaces();
  navigate('dashboard'); // Initial view
});

// ─── Workspaces ────────────────────────────────────────────────────────────

async function loadWorkspaces() {
  try {
    workspaces = await API.get('/api/workspaces');
    
    // Automatically select first workspace if none selected
    if (!currentWorkspaceId && workspaces.length > 0) {
      currentWorkspaceId = workspaces[0].id;
    }
    
    renderWorkspacesGrid();
    renderSidebarWorkspaceInfo();
  if(typeof refreshIcons==='function') refreshIcons();
    renderWorkspaceSelector();
    
    // Load topics for current workspace
    loadTopics();
  } catch (err) {
    showToast('Failed to load workspaces', 'error');
  }
}

function renderWorkspacesGrid() {
  const grid = document.getElementById('workspaces-grid');
  if (!grid) return;

  if (workspaces.length === 0) {
    grid.innerHTML = `
      <div class="card workspace-card text-center" onclick="openModal('create-workspace-modal')" style="border-style:dashed;">
        <div style="margin-bottom:8px; display:flex; justify-content:center;"><i data-lucide="folder-plus" style="width:32px;height:32px;"></i></div>
        <div class="card-title">Create Your First Workspace</div>
        <div class="card-desc">Organize topics into focused learning areas</div>
      </div>
    `;
    return;
  }

  grid.innerHTML = workspaces.map((ws, i) => `
    <div class="card workspace-card" onclick="selectWorkspace('${esc(ws.id)}')" style="animation-delay:${i * 0.05}s; ${ws.id === currentWorkspaceId ? 'border-color: var(--primary);' : ''}">
      <div style="margin-bottom: var(--space-sm); display:flex;">${renderWsIcon(ws.icon)}</div>
      <div class="card-title">${esc(ws.name)}</div>
      <div class="card-desc">${esc(ws.description) || 'No description'}</div>
      <div class="flex justify-between items-center mt-md">
        <span class="text-xs text-muted">${timeAgo(ws.created_at)}</span>
        <button class="btn btn-ghost btn-sm" onclick="event.stopPropagation(); deleteWorkspace('${esc(ws.id)}')"><i data-lucide="trash-2" style="width:16px;height:16px;"></i></button>
      </div>
    </div>
  `).join('');
}

function renderSidebarWorkspaceInfo() {
  const ws = workspaces.find(w => w.id === currentWorkspaceId);
  const nameEl = document.getElementById('sidebar-workspace-name');
  const avatarEl = document.getElementById('current-workspace-avatar');
  
  if (ws) {
    nameEl.innerHTML = `${esc(ws.name)} <span class="text-xs text-muted">▼</span>`;
    avatarEl.innerHTML = renderWsIcon(ws.icon);
    avatarEl.style.background = ws.color || 'var(--primary)';
  } else {
    nameEl.innerHTML = `Select Workspace <span class="text-xs text-muted">▼</span>`;
    avatarEl.innerHTML = renderWsIcon('W');
    avatarEl.style.background = 'var(--bg-elevated)';
  }
}

function handleWorkspaceClick(e) {
  const sidebar = document.getElementById('app-sidebar');
  if (sidebar && sidebar.classList.contains('collapsed') && window.innerWidth > 991) {
    if (typeof toggleSidebar === 'function') toggleSidebar(e);
  } else {
    openWorkspaceSelector();
  }
}

function openWorkspaceSelector() {
  openModal('ws-selector-modal');
}

function renderWorkspaceSelector() {
  const list = document.getElementById('ws-selector-list');
  if (!list) return;
  
  list.innerHTML = workspaces.map(ws => `
    <div class="ws-item" onclick="selectWorkspace('${esc(ws.id)}'); closeModal('ws-selector-modal')">
      <span style="display:flex;align-items:center;">${renderWsIcon(ws.icon)}</span>
      <span>${esc(ws.name)}</span>
      ${ws.id === currentWorkspaceId ? '<span style="margin-left:auto; color:var(--primary); display:flex;"><i data-lucide="check" style="width:16px;height:16px;"></i></span>' : ''}
    </div>
  `).join('');
}

function selectWorkspace(id) {
  currentWorkspaceId = id;
  localStorage.setItem('last_workspace_id', id);
  
  renderSidebarWorkspaceInfo();
  if(typeof refreshIcons==='function') refreshIcons();
  renderWorkspacesGrid();
  renderWorkspaceSelector();
  
  loadTopics();
}

async function createWorkspace() {
  const name = document.getElementById('ws-name').value.trim();
  if (!name) {
    showToast('Please enter a name', 'error');
    return;
  }

  try {
    const ws = await API.post('/api/workspaces', {
      name,
      description: document.getElementById('ws-desc').value.trim(),
      color: document.getElementById('ws-color').value,
      icon: document.getElementById('ws-icon').value,
    });
    
    showToast(`Workspace "${name}" created!`, 'success');
    closeModal('create-workspace-modal');
    
    // Clear inputs
    document.getElementById('ws-name').value = '';
    document.getElementById('ws-desc').value = '';
    
    selectWorkspace(ws.id);
    loadWorkspaces();
  } catch (err) {
    showToast('Failed to create workspace', 'error');
  }
}

async function deleteWorkspace(id) {
  if (!window.confirm('Delete this workspace?')) return;
  try {
    await API.del(`/api/workspaces?id=${id}`);
    showToast('Workspace deleted', 'success');
    if (currentWorkspaceId === id) {
      currentWorkspaceId = null;
      localStorage.removeItem('last_workspace_id');
    }
    loadWorkspaces();
  } catch (err) {
    showToast('Failed to delete workspace', 'error');
  }
}

// ─── Topics ────────────────────────────────────────────────────────────────

async function loadTopics() {
  try {
    const url = currentWorkspaceId ? `/api/topics?workspace_id=${currentWorkspaceId}` : '/api/topics';
    topics = await API.get(url);
    renderSidebarTopics();
  } catch (err) {
    showToast('Failed to load topics', 'error');
  }
}

function renderSidebarTopics() {
  const list = document.getElementById('sidebar-topics-list');
  if (!list) return;

  if (topics.length === 0) {
    list.innerHTML = `<div class="text-xs text-muted text-center mt-md p-md">No topics yet.<br>Explore something to begin!</div>`;
    return;
  }

  list.innerHTML = topics.map(t => `
    <div class="topic-item" onclick="openTopic('${esc(t.id)}')">
      <span class="topic-icon"><i data-lucide="file-text" style="width:16px;height:16px;"></i></span>
      <span class="topic-title-sidebar">${esc(t.title)}</span>
    </div>
  `).join('');
}

async function quickExploreLanding() {
  const input = document.getElementById('landing-explore-input');
  const topic = input.value.trim();
  if (!topic) return;
  
  await createAndExploreTopic(topic);
}

function exploreSuggestion(topic) {
  createAndExploreTopic(topic);
}

async function createAndExploreTopic(title) {
  try {
    const result = await API.post('/api/topics', { 
      title: title,
      workspace_id: currentWorkspaceId || null
    });
    
    showToast(`Topic created!`, 'success');
    document.getElementById('landing-explore-input').value = '';
    
    // Reload topics to update sidebar
    await loadTopics();
    
    // Navigate to explore view
    navigate('explore', { topicId: result.id });
    
  } catch (err) {
    showToast('Failed to create topic: ' + err.message, 'error');
  }
}

function openTopic(id) {
  navigate('explore', { topicId: id });
}

async function deleteCurrentTopic() {
  const topicId = window.currentTopicData?.id;
  if (!topicId) return;
  
  if (!window.confirm('Delete this topic?')) return;
  
  try {
    await API.del(`/api/topics?id=${topicId}`);
    showToast('Topic deleted', 'success');
    navigate('dashboard');
    loadTopics();
  } catch (err) {
    showToast('Failed to delete topic', 'error');
  }
}
