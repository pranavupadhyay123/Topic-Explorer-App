/* ═══════════════════════════════════════════════════════════════════════════
   Dashboard Page Logic
   ═══════════════════════════════════════════════════════════════════════════ */

let workspaces = [];
let topics = [];

document.addEventListener('DOMContentLoaded', () => {
  loadWorkspaces();
  loadTopics();

  // Enter key to explore
  document.getElementById('explore-input').addEventListener('keydown', (e) => {
    if (e.key === 'Enter') quickExplore();
  });
});

// ─── Load Workspaces ───────────────────────────────────────────────────────

async function loadWorkspaces() {
  const grid = document.getElementById('workspaces-grid');

  try {
    workspaces = await API.get('/api/workspaces');
    renderWorkspaces();
    updateWorkspaceFilter();
    updateTopicWorkspaceSelect();
  } catch (err) {
    showToast('Failed to load workspaces', 'error');
  }
}

function renderWorkspaces() {
  const grid = document.getElementById('workspaces-grid');

  if (workspaces.length === 0) {
    grid.innerHTML = `
      <div class="card card-clickable animate-in" onclick="openModal('create-workspace-modal')" style="border-style:dashed;">
        <div class="text-center">
          <div style="font-size:2rem;margin-bottom:8px;">✨</div>
          <div class="card-title">Create Your First Workspace</div>
          <div class="card-desc">Organize topics into focused learning areas</div>
        </div>
      </div>
    `;
    return;
  }

  grid.innerHTML = workspaces.map((ws, i) => `
    <div class="card workspace-card card-clickable animate-in" onclick="filterByWorkspace('${esc(ws.id)}')" style="animation-delay:${i * 0.05}s">
      <div class="workspace-color-bar" style="background:${esc(ws.color)}"></div>
      <div class="workspace-icon">${esc(ws.icon)}</div>
      <div class="card-title">${esc(ws.name)}</div>
      <div class="card-desc">${esc(ws.description) || 'No description'}</div>
      <div class="card-meta">
        <span>${timeAgo(ws.created_at)}</span>
        <button class="btn btn-ghost btn-sm" onclick="event.stopPropagation(); deleteWorkspace('${esc(ws.id)}')">🗑️</button>
      </div>
    </div>
  `).join('');
}

function updateWorkspaceFilter() {
  const select = document.getElementById('workspace-filter');
  select.innerHTML = '<option value="">All Workspaces</option>' +
    workspaces.map(ws => `<option value="${esc(ws.id)}">${esc(ws.icon)} ${esc(ws.name)}</option>`).join('');
}

function updateTopicWorkspaceSelect() {
  const select = document.getElementById('topic-workspace');
  if (!select) return;
  select.innerHTML = '<option value="">No Workspace</option>' +
    workspaces.map(ws => `<option value="${esc(ws.id)}">${esc(ws.icon)} ${esc(ws.name)}</option>`).join('');
}

// ─── Load Topics ───────────────────────────────────────────────────────────

async function loadTopics() {
  const grid = document.getElementById('topics-grid');
  const filter = document.getElementById('workspace-filter').value;

  try {
    const url = filter ? `/api/topics?workspace_id=${filter}` : '/api/topics';
    topics = await API.get(url);
    renderTopics();
  } catch (err) {
    showToast('Failed to load topics', 'error');
  }
}

