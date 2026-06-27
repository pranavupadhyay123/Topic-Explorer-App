/* ═══════════════════════════════════════════════════════════════════════════
   Explore Page — Knowledge Graph, Concepts, Timeline, Flashcards, Quiz, Tutor
   ═══════════════════════════════════════════════════════════════════════════ */

const TYPE_COLORS = {
  concept: '#818cf8', theory: '#c084fc', technology: '#34d399',
  person: '#fbbf24', event: '#f472b6', process: '#38bdf8',
  tool: '#4ade80', framework: '#fb923c', principle: '#c4b5fd', pattern: '#67e8f9',
};

const TYPE_ICONS = {
  concept: '💡', theory: '📐', technology: '⚡', person: '👤',
  event: '📅', process: '🔄', tool: '🔧', framework: '🏗️',
  principle: '📏', pattern: '🧩',
};

const REL_COLORS = {
  depends_on: '#f87171', part_of: '#818cf8', relates_to: '#94a3b8',
  contradicts: '#ef4444', extends: '#34d399', implements: '#38bdf8',
  uses: '#fbbf24', enables: '#a78bfa', precedes: '#f472b6', influences: '#fb923c',
};

let topicId = '';
let topicTitle = '';
let data = { concepts: [], relationships: [], timeline: [], flashcards: [], learning_paths: [] };
let conversationId = null;
let currentConceptName = '';
let currentConceptId = '';
let quizQuestions = [];
let quizIndex = 0;
let quizScore = 0;
let isExploring = false;

// ─── Init ──────────────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', () => {
  topicId = getUrlParam('id');
  topicTitle = decodeURIComponent(getUrlParam('title') || '');

  if (!topicId) {
    window.location.href = '/';
    return;
  }

  document.getElementById('topic-title').textContent = topicTitle;
  document.title = `${topicTitle} — Topic Explorer`;

  initTabs('#explore-tabs');
  loadTopicData();

  window.onTabChange = (tab) => {
    if (tab === 'graph') renderGraph();
    if (tab === 'tree') renderTree();
  };

  window.addEventListener('resize', () => {
    const activeTab = document.querySelector('.tab.active')?.dataset.tab;
    if (activeTab === 'graph') renderGraph();
    if (activeTab === 'tree') renderTree();
  });
});

// ─── Load Data ─────────────────────────────────────────────────────────────

async function loadTopicData() {
  try {
    data = await API.get(`/api/concepts?topic_id=${topicId}`);
    
    // Auto-explore if this topic is completely empty
    if (data.concepts.length === 0) {
      startExploration();
    } else {
      renderAll();
    }
  } catch (err) {
    console.error('Failed to load topic data:', err);
  }
}

function renderAll() {
  const status = document.getElementById('topic-status');
  status.textContent = `${data.concepts.length} concepts · ${data.relationships.length} relationships`;

  if (data.concepts.length > 0) {
    const exploreBtn = document.getElementById('explore-btn');
    if (exploreBtn) {
      exploreBtn.style.display = 'inline-flex';
      exploreBtn.innerHTML = '🔄 Re-explore';
    }
  }

  renderGraph();
  renderTree();
  renderConcepts();
  renderTimeline();
  renderFlashcards();
  renderLearningPaths();
}

// ─── Exploration ───────────────────────────────────────────────────────────

async function startExploration() {
  const btn = document.getElementById('explore-btn');
  if (btn) {
    btn.disabled = true;
    btn.innerHTML = '<div class="spinner"></div> Exploring...';
  }

  isExploring = true;

  try {
    // 1. Concepts
    showToast('Generating concepts...', 'info');
    await API.post('/api/ai/generate_concepts', { topic_id: topicId, topic: topicTitle });
    await loadTopicData(); // Renders concepts
    showToast('Concepts ready. Rendering graph...', 'success');
    
    // 2. Relationships
    const conceptNames = data.concepts.map(c => c.name);
    await API.post('/api/ai/generate_relationships', { topic_id: topicId, topic: topicTitle, concept_names: conceptNames });
    await loadTopicData(); // Updates graph with relationships
    showToast('Relationships ready. Updating graph...', 'success');
    
    // 3. Flashcards
    await API.post('/api/ai/generate_flashcards', { topic_id: topicId, topic: topicTitle });
    await loadTopicData(); // Shows flashcards
    showToast('Flashcards ready. Generating quiz...', 'success');

    // 4. Quiz (Using existing quiz logic if we had one, otherwise just tell the user it's ready since it generates on demand in the Quiz tab)
    showToast('Quiz ready. Generating timeline...', 'success');
    
    // 5. Timeline
    await API.post('/api/ai/generate_timeline', { topic_id: topicId, topic: topicTitle });
    await loadTopicData(); // Shows timeline
    showToast('Timeline ready. Generating learning path...', 'success');
    
    // 6. Learning Path
    await API.post('/api/ai/generate_learning_path', { topic_id: topicId, topic: topicTitle });
    await loadTopicData(); // Shows learning path
    showToast('Learning path ready. Exploration complete! 🎉', 'success');
    
  } catch (err) {
    showToast('Exploration failed: ' + err.message, 'error');
  } finally {
    isExploring = false;
    if (btn) {
      btn.disabled = false;
      btn.innerHTML = '🔄 Re-explore';
    }
  }
}

