// Pisual IPC Documentation - Clean Minimalist Engine

document.addEventListener('DOMContentLoaded', () => {
  initPlayground();
  initCopy();
  initSearch();
  initMobileMenu();
  initScrollSpy();
});

// Understated toast notification
function showToast(text) {
  let toast = document.querySelector('.toast-box');
  if (toast) toast.remove();

  toast = document.createElement('div');
  toast.className = 'toast-box';
  toast.textContent = text;
  document.body.appendChild(toast);

  setTimeout(() => {
    toast.remove();
  }, 2000);
}

// Copy to clipboard
function initCopy() {
  document.querySelectorAll('.copy-btn').forEach(btn => {
    btn.addEventListener('click', async () => {
      const code = btn.closest('.code-block')?.querySelector('code');
      if (!code) return;

      try {
        await navigator.clipboard.writeText(code.innerText);
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        showToast('Copied to clipboard');
        setTimeout(() => {
          btn.textContent = originalText;
        }, 1800);
      } catch (e) {
        showToast('Failed to copy');
      }
    });
  });
}

// Playground / Request Builder
function initPlayground() {
  const actionSel = document.getElementById('pg-action');
  if (!actionSel) return;

  const idInput = document.getElementById('pg-id');
  const worldInput = document.getElementById('pg-world');
  const xInput = document.getElementById('pg-x');
  const yInput = document.getElementById('pg-y');
  const zInput = document.getElementById('pg-z');
  const linesInput = document.getElementById('pg-lines');
  const billboardSel = document.getElementById('pg-billboard');
  const shadowCheck = document.getElementById('pg-shadow');
  const seeThroughCheck = document.getElementById('pg-see-through');

  const jsonPreview = document.getElementById('pg-json-preview');
  const rustPreview = document.getElementById('pg-rust-preview');
  const simulateBtn = document.getElementById('btn-simulate');
  const responseView = document.getElementById('pg-response-view');

  const boxJson = document.getElementById('box-json');
  const boxRust = document.getElementById('box-rust');
  const tabBtns = document.querySelectorAll('.pg-tab-btn');

  tabBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      tabBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      const tab = btn.getAttribute('data-tab');
      if (tab === 'json') {
        boxJson.style.display = 'block';
        boxRust.style.display = 'none';
      } else {
        boxJson.style.display = 'none';
        boxRust.style.display = 'block';
      }
    });
  });

  function updateVisibility() {
    const action = actionSel.value;
    const isCreate = action === 'create' || action === 'create_ram';
    const isEdit = action === 'edit';
    const isMove = action === 'move';
    const isList = action === 'list';

    document.getElementById('row-id').style.display = isList ? 'none' : 'block';
    document.getElementById('row-world').style.display = (isCreate || isMove) ? 'block' : 'none';
    document.getElementById('row-pos').style.display = (isCreate || isMove) ? 'block' : 'none';
    document.getElementById('row-lines').style.display = (isCreate || isEdit) ? 'block' : 'none';
    document.getElementById('row-visuals').style.display = (isCreate || isEdit) ? 'block' : 'none';
  }

  function renderPayload() {
    const action = actionSel.value;
    const payload = { action };

    if (action !== 'list') {
      payload.id = idInput.value.trim() || 'spawn_hologram';
    }

    if (action === 'create' || action === 'create_ram') {
      payload.world = worldInput.value.trim() || 'world';
      payload.position = [
        parseFloat(xInput.value) || 0.0,
        parseFloat(yInput.value) || 64.0,
        parseFloat(zInput.value) || 0.0
      ];
      payload.lines = linesInput.value.split('\n').filter(l => l.length > 0);
      if (billboardSel.value !== 'center') {
        payload.billboard = billboardSel.value;
      }
      if (!shadowCheck.checked) {
        payload.shadow = false;
      }
      if (seeThroughCheck.checked) {
        payload.see_through = true;
      }
    } else if (action === 'edit') {
      const lines = linesInput.value.split('\n').filter(l => l.length > 0);
      if (lines.length > 0) payload.lines = lines;
      if (billboardSel.value) payload.billboard = billboardSel.value;
      if (!shadowCheck.checked) payload.shadow = false;
      if (seeThroughCheck.checked) payload.see_through = true;
    } else if (action === 'move') {
      payload.position = [
        parseFloat(xInput.value) || 0.0,
        parseFloat(yInput.value) || 64.0,
        parseFloat(zInput.value) || 0.0
      ];
      if (worldInput.value.trim()) {
        payload.world = worldInput.value.trim();
      }
    }

    let jsonText = JSON.stringify(payload, null, 2);
    // Format [x, y, z] arrays inline to save vertical space
    jsonText = jsonText.replace(/\[\s*(-?\d+(\.\d+)?),\s*(-?\d+(\.\d+)?),\s*(-?\d+(\.\d+)?)\s*\]/g, '[$1, $3, $5]');
    jsonPreview.textContent = jsonText;

    const rustSnippet = `use pumpkin_plugin_api::ipc;

// 1. Serialize payload
let payload = serde_json::to_vec(&serde_json::json!(${jsonText}))
    .map_err(|e| e.to_string())?;

// 2. Dispatch to Pisual
let res_bytes = ipc::send_ipc_message("Pisual".to_string(), payload)
    .map_err(|e| format!("Transport error: {e}"))?;

let res: serde_json::Value = serde_json::from_slice(&res_bytes)
    .map_err(|e| format!("Invalid response: {e}"))?;`;

    rustPreview.textContent = rustSnippet;
  }

  const inputs = [actionSel, idInput, worldInput, xInput, yInput, zInput, linesInput, billboardSel, shadowCheck, seeThroughCheck];
  inputs.forEach(el => {
    el?.addEventListener('input', () => {
      updateVisibility();
      renderPayload();
    });
    el?.addEventListener('change', () => {
      updateVisibility();
      renderPayload();
    });
  });

  simulateBtn.addEventListener('click', () => {
    const action = actionSel.value;
    const id = idInput.value.trim() || 'spawn_hologram';
    let res = {};

    switch (action) {
      case 'create':
      case 'create_ram':
        res = { success: true, message: `hologram '${id}' created successfully` };
        break;
      case 'edit':
        res = { success: true, message: `Hologram '${id}' updated successfully` };
        break;
      case 'move':
        res = { success: true, message: `Hologram '${id}' moved successfully` };
        break;
      case 'delete':
        res = { success: true, message: `Hologram '${id}' deleted successfully` };
        break;
      case 'get':
        res = {
          success: true,
          data: {
            id: id,
            world_name: worldInput.value || 'world',
            position: [
              parseFloat(xInput.value) || 0.0,
              parseFloat(yInput.value) || 64.0,
              parseFloat(zInput.value) || 0.0
            ],
            lines: linesInput.value.split('\n').filter(l => l.length > 0),
            billboard: billboardSel.value,
            shadow: shadowCheck.checked,
            see_through: seeThroughCheck.checked,
            scale: [1.0, 1.0, 1.0]
          }
        };
        break;
      case 'list':
        res = {
          success: true,
          data: [
            {
              id: 'spawn_hologram',
              world_name: 'world',
              position: [0.0, 65.0, 0.0],
              lines: ['&6&lSERVER SPAWN', '&7Welcome to Pumpkin MC!'],
              billboard: 'vertical',
              shadow: true,
              see_through: false,
              scale: [1.0, 1.0, 1.0]
            }
          ]
        };
        break;
    }

    responseView.textContent = JSON.stringify(res, null, 2);
  });

  updateVisibility();
  renderPayload();
}

