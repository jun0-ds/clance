const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { LogicalSize } = window.__TAURI__.dpi;

const WIDGET_WIDTH = 300;
let currentSort = 'cpu';

function formatGb(gb) {
  return gb < 10 ? gb.toFixed(1) : Math.round(gb);
}

function setBar(id, percent) {
  const el = document.getElementById(id);
  el.style.width = Math.min(100, percent) + '%';
  if (percent > 80) {
    el.style.background = 'linear-gradient(90deg, #f5576c, #ff6b6b)';
  } else if (percent > 60) {
    el.style.background = 'linear-gradient(90deg, #f093fb, #f5576c)';
  } else {
    el.style.background = 'linear-gradient(90deg, #4facfe, #00f2fe)';
  }
}

async function updateCpu() {
  const info = await invoke('get_cpu_info');
  document.getElementById('cpu-value').textContent = Math.round(info.usage) + '%';
  document.getElementById('cpu-name').textContent = info.name;
  setBar('cpu-bar', info.usage);
}

async function updateMemory() {
  const info = await invoke('get_memory_info');
  document.getElementById('memory-value').textContent =
    formatGb(info.used_gb) + '/' + formatGb(info.total_gb) + 'GB';
  setBar('memory-bar', info.usage_percent);
}

async function updateGpu() {
  const info = await invoke('get_gpu_info');
  const section = document.getElementById('gpu-section');
  if (!info.available) {
    section.style.display = 'none';
    return;
  }
  section.style.display = '';
  document.getElementById('gpu-value').textContent = info.usage_percent + '%';
  document.getElementById('gpu-name').textContent = info.name;
  // Util bar
  document.getElementById('gpu-util-value').textContent = info.usage_percent + '%';
  setBar('gpu-util-bar', info.usage_percent);
  // VRAM bar
  const vramPercent = info.memory_total_mb > 0 ? (info.memory_used_mb / info.memory_total_mb) * 100 : 0;
  const usedGb = (info.memory_used_mb / 1024).toFixed(1);
  const totalGb = (info.memory_total_mb / 1024).toFixed(1);
  document.getElementById('gpu-vram-value').textContent = usedGb + '/' + totalGb + 'G';
  setBar('gpu-vram-bar', vramPercent);
}

function formatProcValue(proc, sortBy) {
  switch (sortBy) {
    case 'memory': return proc.memory_mb.toFixed(0) + 'MB';
    case 'gpu': return proc.gpu_memory_mb.toFixed(0) + 'MB';
    default: return proc.cpu_usage.toFixed(1) + '%';
  }
}

async function updateProcesses() {
  const procs = await invoke('get_top_processes', { sortBy: currentSort });
  const list = document.getElementById('process-list');
  list.innerHTML = procs.map((p, i) =>
    `<li><span class="proc-name">${i + 1}. ${p.name}</span><span class="proc-value">${formatProcValue(p, currentSort)}</span></li>`
  ).join('');
}

async function resizeToContent() {
  const widget = document.getElementById('widget');
  const height = widget.scrollHeight;
  try {
    await getCurrentWindow().setSize(new LogicalSize(WIDGET_WIDTH, height));
  } catch (e) {
    // ignore resize errors
  }
}

async function update() {
  try {
    await Promise.all([updateCpu(), updateMemory(), updateGpu(), updateProcesses()]);
    await resizeToContent();
  } catch (e) {
    console.error('Update failed:', e);
  }
}

// Collapsible sections
document.querySelectorAll('.section-header').forEach(header => {
  header.addEventListener('click', () => {
    header.closest('.section').classList.toggle('collapsed');
    setTimeout(resizeToContent, 350);
  });
});

// Sort tabs
document.querySelectorAll('.sort-tab').forEach(tab => {
  tab.addEventListener('click', (e) => {
    e.stopPropagation();
    document.querySelectorAll('.sort-tab').forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    currentSort = tab.dataset.sort;
    updateProcesses().then(resizeToContent);
  });
});

// Close button
document.getElementById('close-btn').addEventListener('click', () => {
  getCurrentWindow().close();
});

// Start polling
update();
setInterval(update, 2000);
