/* ═══════════════════════════════════════════════════════════════════════════
   Topic Explorer — Core SPA Logic
   ═══════════════════════════════════════════════════════════════════════════ */

const API = {
  async get(url) {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`GET ${url} failed: ${res.status}`);
    return res.json();
  },
  async post(url, data) {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new Error(err.error || `POST ${url} failed: ${res.status}`);
    }
    return res.json();
  },
  async put(url, data) {
    const res = await fetch(url, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!res.ok) throw new Error(`PUT ${url} failed: ${res.status}`);
    return res.json();
  },
  async del(url) {
    const res = await fetch(url, { method: 'DELETE' });
    if (!res.ok) throw new Error(`DELETE ${url} failed: ${res.status}`);
    return res.json();
  },
};

// ─── SPA Router ────────────────────────────────────────────────────────────

let currentView = 'dashboard';

function navigate(viewName, params = {}) {
  // Hide all views
  document.querySelectorAll('.view-container').forEach(v => v.classList.remove('active'));
  
  // Update sidebar active state
  document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
  document.querySelectorAll('.topic-item').forEach(n => n.classList.remove('active'));

  // Close sidebar on mobile
  if (typeof closeSidebar === 'function' && window.innerWidth <= 991) {
    closeSidebar();
  }

  // Show target view
  const targetView = document.getElementById(`view-${viewName}`);
  if (targetView) targetView.classList.add('active');

  currentView = viewName;

  if (viewName === 'dashboard') {
    document.querySelector('.nav-item[onclick="navigate(\'dashboard\')"]')?.classList.add('active');
    const headerTitle = document.getElementById('mobile-header-title');
    if (headerTitle) headerTitle.textContent = 'Topic Explorer';
    if (typeof loadWorkspaces === 'function') loadWorkspaces();
  } else if (viewName === 'explore') {
    if (params.topicId) {
      const topicItem = document.querySelector(`.topic-item[onclick="openTopic('${params.topicId}')"]`);
      if (topicItem) topicItem.classList.add('active');
      if (typeof loadTopicData === 'function') loadTopicData(params.topicId);
    }
  } else if (viewName === 'notes') {
    document.querySelector('.nav-item[onclick="navigate(\'notes\')"]')?.classList.add('active');
    if (typeof loadNotes === 'function') loadNotes();
  } else if (viewName === 'settings') {
    document.querySelector('.nav-item[onclick="navigate(\'settings\')"]')?.classList.add('active');
    if (typeof loadSettings === 'function') loadSettings();
  }
}

// ─── Theme Toggle ──────────────────────────────────────────────────────────

function appToggleTheme() {
  const current = document.documentElement.getAttribute('data-theme');
  const next = current === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  localStorage.setItem('theme', next);

  // Update button in sidebar
  const themeLabel = document.querySelector('.theme-label');
  const themeIcon = document.querySelector('.theme-icon');
  if (themeLabel) themeLabel.textContent = next === 'dark' ? 'Light Mode' : 'Dark Mode';
  if (themeIcon) themeIcon.innerHTML = next === 'dark' ? '<i data-lucide="sun"></i>' : '<i data-lucide="moon"></i>';
  refreshIcons();
}

// Initialize theme button text on load
document.addEventListener('DOMContentLoaded', () => {
  const current = document.documentElement.getAttribute('data-theme');
  const themeLabel = document.querySelector('.theme-label');
  const themeIcon = document.querySelector('.theme-icon');
  if (themeLabel) themeLabel.textContent = current === 'dark' ? 'Light Mode' : 'Dark Mode';
  if (themeIcon) themeIcon.innerHTML = current === 'dark' ? '<i data-lucide="sun"></i>' : '<i data-lucide="moon"></i>';
  refreshIcons();
});

// ─── Command Palette ───────────────────────────────────────────────────────

document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault();
    openCommandPalette();
  }
});

function openCommandPalette() {
  openModal('command-palette');
  const input = document.getElementById('cmd-input');
  input.value = '';
  input.focus();
  renderCmdResults('');
}

document.getElementById('cmd-input')?.addEventListener('input', (e) => {
  renderCmdResults(e.target.value.toLowerCase());
});

function renderCmdResults(query) {
  const resultsDiv = document.getElementById('cmd-results');
  if (!resultsDiv) return;
  
  // Basic mock results for now, in a real app we'd search topics/workspaces/notes
  let html = '';
  
  if (!query || 'dashboard'.includes(query)) {
    html += `<div class="cmd-item" onclick="closeModal('command-palette'); navigate('dashboard')"><i data-lucide="home" class="mr-sm"></i> Go to Dashboard</div>`;
  }
  if (!query || 'notes'.includes(query)) {
    html += `<div class="cmd-item" onclick="closeModal('command-palette'); navigate('notes')"><i data-lucide="file-text" class="mr-sm"></i> Go to Notes</div>`;
  }
  if (!query || 'settings'.includes(query)) {
    html += `<div class="cmd-item" onclick="closeModal('command-palette'); navigate('settings')"><i data-lucide="settings" class="mr-sm"></i> Go to Settings</div>`;
  }
  
  // Add some actions
  html += `<div class="cmd-item" onclick="closeModal('command-palette'); openModal('create-workspace-modal')"><i data-lucide="folder-plus" class="mr-sm"></i> Create New Workspace</div>`;
  
  resultsDiv.innerHTML = html;
  refreshIcons();
}