// Search
function initSearch() {
  const searchInput = document.getElementById('search-input');
  if (!searchInput) return;

  searchInput.addEventListener('input', (e) => {
    const q = e.target.value.toLowerCase().trim();
    const blocks = document.querySelectorAll('.action-block');

    blocks.forEach(b => {
      const match = b.textContent.toLowerCase().includes(q);
      b.style.display = match ? 'block' : 'none';
    });
  });
}

// Mobile sidebar menu toggle
function initMobileMenu() {
  const toggle = document.getElementById('menu-toggle');
  const sidebar = document.getElementById('sidebar');

  if (toggle && sidebar) {
    toggle.addEventListener('click', () => {
      sidebar.classList.toggle('open');
    });

    sidebar.querySelectorAll('a').forEach(link => {
      link.addEventListener('click', () => {
        if (window.innerWidth <= 820) {
          sidebar.classList.remove('open');
        }
      });
    });
  }
}

// Scrollspy for sidebar & TOC
function initScrollSpy() {
  const sections = document.querySelectorAll('section, .action-block');
  const navLinks = document.querySelectorAll('.sidebar .nav-item a');
  const tocLinks = document.querySelectorAll('.toc-list a');

  window.addEventListener('scroll', () => {
    let currentId = '';
    const top = window.scrollY + 90;

    sections.forEach(sec => {
      if (sec.offsetTop <= top) {
        currentId = sec.getAttribute('id');
      }
    });

    if (currentId) {
      navLinks.forEach(link => {
        if (link.getAttribute('href') === `#${currentId}`) {
          link.classList.add('active');
        } else {
          link.classList.remove('active');
        }
      });

      tocLinks.forEach(link => {
        if (link.getAttribute('href') === `#${currentId}`) {
          link.classList.add('active');
        } else {
          link.classList.remove('active');
        }
      });
    }
  }, { passive: true });
}
