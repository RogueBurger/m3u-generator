import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';

const dropZone = document.getElementById('drop-zone');
const status = document.getElementById('status');

function showStatus(type, message) {
  status.className = type;
  status.textContent = message;
}

function clearStatus() {
  status.className = '';
  status.textContent = '';
}

async function handleFiles(filePaths) {
  if (!filePaths || filePaths.length === 0) return;

  clearStatus();
  dropZone.classList.add('processing');
  dropZone.classList.remove('drag-over');

  try {
    const m3uPath = await invoke('create_m3u', { filePaths });
    showStatus('success', `Created: ${m3uPath}`);
  } catch (error) {
    showStatus('error', String(error));
  } finally {
    dropZone.classList.remove('processing');
  }
}

// Click to browse
dropZone.addEventListener('click', async () => {
  if (dropZone.classList.contains('processing')) return;
  const result = await open({ multiple: true, directory: false });
  if (result) {
    const paths = Array.isArray(result) ? result : [result];
    await handleFiles(paths);
  }
});

dropZone.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    dropZone.click();
  }
});

// Prevent browser from navigating to dropped files
window.addEventListener('dragover', (e) => e.preventDefault());
window.addEventListener('drop', (e) => e.preventDefault());

// Tauri window drag events for visual feedback and file handling
(async () => {
  await listen('tauri://drag-enter', () => dropZone.classList.add('drag-over'));
  await listen('tauri://drag-leave', () => dropZone.classList.remove('drag-over'));
  await listen('tauri://drag-drop', (event) => {
    const paths = event.payload.paths;
    if (paths && paths.length > 0) {
      handleFiles(paths);
    } else {
      dropZone.classList.remove('drag-over');
    }
  });
})();