// ─── Modals & Overlay ──────────────────────────────────────────────────────

function openModal(id) {
  document.getElementById(id)?.classList.add('active');
}

function closeModal(id) {
  document.getElementById(id)?.classList.remove('active');
}

document.addEventListener('click', (e) => {
  if (e.target.classList.contains('modal-overlay')) {
    e.target.classList.remove('active');
  }
});

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    document.querySelectorAll('.modal-overlay.active').forEach(m => m.classList.remove('active'));
  }
});

// ─── Tabs Initialization ───────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('.tabs-nav').forEach(nav => {
    nav.addEventListener('click', (e) => {
      const tabBtn = e.target.closest('.tab');
      if (!tabBtn) return;

      const target = tabBtn.dataset.tab;
      
      nav.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
      tabBtn.classList.add('active');

      const container = nav.nextElementSibling;
      container.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
      
      const panel = container.querySelector(`[data-panel="${target}"]`);
      if (panel) panel.classList.add('active');
    });
  });
});

// ─── Toasts ────────────────────────────────────────────────────────────────

function showToast(message, type = 'info') {
  const container = document.getElementById('toast-container');
  if (!container) return;

  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.innerHTML = `
    <span>${esc(message)}</span>
    <button onclick="this.parentElement.remove()" class="btn-ghost" style="border:none; cursor:pointer;">&times;</button>
  `;
  container.appendChild(toast);
  
  setTimeout(() => {
    toast.style.opacity = '0';
    setTimeout(() => toast.remove(), 200);
  }, 4000);
}

// ─── Helpers ───────────────────────────────────────────────────────────────

function esc(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function timeAgo(dateStr) {
  if (!dateStr) return '';
  const date = new Date(dateStr + (dateStr.includes('Z') ? '' : 'Z'));
  const now = new Date();
  const diff = now - date;
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ago`;
  if (hours > 0) return `${hours}h ago`;
  if (minutes > 0) return `${minutes}m ago`;
  return 'just now';
}

// ─── Responsive Sidebar Logic ──────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', () => {
  // Restore desktop collapsed state
  const isCollapsed = localStorage.getItem('sidebar_collapsed') === 'true';
  if (isCollapsed && window.innerWidth > 991) {
    document.getElementById('app-sidebar')?.classList.add('collapsed');
  }
});

function toggleSidebar(e) {
  if (e) e.stopPropagation();
  const sidebar = document.getElementById('app-sidebar');
  if (!sidebar) return;
  
  if (window.innerWidth > 991) {
    // Desktop: toggle collapsed state
    sidebar.classList.toggle('collapsed');
    localStorage.setItem('sidebar_collapsed', sidebar.classList.contains('collapsed'));
  } else {
    // Mobile/Tablet: toggle open state
    const isOpen = sidebar.classList.contains('open');
    if (isOpen) closeSidebar();
    else openSidebar();
  }
}

function openSidebar() {
  const sidebar = document.getElementById('app-sidebar');
  const overlay = document.getElementById('sidebar-overlay');
  if (sidebar) sidebar.classList.add('open');
  if (overlay) overlay.classList.add('active');
}

function closeSidebar() {
  const sidebar = document.getElementById('app-sidebar');
  const overlay = document.getElementById('sidebar-overlay');
  if (sidebar) sidebar.classList.remove('open');
  if (overlay) overlay.classList.remove('active');
}

// Global shortcut Ctrl/Cmd + B
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
    e.preventDefault();
    toggleSidebar();
  }
});

// ─── Mobile Swipe Gestures ───────────────────────────────────────────────

let touchStartX = 0;
let touchEndX = 0;

document.addEventListener('touchstart', e => {
  touchStartX = e.changedTouches[0].screenX;
}, { passive: true });

document.addEventListener('touchend', e => {
  touchEndX = e.changedTouches[0].screenX;
  handleSwipe();
}, { passive: true });

function handleSwipe() {
  // Only trigger swipe logic on tablet/mobile screens
  if (window.innerWidth > 991) return;
  
  const SWIPE_THRESHOLD = 50; // Minimum distance to trigger swipe
  const EDGE_ZONE = 30; // Max pixels from left edge to allow swipe-to-open
  
  const diffX = touchEndX - touchStartX;
  const sidebar = document.getElementById('app-sidebar');
  if (!sidebar) return;
  const isOpen = sidebar.classList.contains('open');

  // Swiping Right (to open)
  if (diffX > SWIPE_THRESHOLD && !isOpen) {
    // Only allow opening if starting near the left edge to prevent accidental triggers while scrolling content
    if (touchStartX < EDGE_ZONE) {
      openSidebar();
    }
  }
  
  // Swiping Left (to close)
  if (diffX < -SWIPE_THRESHOLD && isOpen) {
    closeSidebar();
  }
}


// ─── Icons ────────────────────────────────────────────────────────────────
function refreshIcons() {
  if (window.lucide) {
    lucide.createIcons();
  }
}

document.addEventListener('DOMContentLoaded', () => {
  refreshIcons();
});
