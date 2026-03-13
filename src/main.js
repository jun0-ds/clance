const { invoke } = window.__TAURI__.core;
const { getCurrentWindow, currentMonitor } = window.__TAURI__.window;
const { LogicalSize, LogicalPosition } = window.__TAURI__.dpi;

const WIDGET_WIDTH = 300;
const SNAP_DISTANCE = 20;
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

// Opacity slider
const opacitySlider = document.getElementById('opacity-slider');
opacitySlider.addEventListener('input', () => {
  const val = opacitySlider.value / 100;
  document.getElementById('widget').style.background = `rgba(20, 20, 30, ${val})`;
});

// Minimize to tray with animation
document.getElementById('minimize-btn').addEventListener('click', async () => {
  const widget = document.getElementById('widget');
  widget.style.transition = 'opacity 0.2s ease, transform 0.2s ease';
  widget.style.opacity = '0';
  widget.style.transform = 'scale(0.8) translateY(20px)';
  setTimeout(async () => {
    await getCurrentWindow().hide();
    widget.style.transition = 'none';
    widget.style.opacity = '1';
    widget.style.transform = '';
  }, 200);
});

// Close (quit app)
document.getElementById('close-btn').addEventListener('click', () => {
  getCurrentWindow().close();
});

// Edge snapping
let snapTimeout = null;
getCurrentWindow().listen('tauri://move', () => {
  if (snapTimeout) clearTimeout(snapTimeout);
  snapTimeout = setTimeout(snapToEdge, 50);
});

async function snapToEdge() {
  try {
    const win = getCurrentWindow();
    const monitor = await currentMonitor();
    if (!monitor) return;

    const pos = await win.outerPosition();
    const size = await win.outerSize();
    const scale = monitor.scaleFactor;

    // Monitor work area (logical pixels)
    const mx = monitor.position.x / scale;
    const my = monitor.position.y / scale;
    const mw = monitor.size.width / scale;
    const mh = monitor.size.height / scale;

    // Current position (physical → logical)
    let x = pos.x / scale;
    let y = pos.y / scale;
    const w = size.width / scale;
    const h = size.height / scale;

    let snapped = false;

    // Snap to left edge
    if (Math.abs(x - mx) < SNAP_DISTANCE) { x = mx; snapped = true; }
    // Snap to right edge
    if (Math.abs((x + w) - (mx + mw)) < SNAP_DISTANCE) { x = mx + mw - w; snapped = true; }
    // Snap to top edge
    if (Math.abs(y - my) < SNAP_DISTANCE) { y = my; snapped = true; }
    // Snap to bottom edge
    if (Math.abs((y + h) - (mx + mh)) < SNAP_DISTANCE) { y = my + mh - h; snapped = true; }

    if (snapped) {
      await win.setPosition(new LogicalPosition(x, y));
    }
  } catch (e) {
    // ignore snap errors
  }
}

// Start polling
update();
setInterval(update, 2000);