// ─── Knowledge Graph (D3.js) ───────────────────────────────────────────────

function renderGraph() {
  const container = document.getElementById('graph-container');

  if (data.concepts.length === 0 && isExploring) {
    container.innerHTML = `
      <div class="loading-overlay" style="height:100%; display:flex; flex-direction:column; align-items:center; justify-content:center;">
        <div class="spinner" style="width: 48px; height: 48px; margin-bottom: 24px; border-width: 4px;"></div>
        <div class="empty-state-title" style="margin-bottom: 8px;">AI is exploring...</div>
        <div class="card-desc" style="text-align: center; max-width: 400px;">Building knowledge graph for <b>${esc(topicTitle)}</b>.</div>
      </div>`;
    return;
  }

  if (data.concepts.length === 0) {
    container.innerHTML = `
      <div class="loading-overlay" style="height:100%">
        <div class="empty-state-icon">🕸️</div>
        <div class="empty-state-title">Knowledge Graph</div>
        <div class="card-desc">Click "Explore with AI" to generate an interactive knowledge graph</div>
      </div>`;
    return;
  }

  container.innerHTML = '';

  const width = container.clientWidth || container.parentElement?.clientWidth || window.innerWidth * 0.7 || 1000;
  const height = container.clientHeight || 500;

  const svg = d3.select(container)
    .append('svg')
    .attr('width', width)
    .attr('height', height);

  const gMain = svg.append('g');

  // Build nodes and links
  const nodeMap = {};
  const nodes = data.concepts.map(c => {
    const node = { id: c.id, name: c.name, type: c.type, importance: c.importance, r: 8 + (c.importance || 5) * 2 };
    
    // Lock the currently active/expanded node perfectly to the center
    if (c.id === currentConceptId) {
      node.fx = width / 2;
      node.fy = height / 2;
    }
    
    nodeMap[c.id] = node;
    return node;
  });

  const links = data.relationships
    .filter(r => nodeMap[r.source_concept_id] && nodeMap[r.target_concept_id])
    .map(r => ({
      source: r.source_concept_id,
      target: r.target_concept_id,
      type: r.relationship_type,
      description: r.description,
    }));

  // Force simulation
  const simulation = d3.forceSimulation(nodes)
    .force('link', d3.forceLink(links).id(d => d.id).distance(d => Math.max(200, ((d.description || d.type || '').length * 5.5) + 80)))
    .force('charge', d3.forceManyBody().strength(-1200))
    .force('center', d3.forceCenter(width / 2, height / 2))
    .force('collision', d3.forceCollide().radius(d => d.r + 50));

  const dynamicColorScale = d3.scaleOrdinal(d3.schemeCategory10);
  const getRelColor = type => REL_COLORS[type] || dynamicColorScale(type);

  // Gradient & Marker defs
  const defs = svg.append('defs');
  Object.entries(TYPE_COLORS).forEach(([type, color]) => {
    const grad = defs.append('radialGradient').attr('id', `grad-${type}`);
    grad.append('stop').attr('offset', '0%').attr('stop-color', color).attr('stop-opacity', 0.8);
    grad.append('stop').attr('offset', '100%').attr('stop-color', color).attr('stop-opacity', 0.3);
  });

  // Create unique directed arrowhead markers for each relationship color
  const uniqueRelTypes = [...new Set(links.map(l => l.type || 'rel'))];
  uniqueRelTypes.forEach(type => {
    const cleanId = type.replace(/[^a-zA-Z0-9]/g, '_');
    const color = getRelColor(type);
    defs.append('marker')
      .attr('id', `arrow-${cleanId}`)
      .attr('viewBox', '0 -3 6 6')
      .attr('refX', 6)
      .attr('refY', 0)
      .attr('markerWidth', 5)
      .attr('markerHeight', 5)
      .attr('orient', 'auto')
      .append('path')
      .attr('fill', color)
      .attr('d', 'M0,-2.5L6,0L0,2.5');
  });

  // Links — beautiful, minimal, sleek lines with arrowheads
  const link = gMain.append('g')
    .selectAll('line')
    .data(links)
    .join('line')
    .attr('stroke', d => getRelColor(d.type))
    .attr('stroke-opacity', 0.5)
    .attr('stroke-width', 1.2)
    .attr('marker-end', d => `url(#arrow-${(d.type || 'rel').replace(/[^a-zA-Z0-9]/g, '_')})`);

  const linkText = gMain.append('g')
    .selectAll('text')
    .data(links)
    .join('text')
    .text(d => d.description || d.type)
    .attr('font-size', '10px')
    .attr('fill', d => getRelColor(d.type))
    .attr('text-anchor', 'middle')
    .style('pointer-events', 'none')
    .attr('opacity', 0.85);

  // Nodes
  const node = gMain.append('g')
    .selectAll('g')
    .data(nodes)
    .join('g')
    .style('cursor', 'pointer')
    .call(d3.drag()
      .on('start', (event, d) => { if (!event.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
      .on('drag', (event, d) => { d.fx = event.x; d.fy = event.y; })
      .on('end', (event, d) => { if (!event.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; })
    );

  // Glow filter
  const filter = defs.append('filter').attr('id', 'glow');
  filter.append('feGaussianBlur').attr('stdDeviation', '3').attr('result', 'coloredBlur');
  const feMerge = filter.append('feMerge');
  feMerge.append('feMergeNode').attr('in', 'coloredBlur');
  feMerge.append('feMergeNode').attr('in', 'SourceGraphic');

  node.append('circle')
    .attr('r', d => d.r)
    .attr('fill', d => TYPE_COLORS[d.type] || '#818cf8')
    .attr('fill-opacity', 0.7)
    .attr('stroke', d => TYPE_COLORS[d.type] || '#818cf8')
    .attr('stroke-width', 2)
    .attr('filter', 'url(#glow)')
    .on('click', (event, d) => showConceptDetail(d.id));

  node.append('text')
    .text(d => d.name.length > 15 ? d.name.slice(0, 13) + '…' : d.name)
    .attr('text-anchor', 'middle')
    .attr('dy', d => d.r + 14)
    .attr('fill', '#94a3b8')
    .attr('font-size', '11px')
    .attr('font-family', 'Inter, sans-serif');

  // Hover effects — highlight connected relationships, fade others
  node.on('mouseenter', function (event, d) {
    d3.select(this).select('circle')
      .transition().duration(200)
      .attr('r', d.r * 1.3)
      .attr('fill-opacity', 1);

    link.transition().duration(200)
      .attr('stroke-opacity', l => (l.source.id === d.id || l.target.id === d.id) ? 0.95 : 0.15)
      .attr('stroke-width', l => (l.source.id === d.id || l.target.id === d.id) ? 2 : 1.2);
    
    linkText.transition().duration(200)
      .attr('opacity', l => (l.source.id === d.id || l.target.id === d.id) ? 1 : 0.15);
  }).on('mouseleave', function (event, d) {
    d3.select(this).select('circle')
      .transition().duration(200)
      .attr('r', d.r)
      .attr('fill-opacity', 0.7);

    link.transition().duration(200)
      .attr('stroke-opacity', 0.5)
      .attr('stroke-width', 1.2);
    
    linkText.transition().duration(200)
      .attr('opacity', 0.85);
  });

  // Tick — calculate line endpoints to touch exact circle rim
  simulation.on('tick', () => {
    link.each(function(d) {
      const targetR = (d.target.r || 16) + 3;
      const dx = d.target.x - d.source.x;
      const dy = d.target.y - d.source.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist === 0) return;
      
      const ux = dx / dist;
      const uy = dy / dist;
      
      d3.select(this)
        .attr('x1', d.source.x)
        .attr('y1', d.source.y)
        .attr('x2', d.target.x - ux * targetR)
        .attr('y2', d.target.y - uy * targetR);
    });

    linkText
      .attr('transform', d => {
        const cx = (d.source.x + d.target.x) / 2;
        const cy = (d.source.y + d.target.y) / 2;
        let angle = Math.atan2(d.target.y - d.source.y, d.target.x - d.source.x) * 180 / Math.PI;
        if (angle > 90 || angle < -90) angle += 180;
        return `translate(${cx}, ${cy}) rotate(${angle}) translate(0, -5)`;
      });

    node.attr('transform', d => `translate(${d.x},${d.y})`);
  });

  // Zoom
  const zoom = d3.zoom()
    .scaleExtent([0.3, 3])
    .on('zoom', (event) => {
      gMain.attr('transform', event.transform);
    });

  svg.call(zoom);
}

// ─── Tree Hierarchy ────────────────────────────────────────────────────────

function buildStrictTree(data) {
  if (!data.concepts || data.concepts.length === 0) return null;
  const rootConcept = data.concepts.find(c => c.name.toLowerCase() === topicTitle.toLowerCase()) || data.concepts[0];
  const visited = new Set([rootConcept.id]);
  
  const buildNode = (concept, edgeLabel, edgeType) => {
    const node = { id: concept.id, name: concept.name, type: concept.type, description: concept.description, edgeLabel: edgeLabel, edgeType: edgeType, children: [] };
    const neighbors = data.relationships.filter(r => r.source_concept_id === concept.id || r.target_concept_id === concept.id);
    for (const rel of neighbors) {
      const neighborId = rel.source_concept_id === concept.id ? rel.target_concept_id : rel.source_concept_id;
      if (!visited.has(neighborId)) {
        visited.add(neighborId);
        const neighborConcept = data.concepts.find(c => c.id === neighborId);
        if (neighborConcept) {
          node.children.push(buildNode(neighborConcept, rel.description || rel.relationship_type, rel.relationship_type));
        }
      }
    }
    if (node.children.length === 0) delete node.children;
    return node;
  };

  const rootNode = buildNode(rootConcept, "", "root");
  
  for (const c of data.concepts) {
    if (!visited.has(c.id)) {
      visited.add(c.id);
      if (!rootNode.children) rootNode.children = [];
      rootNode.children.push({ id: c.id, name: c.name, type: c.type, description: c.description });
    }
  }
  return rootNode;
}

function renderTree() {
  const container = document.getElementById('tree-container');
  if (!container || data.concepts.length === 0) return;
  container.innerHTML = '';

  const width = container.clientWidth || container.parentElement?.clientWidth || window.innerWidth * 0.7 || 800;
  const height = container.clientHeight || 600;

  const svg = d3.select('#tree-container').append('svg')
    .attr('width', '100%')
    .attr('height', '100%')
    .attr('viewBox', [0, 0, width, height]);

  const gMain = svg.append('g');
  const treeData = buildStrictTree(data);
  if (!treeData) return;

  const root = d3.hierarchy(treeData);

  let maxEdgeLength = 0;
  root.each(d => {
    if (d.data.edgeLabel && d.data.edgeLabel.length > maxEdgeLength) {
      maxEdgeLength = d.data.edgeLabel.length;
    }
  });

  const dx = 60;
  const dy = Math.max(250, (maxEdgeLength * 5.5) + 80);
  const treeLayout = d3.tree().nodeSize([dx, dy]);
  treeLayout(root);

  let x0 = Infinity;
  let x1 = -x0;
  root.each(d => {
    if (d.x > x1) x1 = d.x;
    if (d.x < x0) x0 = d.x;
  });

  const dynamicColorScale = d3.scaleOrdinal(d3.schemeCategory10);

  const link = gMain.append('g')
    .attr('fill', 'none')
    .attr('stroke-width', 2)
    .selectAll('path')
    .data(root.links())
    .join('path')
    .attr('stroke', (d, i) => dynamicColorScale(i))
    .attr('d', d3.linkHorizontal().x(d => d.y).y(d => d.x));

  const linkText = gMain.append('g')
    .selectAll('text')
    .data(root.links())
    .join('text')
    .text(d => d.target.data.edgeLabel || '')
    .attr('font-size', '10px')
    .attr('fill', (d, i) => dynamicColorScale(i))
    .attr('text-anchor', 'middle')
    .style('pointer-events', 'none')
    .attr('transform', d => {
      const cx = (d.source.y + d.target.y) / 2;
      const cy = (d.source.x + d.target.x) / 2;
      let angle = Math.atan2(d.target.x - d.source.x, d.target.y - d.source.y) * 180 / Math.PI;
      if (angle > 90 || angle < -90) angle += 180;
      return `translate(${cx}, ${cy}) rotate(${angle}) translate(0, -6)`;
    });

  const node = gMain.append('g')
    .selectAll('g')
    .data(root.descendants())
    .join('g')
    .attr('transform', d => `translate(${d.y},${d.x})`)
    .style('cursor', 'pointer')
    .on('click', (event, d) => showConceptDetail(d.data.id));

  node.append('rect')
    .attr('x', -60)
    .attr('y', -18)
    .attr('width', 120)
    .attr('height', 36)
    .attr('rx', 4)
    .attr('fill', d => TYPE_COLORS[d.data.type] || '#818cf8')
    .attr('fill-opacity', 0.85)
    .attr('stroke', d => TYPE_COLORS[d.data.type] || '#818cf8')
    .attr('stroke-width', 2);

  node.append('text')
    .attr('text-anchor', 'middle')
    .attr('dy', '0.35em')
    .text(d => d.data.name.length > 18 ? d.data.name.slice(0, 16) + '…' : d.data.name)
    .attr('fill', '#ffffff')
    .attr('font-size', '11px')
    .attr('font-weight', '600')
    .style('text-shadow', '0px 1px 3px rgba(0,0,0,0.8)');

  const zoom = d3.zoom()
    .scaleExtent([0.3, 3])
    .on('zoom', (event) => gMain.attr('transform', event.transform));

  svg.call(zoom);
  svg.call(zoom.transform, d3.zoomIdentity.translate(dy, height / 2 - (x0 + x1) / 2));
}

// ─── Concepts List ─────────────────────────────────────────────────────────

function renderConcepts() {
  const container = document.getElementById('concepts-list');

  if (data.concepts.length === 0) {
    showEmpty(container, '📦', 'No concepts yet', 'Explore the topic to discover concepts');
    return;
  }

  container.innerHTML = data.concepts.map((c, i) => {
    const type = c.type || 'concept';
    const icon = TYPE_ICONS[type] || '💡';
    const impDots = Array.from({length: 10}, (_, j) =>
      `<span class="importance-dot ${j < (c.importance || 5) ? 'filled' : ''}"></span>`
    ).join('');

    return `
      <div class="concept-card animate-in" style="animation-delay:${i * 0.03}s" onclick="showConceptDetail('${esc(c.id)}')">
        <div class="concept-card-header">
          <span class="concept-card-name">${icon} ${esc(c.name)}</span>
          <span class="badge badge-${type}">${type}</span>
        </div>
        <div class="card-desc">${esc(c.description)}</div>
        <div class="concept-card-importance mt-sm">${impDots}</div>
      </div>
    `;
  }).join('');
}

// ─── Concept Detail Modal ──────────────────────────────────────────────────

function showConceptDetail(conceptId) {
  const concept = data.concepts.find(c => c.id === conceptId);
  if (!concept) return;

  currentConceptName = concept.name;
  currentConceptId = concept.id;

  document.getElementById('concept-modal-title').innerHTML =
    `${TYPE_ICONS[concept.type] || '💡'} ${esc(concept.name)}`;

  let body = `
    <div class="flex gap-sm mb-md">
      <span class="badge badge-${concept.type || 'concept'}">${concept.type || 'concept'}</span>
      <span class="badge badge-primary">Importance: ${concept.importance}/10</span>
    </div>
    <p class="card-desc mb-md">${esc(concept.description)}</p>
  `;

  if (concept.details) {
    body += `
      <div class="mb-md">
        <h4 class="font-semibold mb-sm">Details</h4>
        <div class="card-desc" style="white-space:pre-wrap;">${esc(concept.details)}</div>
      </div>
    `;
  }

  // Code examples
  let codeExamples = [];
  try { codeExamples = typeof concept.code_examples === 'string' ? JSON.parse(concept.code_examples) : concept.code_examples || []; } catch {}
  if (codeExamples.length > 0) {
    body += `<div class="mb-md"><h4 class="font-semibold mb-sm">Code Examples</h4>`;
    codeExamples.forEach(ex => {
      body += `
        <div class="card mb-sm" style="padding:12px;">
          <div class="text-xs text-muted mb-sm">${esc(ex.language || 'code')} — ${esc(ex.description || '')}</div>
          <pre style="background:var(--bg-root);padding:12px;border-radius:8px;overflow-x:auto;font-family:var(--font-mono);font-size:0.8rem;color:var(--primary-light);">${esc(ex.code || '')}</pre>
        </div>
      `;
    });
    body += `</div>`;
  }

  // Related relationships
  const rels = data.relationships.filter(r =>
    r.source_concept_id === conceptId || r.target_concept_id === conceptId
  );
  if (rels.length > 0) {
    body += `<div class="mb-md"><h4 class="font-semibold mb-sm">Relationships</h4>`;
    rels.forEach(r => {
      const otherName = r.source_concept_id === conceptId
        ? data.concepts.find(c => c.id === r.target_concept_id)?.name || '?'
        : data.concepts.find(c => c.id === r.source_concept_id)?.name || '?';
      body += `<div class="flex gap-sm items-center mb-sm text-sm">
        <span style="color:${REL_COLORS[r.relationship_type] || '#94a3b8'}">●</span>
        <span class="text-secondary">${esc(r.relationship_type)}</span>
        <span>→</span>
        <span class="font-semibold">${esc(otherName)}</span>
      </div>`;
    });
    body += `</div>`;
  }

  document.getElementById('concept-modal-body').innerHTML = body;
  openModal('concept-modal');
}

async function expandConcept() {
  const btn = document.getElementById('expand-concept-btn');
  btn.disabled = true;
  btn.innerHTML = '<div class="spinner"></div> Expanding...';

  try {
    await API.post('/api/ai/explore', {
      topic_id: topicId,
      concept_name: currentConceptName,
      parent_topic: topicTitle,
      parent_concept_id: currentConceptId,
    });
    showToast('Concept expanded with new sub-concepts!', 'success');
    closeModal('concept-modal');
    await loadTopicData();
  } catch (err) {
    showToast('Expansion failed: ' + err.message, 'error');
  } finally {
    btn.disabled = false;
    btn.innerHTML = '🔍 Deep Dive';
  }
}

// ─── Timeline ──────────────────────────────────────────────────────────────

function renderTimeline() {
  const container = document.getElementById('timeline-list');

  if (!data.timeline || data.timeline.length === 0) {
    showEmpty(container, '📅', 'No timeline events', 'Explore the topic to discover its timeline');
    return;
  }

  // Sort events by order_index if available
  const sorted = [...data.timeline].sort((a, b) => (a.order_index || 0) - (b.order_index || 0));

  container.innerHTML = `
    <div class="flex justify-between items-center mb-lg pb-md" style="border-bottom: 1px solid var(--border);">
      <div>
        <h3 class="font-heading text-lg font-bold" style="color:#fff;font-size:1.5rem;">Timeline & Milestones</h3>
        <p class="text-secondary text-sm">Chronological progression and key historical milestones in <b>${esc(topicTitle)}</b></p>
      </div>
      <span class="badge badge-primary">${sorted.length} Events</span>
    </div>
    <div class="timeline">
      ${sorted.map((e, i) => {
        const impBadge = e.importance === 'high' ? '<span class="badge badge-warning">⚡ High Impact</span>' : '';
        const catBadge = e.category && e.category !== 'general' ? `<span class="badge badge-secondary">${esc(e.category)}</span>` : '';
        return `
          <div class="timeline-item animate-in" style="animation-delay:${i * 0.08}s">
            <div class="timeline-header">
              <span class="timeline-date">⏱ ${esc(e.date_label || 'Period')} ${e.period ? '· ' + esc(e.period) : ''}</span>
              <div class="flex gap-sm">${catBadge} ${impBadge}</div>
            </div>
            <div class="timeline-title">${esc(e.title)}</div>
            <div class="timeline-desc">${esc(e.description)}</div>
          </div>
        `;
      }).join('')}
    </div>
  `;
}

// ─── Flashcards ────────────────────────────────────────────────────────────

let flashcardIndex = 0;
let flashcardStats = { known: 0, review: 0 };

function renderFlashcards() {
  const container = document.getElementById('flashcards-area');

  if (!data.flashcards || data.flashcards.length === 0) {
    showEmpty(container, '🃏', 'No flashcards yet', 'Explore the topic to generate flashcards');
    return;
  }

  flashcardIndex = 0;
  flashcardStats = { known: 0, review: 0 };
  renderCurrentFlashcard();

  // Attach keyboard shortcuts
  document.removeEventListener('keydown', handleFlashcardKeys);
  document.addEventListener('keydown', handleFlashcardKeys);
}

function handleFlashcardKeys(e) {
  const activeTab = document.querySelector('.tab.active');
  if (!activeTab || activeTab.getAttribute('data-tab') !== 'flashcards') return;
  if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

  if (e.code === 'Space') {
    e.preventDefault();
    const card = document.querySelector('.flashcard');
    if (card) card.classList.toggle('flipped');
  } else if (e.code === 'ArrowRight') {
    e.preventDefault();
    nextFlashcard();
  } else if (e.code === 'ArrowLeft') {
    e.preventDefault();
    prevFlashcard();
  }
}

function renderCurrentFlashcard() {
  const container = document.getElementById('flashcards-area');
  const total = data.flashcards.length;
  const fc = data.flashcards[flashcardIndex];
  const progressPct = ((flashcardIndex + 1) / total) * 100;
  const diffStars = '★'.repeat(fc.difficulty || 2) + '☆'.repeat(Math.max(0, 5 - (fc.difficulty || 2)));

  container.innerHTML = `
    <div class="flashcards-container animate-in">
      <div class="flex justify-between items-center text-sm text-secondary mb-xs">
        <span class="font-semibold" style="color:var(--text-primary);">Card ${flashcardIndex + 1} of ${total}</span>
        <span class="badge badge-primary">Difficulty: <span style="color:#fbbf24;letter-spacing:1px;margin-left:4px;">${diffStars}</span></span>
      </div>
      <div class="flashcard-progress-bar">
        <div class="flashcard-progress-fill" style="width:${progressPct}%"></div>
      </div>

      <div class="flashcard" onclick="this.classList.toggle('flipped')">
        <div class="flashcard-inner">
          <div class="flashcard-front">
            <div class="flashcard-label">
              <span>❓ Question</span>
              <span>Click or Press [Space] to flip 🔄</span>
            </div>
            <div class="flashcard-content">${esc(fc.question)}</div>
            <div class="flashcard-hint">Hover / Click card to reveal answer</div>
          </div>
          <div class="flashcard-back">
            <div class="flashcard-label" style="color:var(--primary-light);">
              <span>💡 Answer</span>
              <span>Click to flip back</span>
            </div>
            <div class="flashcard-content">${esc(fc.answer)}</div>
            <div class="flashcard-hint" style="color:#a855f7;">Did you get it right? Rate below ↓</div>
          </div>
        </div>
      </div>

      <div class="flex justify-between items-center gap-md mt-lg flex-wrap">
        <button class="btn btn-secondary" onclick="prevFlashcard()" ${flashcardIndex === 0 ? 'disabled' : ''}>← Previous [◀]</button>
        <div class="flex gap-sm">
          <button class="btn btn-sm" style="background:var(--warning-bg);color:var(--warning);border:1px solid rgba(245,158,11,0.3);" onclick="markFlashcard('review'); nextFlashcard();">🔁 Needs Review</button>
          <button class="btn btn-sm" style="background:var(--success-bg);color:var(--success);border:1px solid rgba(16,185,129,0.3);" onclick="markFlashcard('known'); nextFlashcard();">⚡ Got It!</button>
        </div>
        <button class="btn btn-primary" onclick="nextFlashcard()" ${flashcardIndex === total - 1 ? 'disabled' : ''}>Next [▶] →</button>
      </div>

      ${flashcardStats.known > 0 || flashcardStats.review > 0 ? `
        <div class="flex justify-center gap-lg mt-md text-xs text-muted">
          <span style="color:var(--success);font-weight:600;">⚡ Mastered: ${flashcardStats.known}</span>
          <span style="color:var(--warning);font-weight:600;">🔁 Reviewing: ${flashcardStats.review}</span>
        </div>
      ` : ''}
    </div>
  `;
}

function nextFlashcard() {
  if (flashcardIndex < data.flashcards.length - 1) {
    flashcardIndex++;
    renderCurrentFlashcard();
  }
}

function prevFlashcard() {
  if (flashcardIndex > 0) {
    flashcardIndex--;
    renderCurrentFlashcard();
  }
}

function markFlashcard(status) {
  if (status === 'known') flashcardStats.known++;
  if (status === 'review') flashcardStats.review++;
  showToast(status === 'known' ? 'Marked as mastered! ⚡' : 'Added to review queue 🔁', 'info');
}

// ─── Learning Paths ────────────────────────────────────────────────────────

function renderLearningPaths() {
  const container = document.getElementById('learning-path-area');

  if (!data.learning_paths || data.learning_paths.length === 0) {
    showEmpty(container, '📚', 'No learning paths yet', 'Explore the topic to generate a learning path');
    return;
  }

  container.innerHTML = data.learning_paths.map((lp, lpIdx) => {
    let steps = [];
    try { steps = typeof lp.steps === 'string' ? JSON.parse(lp.steps) : lp.steps || []; } catch {}

    // Load progress from localStorage
    const progressKey = `lp_progress_${topicId}_${lp.id || lpIdx}`;
    let completedSteps = [];
    try { completedSteps = JSON.parse(localStorage.getItem(progressKey)) || []; } catch {}

    const completedCount = steps.filter((_, idx) => completedSteps.includes(idx)).length;
    const progressPct = steps.length > 0 ? Math.round((completedCount / steps.length) * 100) : 0;

    return `
      <div class="learning-path-card animate-in" style="animation-delay:${lpIdx * 0.1}s">
        <div class="learning-path-header">
          <div>
            <div class="flex items-center gap-sm mb-xs">
              <span class="badge badge-primary">Structured Curriculum</span>
              <span class="badge badge-${lp.difficulty === 'advanced' ? 'error' : lp.difficulty === 'intermediate' ? 'warning' : 'success'}">${esc(lp.difficulty || 'Beginner')}</span>
              ${lp.estimated_time ? `<span class="badge badge-secondary">⏱ ${esc(lp.estimated_time)}</span>` : ''}
            </div>
            <h3 class="font-heading text-xl font-bold" style="color:#fff;font-size:1.6rem;">${esc(lp.title)}</h3>
            <p class="text-secondary mt-xs">${esc(lp.description)}</p>
          </div>
          <div style="min-width:200px;text-align:right;">
            <div class="text-xs text-muted font-semibold uppercase mb-xs">Progress: ${completedCount}/${steps.length} steps (${progressPct}%)</div>
            <div class="flashcard-progress-bar">
              <div class="flashcard-progress-fill" style="width:${progressPct}%;background:${progressPct === 100 ? 'var(--success)' : 'linear-gradient(90deg, var(--primary), var(--tertiary))'};"></div>
            </div>
          </div>
        </div>

        <div class="learning-step-list">
          ${steps.map((s, i) => {
            const isCompleted = completedSteps.includes(i);
            return `
              <div class="learning-step-card ${isCompleted ? 'completed' : ''}" onclick="toggleStepComplete('${esc(lp.id || lpIdx)}', ${i})">
                <div class="step-checkbox">${isCompleted ? '✓' : ''}</div>
                <div style="flex:1;">
                  <div class="step-number">Step ${i + 1}</div>
                  <div class="step-title">${esc(s.title)}</div>
                  <div class="step-desc">${esc(s.description || '')}</div>
                </div>
              </div>
            `;
          }).join('')}
        </div>
      </div>
    `;
  }).join('');
}

function toggleStepComplete(lpId, stepIndex) {
  const progressKey = `lp_progress_${topicId}_${lpId}`;
  let completedSteps = [];
  try { completedSteps = JSON.parse(localStorage.getItem(progressKey)) || []; } catch {}

  if (completedSteps.includes(stepIndex)) {
    completedSteps = completedSteps.filter(i => i !== stepIndex);
  } else {
    completedSteps.push(stepIndex);
  }

  try { localStorage.setItem(progressKey, JSON.stringify(completedSteps)); } catch {}
  renderLearningPaths();
}

// ─── Quiz ──────────────────────────────────────────────────────────────────

async function startQuiz() {
  const area = document.getElementById('quiz-area');
  showLoading(area, 'Generating quiz questions...');

  try {
    const names = data.concepts.map(c => c.name);
    quizQuestions = await API.post('/api/ai/quiz', { topic: topicTitle, concept_names: names });
    quizIndex = 0;
    quizScore = 0;
    renderQuizQuestion();
  } catch (err) {
    showToast('Failed to generate quiz: ' + err.message, 'error');
    showEmpty(area, '❓', 'Quiz generation failed', 'Try again later');
  }
}

function renderQuizQuestion() {
  const area = document.getElementById('quiz-area');

  if (quizQuestions.length === 0) {
    area.innerHTML = `
      <div class="card text-center animate-in" style="max-width:600px;margin:var(--space-2xl) auto;padding:var(--space-2xl);">
        <div style="font-size:3.5rem;margin-bottom:16px;">🎯</div>
        <h3 class="card-title" style="font-size:1.75rem;">AI-Powered Knowledge Quiz</h3>
        <p class="card-desc mt-sm mb-xl">Test your mastery of <b>${esc(topicTitle)}</b> with dynamically generated multiple choice questions.</p>
        <button class="btn btn-primary btn-lg" onclick="startQuiz()">🚀 Start Quiz Now</button>
      </div>
    `;
    return;
  }

  if (quizIndex >= quizQuestions.length) {
    const pct = Math.round((quizScore / quizQuestions.length) * 100);
    const badgeType = pct >= 80 ? 'success' : pct >= 50 ? 'warning' : 'error';
    area.innerHTML = `
      <div class="card text-center animate-in" style="max-width:550px;margin:var(--space-2xl) auto;padding:var(--space-2xl);">
        <div style="font-size:4rem;margin-bottom:16px;">${pct >= 80 ? '🏆' : pct >= 50 ? '⭐' : '📚'}</div>
        <div class="card-title" style="font-size:1.8rem;">Quiz Completed!</div>
        <div class="badge badge-${badgeType} mt-sm mb-lg" style="font-size:1rem;padding:6px 16px;">Score: ${pct}%</div>
        <div style="font-size:2.5rem;font-weight:800;color:#fff;">${quizScore} / ${quizQuestions.length}</div>
        <p class="text-secondary mt-sm mb-xl">You got ${quizScore} questions correct out of ${quizQuestions.length}.</p>
        <button class="btn btn-primary btn-lg" onclick="quizIndex=0;quizScore=0;renderQuizQuestion();">🔄 Try Again</button>
      </div>
    `;
    return;
  }

  const q = quizQuestions[quizIndex];
  area.innerHTML = `
    <div class="card animate-in" style="max-width:750px;margin:var(--space-xl) auto;padding:var(--space-2xl);">
      <div class="flex justify-between items-center mb-lg pb-md" style="border-bottom:1px solid var(--border);">
        <span class="badge badge-primary">Question ${quizIndex + 1} of ${quizQuestions.length}</span>
        <span class="font-semibold text-secondary">Score: <b style="color:var(--success);">${quizScore}</b></span>
      </div>
      <div class="card-title mb-xl" style="font-size:1.35rem;line-height:1.5;">${esc(q.question)}</div>
      <div class="grid gap-sm mb-lg">
        ${q.options.map((opt, i) => `
          <button class="quiz-option" id="quiz-opt-${i}" onclick="answerQuiz(${i})">
            <span><b>${String.fromCharCode(65 + i)}.</b> ${esc(opt)}</span>
            <span class="quiz-status-icon"></span>
          </button>
        `).join('')}
      </div>
      <div id="quiz-explanation" class="hidden mt-md p-md card-desc" style="background:rgba(255,255,255,0.03);padding:16px;border-radius:var(--radius-md);border-left:4px solid var(--primary);"></div>
      <div id="quiz-next" class="hidden mt-lg text-right">
        <button class="btn btn-primary btn-lg" onclick="quizIndex++;renderQuizQuestion();">Next Question →</button>
      </div>
    </div>
  `;
}

function answerQuiz(selected) {
  const q = quizQuestions[quizIndex];
  const correct = q.correct_answer;

  document.querySelectorAll('.quiz-option').forEach((opt, i) => {
    opt.disabled = true;
    opt.onclick = null;
    if (i === correct) opt.classList.add('correct');
    if (i === selected && i !== correct) opt.classList.add('wrong');
    if (i === selected) opt.classList.add('selected');
  });

  if (selected === correct) quizScore++;

  const explanation = document.getElementById('quiz-explanation');
  explanation.classList.remove('hidden');
  explanation.textContent = q.explanation || (selected === correct ? 'Correct! 🎉' : 'Not quite. Review this concept.');

  document.getElementById('quiz-next').classList.remove('hidden');
}

// Initialize quiz area
document.addEventListener('DOMContentLoaded', () => {
  const quizArea = document.getElementById('quiz-area');
  if (quizArea) {
    quizArea.innerHTML = `
      <div class="card text-center animate-in" style="max-width:600px;margin:var(--space-2xl) auto;padding:var(--space-2xl);">
        <div style="font-size:3.5rem;margin-bottom:16px;">🎯</div>
        <h3 class="card-title" style="font-size:1.75rem;">AI-Powered Knowledge Quiz</h3>
        <p class="card-desc mt-sm mb-xl">Test your mastery of this topic with dynamically generated multiple choice questions.</p>
        <button class="btn btn-primary btn-lg" onclick="startQuiz()">🚀 Start Quiz Now</button>
      </div>
    `;
  }
});

// ─── AI Tutor Chat ─────────────────────────────────────────────────────────

async function sendChatMessage() {
  const input = document.getElementById('chat-input');
  const message = input.value.trim();
  if (!message) return;

  const messagesDiv = document.getElementById('chat-messages');

  // Add user message
  messagesDiv.innerHTML += `<div class="chat-message user">${esc(message)}</div>`;
  input.value = '';
  messagesDiv.scrollTop = messagesDiv.scrollHeight;

  // Show typing indicator
  messagesDiv.innerHTML += `<div class="chat-message assistant" id="typing-indicator"><div class="spinner" style="width:16px;height:16px;"></div> Thinking...</div>`;
  messagesDiv.scrollTop = messagesDiv.scrollHeight;

  try {
    const result = await API.post('/api/ai/tutor', {
      topic: topicTitle,
      topic_id: topicId,
      message,
      conversation_id: conversationId,
    });

    conversationId = result.conversation_id;

    document.getElementById('typing-indicator')?.remove();
    messagesDiv.innerHTML += `<div class="chat-message assistant">${esc(result.response)}</div>`;
    messagesDiv.scrollTop = messagesDiv.scrollHeight;
  } catch (err) {
    document.getElementById('typing-indicator')?.remove();
    messagesDiv.innerHTML += `<div class="chat-message assistant" style="color:var(--error);">Error: ${esc(err.message)}</div>`;
  }
}
