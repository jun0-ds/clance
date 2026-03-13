const { invoke } = window.__TAURI__.core;

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
  const usedGb = (info.memory_used_mb / 1024).toFixed(1);
  const totalGb = (info.memory_total_mb / 1024).toFixed(1);
  document.getElementById('gpu-vram').textContent = 'VRAM: ' + usedGb + '/' + totalGb + 'GB';
  setBar('gpu-bar', (info.memory_used_mb / info.memory_total_mb) * 100);
}

async function updateProcesses() {
  const procs = await invoke('get_top_processes');
  const list = document.getElementById('process-list');
  list.innerHTML = procs.map((p, i) =>
    `<li><span class="proc-name">${i + 1}. ${p.name}</span><span class="proc-cpu">${p.cpu_usage.toFixed(1)}%</span></li>`
  ).join('');
}

async function update() {
  try {
    await Promise.all([updateCpu(), updateMemory(), updateGpu(), updateProcesses()]);
  } catch (e) {
    console.error('Update failed:', e);
  }
}

// Collapsible sections
document.querySelectorAll('.section-header').forEach(header => {
  header.addEventListener('click', () => {
    header.closest('.section').classList.toggle('collapsed');
  });
});

// Start polling
update();
setInterval(update, 2000);
