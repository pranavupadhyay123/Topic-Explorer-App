/* ═══════════════════════════════════════════════════════════════════════════
   Topic Explorer — Core Application JavaScript
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

// ─── Toast Notifications ───────────────────────────────────────────────────

function showToast(message, type = 'info') {
  let container = document.querySelector('.toast-container');
  if (!container) {
    container = document.createElement('div');
    container.className = 'toast-container';
    document.body.appendChild(container);
  }

  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.style.display = 'flex';
  toast.style.alignItems = 'center';
  toast.style.justifyContent = 'space-between';
  toast.innerHTML = `
    <span style="flex: 1; padding-right: 12px; word-wrap: break-word;">${esc(message)}</span>
    <button onclick="this.parentElement.remove()" style="background:none; border:none; color:inherit; cursor:pointer; font-size:1.4rem; line-height:1; font-weight:bold; opacity:0.7; transition: opacity 0.2s;">&times;</button>
  `;
  container.appendChild(toast);
}

// ─── Modal Helpers ─────────────────────────────────────────────────────────

function openModal(id) {
  document.getElementById(id)?.classList.add('active');
}

function closeModal(id) {
  document.getElementById(id)?.classList.remove('active');
}

// Close modals on overlay click
document.addEventListener('click', (e) => {
  if (e.target.classList.contains('modal-overlay')) {
    e.target.classList.remove('active');
  }
});

// Close modals on Escape
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    document.querySelectorAll('.modal-overlay.active').forEach(m => m.classList.remove('active'));
  }
});

// ─── Tab Switching ─────────────────────────────────────────────────────────

function initTabs(containerSelector) {
  const container = document.querySelector(containerSelector);
  if (!container) return;

  container.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
      const target = tab.dataset.tab;

      container.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      container.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
      const panel = container.querySelector(`[data-panel="${target}"]`);
      if (panel) panel.classList.add('active');

      if (typeof window.onTabChange === 'function') {
        window.onTabChange(target);
      }
    });
  });
}

// ─── Time Formatting ───────────────────────────────────────────────────────

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

function formatDate(dateStr) {
  if (!dateStr) return '';
  const date = new Date(dateStr + (dateStr.includes('Z') ? '' : 'Z'));
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

// ─── Confirm Dialog ────────────────────────────────────────────────────────

function confirmAction(message) {
  return new Promise((resolve) => {
    // Simple confirm for now
    resolve(window.confirm(message));
  });
}

// ─── Loading State ─────────────────────────────────────────────────────────

function showLoading(container, message = 'Loading...') {
  if (typeof container === 'string') container = document.querySelector(container);
  if (!container) return;
  container.innerHTML = `
    <div class="loading-overlay">
      <div class="spinner spinner-lg"></div>
      <div class="loading-text">${message}</div>
    </div>
  `;
}

function showEmpty(container, icon, title, subtitle) {
  if (typeof container === 'string') container = document.querySelector(container);
  if (!container) return;
  container.innerHTML = `
    <div class="empty-state">
      <div class="empty-state-icon">${icon}</div>
      <div class="empty-state-title">${title}</div>
      <div class="card-desc">${subtitle || ''}</div>
    </div>
  `;
}

// ─── Escape HTML ───────────────────────────────────────────────────────────

function esc(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// ─── URL Params ────────────────────────────────────────────────────────────

function getUrlParam(name) {
  return new URLSearchParams(window.location.search).get(name);
}