function renderTopics() {
  const grid = document.getElementById('topics-grid');
  const subtitle = document.getElementById('topics-subtitle');

  subtitle.textContent = `${topics.length} topic${topics.length !== 1 ? 's' : ''}`;

  if (topics.length === 0) {
    showEmpty(grid, '🔍', 'No topics yet', 'Type a topic above and hit Explore to get started!');
    return;
  }

  grid.innerHTML = topics.map((t, i) => {
    const statusBadge = t.status === 'explored'
      ? '<span class="badge badge-success">✓ Explored</span>'
      : '<span class="badge badge-warning">Pending</span>';

    const wsInfo = t.workspace_name
      ? `<span style="color:${esc(t.workspace_color)}">${esc(t.workspace_icon)} ${esc(t.workspace_name)}</span>`
      : '';

    return `
      <div class="card card-clickable animate-in" onclick="openTopic('${esc(t.id)}')" style="animation-delay:${i * 0.05}s">
        <div class="flex justify-between items-center mb-sm">
          <div class="card-title">${esc(t.title)}</div>
          ${statusBadge}
        </div>
        <div class="card-desc">${esc(t.description) || 'No description'}</div>
        <div class="card-meta">
          ${wsInfo}
          <span>${timeAgo(t.created_at)}</span>
          <button class="btn btn-ghost btn-sm" onclick="event.stopPropagation(); deleteTopic('${esc(t.id)}')">🗑️</button>
        </div>
      </div>
    `;
  }).join('');
}

// ─── Actions ───────────────────────────────────────────────────────────────

async function quickExplore() {
  const input = document.getElementById('explore-input');
  const topic = input.value.trim();
  if (!topic) {
    showToast('Please enter a topic to explore', 'error');
    return;
  }

  const btn = document.getElementById('explore-btn');
  btn.disabled = true;
  btn.innerHTML = '<div class="spinner"></div> Creating...';

  try {
    const result = await API.post('/api/topics', { title: topic });
    showToast(`Topic "${topic}" created!`, 'success');
    input.value = '';
    window.location.href = `/explore.html?id=${result.id}&title=${encodeURIComponent(topic)}`;
  } catch (err) {
    showToast('Failed to create topic: ' + err.message, 'error');
  } finally {
    btn.disabled = false;
    btn.innerHTML = '🚀 Explore';
  }
}

async function createWorkspace() {
  const name = document.getElementById('ws-name').value.trim();
  if (!name) {
    showToast('Please enter a name', 'error');
    return;
  }

  try {
    await API.post('/api/workspaces', {
      name,
      description: document.getElementById('ws-desc').value.trim(),
      color: document.getElementById('ws-color').value,
      icon: document.getElementById('ws-icon').value,
    });
    showToast(`Workspace "${name}" created!`, 'success');
    closeModal('create-workspace-modal');
    document.getElementById('ws-name').value = '';
    document.getElementById('ws-desc').value = '';
    loadWorkspaces();
  } catch (err) {
    showToast('Failed to create workspace', 'error');
  }
}

async function createTopic() {
  const title = document.getElementById('topic-title').value.trim();
  if (!title) {
    showToast('Please enter a topic', 'error');
    return;
  }

  try {
    const result = await API.post('/api/topics', {
      title,
      workspace_id: document.getElementById('topic-workspace').value || null,
      description: document.getElementById('topic-desc').value.trim(),
    });
    showToast(`Topic "${title}" created!`, 'success');
    closeModal('create-topic-modal');
    window.location.href = `/explore.html?id=${result.id}&title=${encodeURIComponent(title)}`;
  } catch (err) {
    showToast('Failed to create topic', 'error');
  }
}

async function deleteTopic(id) {
  if (!await confirmAction('Delete this topic and all its data?')) return;
  try {
    await API.del(`/api/topics?id=${id}`);
    showToast('Topic deleted', 'success');
    loadTopics();
  } catch (err) {
    showToast('Failed to delete topic', 'error');
  }
}

async function deleteWorkspace(id) {
  if (!await confirmAction('Delete this workspace? Topics will be kept but unassigned.')) return;
  try {
    await API.del(`/api/workspaces?id=${id}`);
    showToast('Workspace deleted', 'success');
    loadWorkspaces();
    loadTopics();
  } catch (err) {
    showToast('Failed to delete workspace', 'error');
  }
}

function filterByWorkspace(id) {
  document.getElementById('workspace-filter').value = id;
  loadTopics();
}

function openTopic(id) {
  const topic = topics.find(t => t.id === id);
  window.location.href = `/explore.html?id=${id}&title=${encodeURIComponent(topic?.title || '')}`;
}
