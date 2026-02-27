// Tauri invoke — wird nach DOMContentLoaded initialisiert
let invoke;

async function loadSettings() {
    try {
        const settings = await invoke('get_settings');
        document.getElementById('serverUrl').value = settings.server_url || '';
        document.getElementById('apiKey').value = settings.api_key || '';
        document.getElementById('pollInterval').value = settings.poll_interval_seconds || 5;
        document.getElementById('autoStart').checked = settings.auto_start !== false;
        await refreshPrinters(settings.printer_name);
    } catch (err) {
        console.error('Einstellungen laden fehlgeschlagen:', err);
    }
}

async function refreshPrinters(selectedPrinter) {
    const sel = document.getElementById('printerSelect');
    try {
        const printers = await invoke('list_printers');
        if (!printers.length) {
            sel.innerHTML = '<option value="">Keine Drucker gefunden</option>';
            return;
        }
        sel.innerHTML = printers.map(p =>
            `<option value="${p}" ${p === selectedPrinter ? 'selected' : ''}>${p.replace(/_/g, ' ')}</option>`
        ).join('');
    } catch (err) {
        console.error('Drucker laden fehlgeschlagen:', err);
        sel.innerHTML = `<option value="">Fehler: ${err}</option>`;
    }
}

async function testConnection() {
    const statusEl = document.getElementById('connectionStatus');
    const url = document.getElementById('serverUrl').value.trim();
    const key = document.getElementById('apiKey').value.trim();

    if (!url || !key) {
        statusEl.innerHTML = '<span class="status-dot unconfigured"></span>URL und Key eingeben';
        return;
    }

    statusEl.innerHTML = '<span class="status-dot unconfigured"></span>Teste...';

    try {
        const ok = await invoke('test_connection', { serverUrl: url, apiKey: key });
        if (ok) {
            statusEl.innerHTML = '<span class="status-dot connected"></span>Verbunden';
        } else {
            statusEl.innerHTML = '<span class="status-dot disconnected"></span>Authentifizierung fehlgeschlagen';
        }
    } catch (err) {
        statusEl.innerHTML = `<span class="status-dot disconnected"></span>${err}`;
    }
}

async function saveAll() {
    const settings = {
        server_url: document.getElementById('serverUrl').value.trim(),
        api_key: document.getElementById('apiKey').value.trim(),
        printer_name: document.getElementById('printerSelect').value,
        poll_interval_seconds: parseInt(document.getElementById('pollInterval').value) || 5,
        auto_start: document.getElementById('autoStart').checked,
        agent_id: '',
    };

    try {
        const current = await invoke('get_settings');
        settings.agent_id = current.agent_id || '';

        await invoke('save_settings', { settings });

        if (settings.auto_start) {
            await invoke('plugin:autostart|enable');
        } else {
            await invoke('plugin:autostart|disable');
        }

        const btn = document.querySelector('.footer .btn-primary');
        btn.style.background = '#34c759';
        btn.textContent = 'Gespeichert!';
        setTimeout(() => {
            btn.style.background = '';
            btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg> Speichern';
        }, 1500);
    } catch (err) {
        alert('Fehler beim Speichern: ' + err);
    }
}

// Initialisierung: warten bis Tauri API verfügbar ist
function init() {
    if (window.__TAURI__ && window.__TAURI__.core) {
        invoke = window.__TAURI__.core.invoke;
        loadSettings();
    } else {
        setTimeout(init, 100);
    }
}

document.addEventListener('DOMContentLoaded', init);
