/* ═══════════════════════════════════════════════════════════════════════════
   Topic Explorer — Knowledge Graph, Concepts, Timeline, Flashcards, Quiz, Tutor
   ═══════════════════════════════════════════════════════════════════════════ */

const TYPE_COLORS = {
  concept: '#818cf8', theory: '#c084fc', technology: '#34d399',
  person: '#fbbf24', event: '#f472b6', process: '#38bdf8',
  tool: '#4ade80', framework: '#fb923c', principle: '#c4b5fd', pattern: '#67e8f9',
};

const TYPE_ICONS = {
  concept: 'lightbulb', theory: 'pen-tool', technology: 'zap', person: 'user',
  event: 'calendar', process: 'repeat', tool: 'wrench', framework: 'boxes',
  principle: 'ruler', pattern: 'puzzle',
};

const REL_COLORS = {
  depends_on: '#f87171', part_of: '#818cf8', relates_to: '#94a3b8',
  contradicts: '#ef4444', extends: '#34d399', implements: '#38bdf8',
  uses: '#fbbf24', enables: '#a78bfa', precedes: '#f472b6', influences: '#fb923c',
};

let currentTopicId = '';
let currentTopicTitle = '';
let topicData = { concepts: [], relationships: [], timeline: [], flashcards: [], learning_paths: [] };
let conversationId = null;
let currentConceptName = '';
let currentConceptId = '';
let quizQuestions = [];
let quizIndex = 0;
let quizScore = 0;

// ─── Load Data ─────────────────────────────────────────────────────────────

window.loadTopicData = async function(id) {
  currentTopicId = id;
  
  // Find topic title from global topics list
  const topic = topics.find(t => t.id === id);
  currentTopicTitle = topic ? topic.title : 'Topic';
  
  // Update header
  document.getElementById('explore-topic-title').textContent = currentTopicTitle;
  const mobileHeaderTitle = document.getElementById('mobile-header-title');
  if (mobileHeaderTitle) mobileHeaderTitle.textContent = currentTopicTitle;
  
  const wsNameEl = document.getElementById('sidebar-workspace-name');
  document.getElementById('topic-breadcrumb-workspace').textContent = wsNameEl ? wsNameEl.textContent.replace('▼', '').trim() : 'Workspace';
  document.getElementById('topic-breadcrumb-title').textContent = currentTopicTitle;

  try {
    topicData = await API.get(`/api/concepts?topic_id=${currentTopicId}`);
    
    // Set window global for other scripts
    window.currentTopicData = { id: currentTopicId, title: currentTopicTitle, ...topicData };
    
    // Auto-explore if empty
    if (topicData.concepts.length === 0) {
      startExploration();
    } else {
      renderAll();
    }
  } catch (err) {
    console.error('Failed to load topic data:', err);
    showToast('Failed to load topic data', 'error');
  }
}

function renderAll() {
  const status = document.getElementById('explore-topic-status');
  if (status) {
    status.textContent = `${topicData.concepts.length} concepts · ${topicData.relationships.length} relationships`;
  }

  const exploreBtn = document.getElementById('start-explore-btn');
  if (exploreBtn) {
    exploreBtn.style.display = 'inline-flex';
    if (topicData.concepts.length > 0) exploreBtn.innerHTML = '<i data-lucide="refresh-cw" class="mr-xs"></i> Re-explore';
  }

  // Ensure graph container is visible before rendering
  setTimeout(() => {
    renderGraph()
    if(typeof refreshIcons==='function') refreshIcons();;
    renderTree()
    if(typeof refreshIcons==='function') refreshIcons();;
    renderConcepts()
    if(typeof refreshIcons==='function') refreshIcons();;
    renderTimeline()
    if(typeof refreshIcons==='function') refreshIcons();;
    renderFlashcards()
    if(typeof refreshIcons==='function') refreshIcons();;
    renderLearningPaths()
    if(typeof refreshIcons==='function') refreshIcons();;
    
    const quizArea = document.getElementById('quiz-area');
    if (quizArea) {
      quizArea.innerHTML = `
        <div class="empty-state p-2xl">
          <button class="btn btn-primary" onclick="startQuiz()"><i data-lucide="target" class="mr-xs"></i> Start Quiz</button>
          <div class="text-sm text-muted mt-md">Test your knowledge with AI-generated questions</div>
        </div>
      `;
    }
  }, 50);
}

// ─── Exploration ───────────────────────────────────────────────────────────

window.startExploration = async function() {
  const btn = document.getElementById('start-explore-btn');
  if (btn) {
    btn.disabled = true;
    btn.innerHTML = '<div class="spinner"></div> Exploring...';
  }

  showToast('AI is exploring your topic... This may take 15-30 seconds.', 'info');

  try {
    await API.post('/api/ai/explore', { topic_id: currentTopicId, topic: currentTopicTitle });
    showToast('Exploration complete! ', 'success');
    await loadTopicData(currentTopicId);
  } catch (err) {
    showToast('Exploration failed: ' + err.message, 'error');
  } finally {
    if (btn) {
      btn.disabled = false;
      btn.innerHTML = '<i data-lucide="refresh-cw" class="mr-xs"></i> Re-explore';
    }
  }
}

// ─── Knowledge Graph (D3.js) ───────────────────────────────────────────────

function renderGraph()
    if(typeof refreshIcons==='function') refreshIcons(); {
  const container = document.getElementById('graph-container');
  if (!container) return;

  if (topicData.concepts.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon"><i data-lucide="network"></i></div>
        <div class="empty-state-title">Knowledge Graph</div>
        <div class="text-muted">Click "Explore with AI" to generate</div>
      </div>`;
    return;
  }

  container.innerHTML = '';
  const width = container.clientWidth || 800;
  const height = container.clientHeight || 600;

  const svg = d3.select(container).append('svg').attr('width', width).attr('height', height);
  const gMain = svg.append('g');

  const nodeMap = {};
  const nodes = topicData.concepts.map(c => {
    const node = { id: c.id, name: c.name, type: c.type, importance: c.importance, r: 8 + (c.importance || 5) * 2 };
    if (c.id === currentConceptId) { node.fx = width / 2; node.fy = height / 2; }
    nodeMap[c.id] = node;
    return node;
  });

  const links = topicData.relationships
    .filter(r => nodeMap[r.source_concept_id] && nodeMap[r.target_concept_id])
    .map(r => ({
      source: r.source_concept_id,
      target: r.target_concept_id,
      type: r.relationship_type,
      description: r.description,
    }));

  const linkDistance = window.innerWidth < 768 ? 90 : 150;

  const simulation = d3.forceSimulation(nodes)
    .force('link', d3.forceLink(links).id(d => d.id).distance(linkDistance))
    .force('charge', d3.forceManyBody().strength(-800))
    .force('center', d3.forceCenter(width / 2, height / 2))
    .force('collision', d3.forceCollide().radius(d => d.r + 30));

  const defs = svg.append('defs');
  const filter = defs.append('filter').attr('id', 'glow');
  filter.append('feGaussianBlur').attr('stdDeviation', '4').attr('result', 'coloredBlur');
  const feMerge = filter.append('feMerge');
  feMerge.append('feMergeNode').attr('in', 'coloredBlur');
  feMerge.append('feMergeNode').attr('in', 'SourceGraphic');

  const link = gMain.append('g').selectAll('line').data(links).join('line')
    .attr('stroke', d => REL_COLORS[d.type] || '#71717a')
    .attr('stroke-opacity', 0.4)
    .attr('stroke-width', 1.5);

  const node = gMain.append('g').selectAll('g').data(nodes).join('g')
    .style('cursor', 'pointer')
    .call(d3.drag()
      .on('start', (e, d) => { if (!e.active) simulation.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
      .on('drag', (e, d) => { d.fx = e.x; d.fy = e.y; })
      .on('end', (e, d) => { if (!e.active) simulation.alphaTarget(0); d.fx = null; d.fy = null; })
    );

  node.append('circle')
    .attr('r', d => d.r)
    .attr('fill', d => TYPE_COLORS[d.type] || '#818cf8')
    .attr('fill-opacity', 0.8)
    .attr('stroke', '#ffffff')
    .attr('stroke-width', 1)
    .attr('filter', 'url(#glow)')
    .on('click', (e, d) => showConceptDetail(d.id));

  node.append('text')
    .text(d => d.name)
    .attr('text-anchor', 'middle')
    .attr('dy', d => d.r + 14)
    .attr('fill', 'var(--text-secondary)')
    .attr('font-size', '11px');

  simulation.on('tick', () => {
    link.attr('x1', d => d.source.x).attr('y1', d => d.source.y).attr('x2', d => d.target.x).attr('y2', d => d.target.y);
    node.attr('transform', d => `translate(${d.x},${d.y})`);
  });

  const zoom = d3.zoom().scaleExtent([0.2, 4]).on('zoom', e => gMain.attr('transform', e.transform));
  svg.call(zoom);

  // Handle Resize
  window.addEventListener('resize', () => {
    const newWidth = container.clientWidth || 800;
    const newHeight = container.clientHeight || 600;
    svg.attr('width', newWidth).attr('height', newHeight);
    simulation.force('center', d3.forceCenter(newWidth / 2, newHeight / 2));
    simulation.alpha(0.3).restart();
  });
}

// ─── Tree Hierarchy ────────────────────────────────────────────────────────

function buildStrictTree(data) {
  if (!data.concepts || data.concepts.length === 0) return null;
  const rootConcept = data.concepts.find(c => c.name.toLowerCase() === currentTopicTitle.toLowerCase()) || data.concepts[0];
  const visited = new Set([rootConcept.id]);
  
  const buildNode = (concept, edgeLabel) => {
    const node = { id: concept.id, name: concept.name, type: concept.type, description: concept.description, edgeLabel, children: [] };
    const neighbors = data.relationships.filter(r => r.source_concept_id === concept.id || r.target_concept_id === concept.id);
    for (const rel of neighbors) {
      const neighborId = rel.source_concept_id === concept.id ? rel.target_concept_id : rel.source_concept_id;
      if (!visited.has(neighborId)) {
        visited.add(neighborId);
        const neighborConcept = data.concepts.find(c => c.id === neighborId);
        if (neighborConcept) {
          node.children.push(buildNode(neighborConcept, rel.description || rel.relationship_type));
        }
      }
    }
    if (node.children.length === 0) delete node.children;
    return node;
  };

  const rootNode = buildNode(rootConcept, "");
  for (const c of data.concepts) {
    if (!visited.has(c.id)) {
      visited.add(c.id);
      if (!rootNode.children) rootNode.children = [];
      rootNode.children.push({ id: c.id, name: c.name, type: c.type, description: c.description });
    }
  }
  return rootNode;
}

function renderTree()
    if(typeof refreshIcons==='function') refreshIcons(); {
  const container = document.getElementById('tree-container');
  if (!container || topicData.concepts.length === 0) return;
  container.innerHTML = '';

  const width = container.clientWidth || 800;
  const height = container.clientHeight || 600;

  const svg = d3.select(container).append('svg').attr('width', width).attr('height', height);
  const gMain = svg.append('g');
  const treeData = buildStrictTree(topicData);
  if (!treeData) return;

  const root = d3.hierarchy(treeData);
  d3.tree().nodeSize([50, 200])(root);

  let x0 = Infinity, x1 = -x0;
  root.each(d => { if (d.x > x1) x1 = d.x; if (d.x < x0) x0 = d.x; });

  gMain.append('g').attr('fill', 'none').attr('stroke', 'var(--border)').attr('stroke-width', 2)
    .selectAll('path').data(root.links()).join('path')
    .attr('d', d3.linkHorizontal().x(d => d.y).y(d => d.x));

  const node = gMain.append('g').selectAll('g').data(root.descendants()).join('g')
    .attr('transform', d => `translate(${d.y},${d.x})`)
    .style('cursor', 'pointer')
    .on('click', (e, d) => showConceptDetail(d.data.id));

  node.append('rect')
    .attr('x', -60).attr('y', -16).attr('width', 120).attr('height', 32).attr('rx', 4)
    .attr('fill', d => TYPE_COLORS[d.data.type] || '#818cf8')
    .attr('fill-opacity', 0.8);

  node.append('text')
    .attr('text-anchor', 'middle').attr('dy', '0.35em')
    .text(d => d.data.name.length > 18 ? d.data.name.slice(0, 16) + '…' : d.data.name)
    .attr('fill', '#ffffff').attr('font-size', '11px').attr('font-weight', '500');

  const zoom = d3.zoom().scaleExtent([0.3, 3]).on('zoom', e => gMain.attr('transform', e.transform));
  svg.call(zoom);
  svg.call(zoom.transform, d3.zoomIdentity.translate(100, height / 2 - (x0 + x1) / 2));
}

// ─── Concepts List ─────────────────────────────────────────────────────────

function renderConcepts()
    if(typeof refreshIcons==='function') refreshIcons(); {
  const container = document.getElementById('concepts-list');
  if (!container) return;

  if (topicData.concepts.length === 0) {
    container.innerHTML = `<div class="text-muted p-md text-center">Explore the topic to discover concepts</div>`;
    return;
  }

  container.innerHTML = topicData.concepts.map(c => {
    const type = c.type || 'concept';
    const icon = TYPE_ICONS[type] || 'lightbulb';
    return `
      <div class="card concept-card" style="cursor:pointer;" onclick="showConceptDetail('${esc(c.id)}')">
        <div class="flex justify-between items-center mb-sm">
          <div class="font-bold"><i data-lucide="${icon}" style="width:16px;height:16px;margin-right:4px;display:inline-block;vertical-align:-3px;"></i> ${esc(c.name)}</div>
          <div class="text-xs" style="color:${TYPE_COLORS[type]}">${type}</div>
        </div>
        <div class="text-sm text-muted line-clamp-3">${esc(c.description)}</div>
      </div>
    `;
  }).join('');
}

// ─── Concept Detail Modal ──────────────────────────────────────────────────

window.showConceptDetail = function(conceptId) {
  const concept = topicData.concepts.find(c => c.id === conceptId);
  if (!concept) return;

  currentConceptName = concept.name;
  currentConceptId = concept.id;

  document.getElementById('concept-modal-title').textContent = `<i data-lucide="${TYPE_ICONS[concept.type] || 'lightbulb'}" style="display:inline-block;vertical-align:-3px;margin-right:6px;"></i> ${concept.name}`;
  
  let body = `<p class="mb-md">${esc(concept.description)}</p>`;
  
  if (concept.details) {
    body += `<div class="mb-md"><h4 class="font-semibold mb-xs">Details</h4><p class="text-muted text-sm">${esc(concept.details)}</p></div>`;
  }
  
  document.getElementById('concept-modal-body').innerHTML = body;
  if(typeof refreshIcons==='function') refreshIcons();
  openModal('concept-modal');
}

window.expandConcept = async function() {
  const btn = document.getElementById('expand-concept-btn');
  btn.disabled = true;
  btn.innerHTML = '<div class="spinner"></div> Deep Diving...';

  try {
    await API.post('/api/ai/explore', {
      topic_id: currentTopicId,
      concept_name: currentConceptName,
      parent_topic: currentTopicTitle,
      parent_concept_id: currentConceptId,
    });
    showToast('Concept expanded!', 'success');
    closeModal('concept-modal');
    await loadTopicData(currentTopicId);
  } catch (err) {
    showToast('Expansion failed', 'error');
  } finally {
    btn.disabled = false;
    btn.innerHTML = '<i data-lucide="search" class="mr-xs"></i> Deep Dive';
  }
}

// ─── Timeline ──────────────────────────────────────────────────────────────

function renderTimeline()
    if(typeof refreshIcons==='function') refreshIcons(); {
  const container = document.getElementById('timeline-list');
  if (!container) return;

  if (!topicData.timeline || topicData.timeline.length === 0) {
    container.innerHTML = `<div class="text-muted text-center p-md">No timeline available</div>`;
    return;
  }

  container.innerHTML = '<div style="border-left:2px solid var(--border); padding-left:16px; margin-left:16px;">' +
    topicData.timeline.map(e => `
      <div class="mb-lg relative">
        <div style="position:absolute; left:-24px; top:6px; width:14px; height:14px; border-radius:50%; background:var(--primary); border:3px solid var(--bg-main);"></div>
        <div class="text-xs text-primary font-bold mb-xs">${esc(e.date_label)}</div>
        <div class="font-semibold mb-xs">${esc(e.title)}</div>
        <div class="text-sm text-muted">${esc(e.description)}</div>
      </div>
    `).join('') + '</div>';
}

// ─── Flashcards ────────────────────────────────────────────────────────────
let flashcardIndex = 0;
function renderFlashcards()
    if(typeof refreshIcons==='function') refreshIcons(); {
  const container = document.getElementById('flashcards-area');
  if (!container) return;
  if (!topicData.flashcards || topicData.flashcards.length === 0) {
    container.innerHTML = `<div class="text-muted text-center p-md">No flashcards available</div>`;
    return;
  }
  flashcardIndex = 0;
  renderCurrentFlashcard();
}

window.renderCurrentFlashcard = function() {
  const container = document.getElementById('flashcards-area');
  const fc = topicData.flashcards[flashcardIndex];
  
  container.innerHTML = `
    <div class="text-center text-sm text-muted mb-md">Card ${flashcardIndex + 1} of ${topicData.flashcards.length}</div>
    <div class="card p-xl text-center" style="max-width:500px; margin:0 auto; cursor:pointer;" onclick="const a=this.querySelector('.ans'); const q=this.querySelector('.que'); if(a.style.display==='none'){a.style.display='block';q.style.display='none'}else{a.style.display='none';q.style.display='block'}">
      <div class="que text-lg font-semibold">${esc(fc.question)}</div>
      <div class="ans text-lg text-primary" style="display:none;">${esc(fc.answer)}</div>
      <div class="text-xs text-muted mt-lg">(Click to flip)</div>
    </div>
    <div class="flex justify-center gap-md mt-lg">
      <button class="btn btn-secondary" onclick="if(flashcardIndex>0){flashcardIndex--;renderCurrentFlashcard()}">Previous</button>
      <button class="btn btn-primary" onclick="if(flashcardIndex<topicData.flashcards.length-1){flashcardIndex++;renderCurrentFlashcard()}">Next</button>
    </div>
  `;
}

// ─── Learning Paths ────────────────────────────────────────────────────────
function renderLearningPaths()
    if(typeof refreshIcons==='function') refreshIcons(); {
  const container = document.getElementById('learning-path-area');
  if (!container) return;
  if (!topicData.learning_paths || topicData.learning_paths.length === 0) {
    container.innerHTML = `<div class="text-muted text-center p-md">No paths available</div>`;
    return;
  }
  
  container.innerHTML = topicData.learning_paths.map(lp => {
    let steps = [];
    try { steps = typeof lp.steps === 'string' ? JSON.parse(lp.steps) : lp.steps || []; } catch {}
    
    return `
      <div class="card mb-lg">
        <h3 class="font-bold mb-xs">${esc(lp.title)}</h3>
        <p class="text-sm text-muted mb-md">${esc(lp.description)}</p>
        <div class="flex flex-col gap-sm">
          ${steps.map((s,i) => `
            <div class="p-sm bg-surface border rounded">
              <span class="font-semibold text-sm">Step ${i+1}: ${esc(s.title)}</span>
              <p class="text-xs text-muted mt-xs">${esc(s.description||'')}</p>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }).join('');
}

// ─── Tutor Chat ────────────────────────────────────────────────────────────
window.sendChatMessage = async function() {
  const input = document.getElementById('chat-input');
  const message = input.value.trim();
  if (!message) return;

  const messagesDiv = document.getElementById('chat-messages');
  messagesDiv.innerHTML += `<div class="chat-message user">${esc(message)}</div>`;
  input.value = '';
  messagesDiv.scrollTop = messagesDiv.scrollHeight;

  const id = 'typing-'+Date.now();
  messagesDiv.innerHTML += `<div class="chat-message assistant" id="${id}">...</div>`;
  messagesDiv.scrollTop = messagesDiv.scrollHeight;

  try {
    const res = await API.post('/api/ai/tutor', {
      topic: currentTopicTitle,
      topic_id: currentTopicId,
      message,
      conversation_id: conversationId,
    });
    conversationId = res.conversation_id;
    document.getElementById(id).textContent = res.response;
  } catch (err) {
    document.getElementById(id).textContent = 'Error: ' + err.message;
  }
  messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

// ─── Quiz ──────────────────────────────────────────────────────────────────
window.startQuiz = async function() {
  const area = document.getElementById('quiz-area');
  area.innerHTML = '<div class="p-2xl text-center"><div class="spinner"></div> Generating...</div>';
  try {
    const names = topicData.concepts.map(c => c.name);
    quizQuestions = await API.post('/api/ai/quiz', { topic: currentTopicTitle, concept_names: names });
    quizIndex = 0; quizScore = 0;
    renderQuizQuestion();
    if(typeof refreshIcons==='function') refreshIcons();
  } catch (err) {
    area.innerHTML = '<div class="p-2xl text-center text-error">Failed to generate quiz</div>';
  }
}

window.renderQuizQuestion = function() {
  const area = document.getElementById('quiz-area');
  if (quizIndex >= quizQuestions.length) {
    area.innerHTML = `
      <div class="card p-xl text-center max-w-md mx-auto">
        <h2 class="text-2xl mb-md">Quiz Complete!</h2>
        <div class="text-4xl text-primary font-bold mb-sm">${quizScore}/${quizQuestions.length}</div>
        <button class="btn btn-primary mt-lg" onclick="startQuiz()">Play Again</button>
      </div>`;
    return;
  }

  const q = quizQuestions[quizIndex];
  area.innerHTML = `
    <div class="card p-xl mx-auto" style="max-width:600px">
      <div class="text-xs text-muted mb-md">Question ${quizIndex+1}/${quizQuestions.length}</div>
      <h3 class="font-semibold mb-lg">${esc(q.question)}</h3>
      <div class="flex flex-col gap-sm">
        ${q.options.map((opt, i) => `
          <button class="btn btn-secondary text-left w-full p-md" onclick="answerQuiz(${i}, ${q.correct_answer}, this)">${esc(opt)}</button>
        `).join('')}
      </div>
      <div id="quiz-next" class="hidden mt-lg">
        <button class="btn btn-primary w-full" onclick="quizIndex++; renderQuizQuestion();
    if(typeof refreshIcons==='function') refreshIcons();">Next Question</button>
      </div>
    </div>
  `;
}

window.answerQuiz = function(selected, correct, btnNode) {
  if (btnNode.parentNode.dataset.answered) return;
  btnNode.parentNode.dataset.answered = "true";
  
  if (selected === correct) {
    quizScore++;
    btnNode.style.borderColor = 'var(--success)';
    btnNode.style.color = 'var(--success)';
  } else {
    btnNode.style.borderColor = 'var(--error)';
    btnNode.style.color = 'var(--error)';
    btnNode.parentNode.children[correct].style.borderColor = 'var(--success)';
  }
  document.getElementById('quiz-next').classList.remove('hidden');
}
