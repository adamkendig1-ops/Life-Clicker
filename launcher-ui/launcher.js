/* Browser-local saves stay in this browser and origin. Only explicit Restore writes them. */
(() => {
  'use strict';
  const $ = id => document.getElementById(id);
  const frame = $('gameFrame');
  let devMode = false;
  let onlineInfo = null;
  const saveKey = /^(LC4_|lifeClickerV3|LC_PREUPDATE_|LC_PREMIGRATION_)/;
  const esc = value => String(value).replace(/[&<>"']/g, c => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'
  })[c]);

  async function api(path, options = {}) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 90000);
    try {
      const response = await fetch(path, { ...options, cache: 'no-store', signal: controller.signal });
      const data = await response.json();
      if (!response.ok) throw new Error(data.error || `HTTP ${response.status}`);
      return data;
    } finally { clearTimeout(timer); }
  }
  const post = (path, body = {}) => api(path, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body)
  });
  const open = id => $(id).classList.add('open');
  const close = id => $(id).classList.remove('open');
  function play() {
    if (!frame.src.includes('/game/')) frame.src = '/game/index.html';
    $('welcome').classList.add('off'); frame.classList.add('on');
  }
  function snapshot() {
    // Read only: the game's existing autosave owns all normal browser-save writes.
    const data = {};
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (saveKey.test(key)) data[key] = localStorage.getItem(key);
    }
    return { created: new Date().toISOString(), data };
  }
  const backup = () => post('/api/backup-saves', snapshot());
  async function loadVersion() {
    const v = await api('/api/version');
    devMode = v.devMode === true;
    $('gameVersion').textContent = `v${v.version}`;
    $('saveVersion').textContent = `Save Version ${v.saveVersion}`;
    $('versionLine').textContent = `${devMode ? 'DEV MODE · ' : ''}Game ${v.version} · Launcher ${v.launcherVersion} · Save Version ${v.saveVersion}`;
    if (devMode) {
      document.title = 'DEV MODE — Life Clicker';
      document.querySelector('.brand').textContent = 'Life Clicker — DEV MODE';
      document.querySelector('.status b').textContent = 'DEV MODE · isolated saves';
      document.querySelector('.side code').textContent = '127.0.0.1:8766';
      $('installUpdate').disabled = true; $('onlineInstall').disabled = true;
      $('updateMsg').textContent = 'DEV MODE: edit game/index.html directly. Installation disabled.';
    }
    return v;
  }
  async function backups() {
    const box = $('backupList'); box.replaceChildren();
    try {
      const result = await api('/api/backups');
      if (!result.backups.length) { box.textContent = 'No backups yet.'; return; }
      for (const entry of result.backups) {
        const row = document.createElement('div'); row.className = 'row';
        const info = document.createElement('div');
        info.textContent = `${entry.name} · ${entry.size.toLocaleString()} bytes`;
        const button = document.createElement('button'); button.className = 'btn'; button.textContent = 'Restore';
        button.onclick = async () => {
          if (!confirm('Restore this save backup? A fresh backup will be created first.')) return;
          try {
            await backup();
            const saved = await api(`/api/backup?name=${encodeURIComponent(entry.name)}`);
            if (!saved.data || Object.values(saved.data).some(v => typeof v !== 'string')) throw new Error('Invalid save snapshot');
            // Stop the old game before restoring so its autosave cannot overwrite restored data.
            frame.src = 'about:blank';
            await new Promise(resolve => frame.addEventListener('load', resolve, { once: true }));
            for (const key of Object.keys(localStorage)) if (saveKey.test(key)) localStorage.removeItem(key);
            for (const [key, value] of Object.entries(saved.data)) if (saveKey.test(key)) localStorage.setItem(key, value);
            $('recoveryMsg').textContent = 'Restored. Reloading game…';
            frame.src = `/game/index.html?restore=${Date.now()}`; play();
          } catch (error) { $('recoveryMsg').textContent = error.message; }
        };
        row.append(info, button); box.append(row);
      }
    } catch (error) { box.textContent = error.message; }
  }
  function showOnline(result, manual) {
    onlineInfo = result;
    $('onlineInstall').hidden = !result.available || devMode;
    if (!result.configured) {
      $('onlineStatus').textContent = devMode ? 'DEV MODE: online installation disabled' : 'Online feed not configured';
      $('onlineText').textContent = $('onlineStatus').textContent;
    } else if (result.error) {
      $('onlineStatus').textContent = `Update check failed: ${result.error}`;
      $('onlineText').textContent = $('onlineStatus').textContent;
    } else if (!result.available) {
      $('onlineStatus').textContent = 'Up to date'; $('onlineText').textContent = 'Life Clicker is up to date.';
    } else {
      $('onlineStatus').textContent = 'Update Available';
      $('onlineText').innerHTML = `<p><b>Update Available</b></p>` +
        (result.gameAvailable ? `<p>Game ${esc(result.currentGameVersion)} → ${esc(result.gameVersion)}</p>` : '') +
        (result.launcherAvailable ? `<p>Launcher ${esc(result.launcherVersion)} (next startup)</p>` : '') +
        `<ul>${(result.releaseNotes || []).map(n => `<li>${esc(n)}</li>`).join('')}</ul>` +
        (result.mandatory ? '<p>This release is marked mandatory.</p>' : '');
    }
    if (manual || result.available) open('onlineOverlay');
  }
  async function checkOnline(force = false) {
    try { showOnline(await api(`/api/online-update${force ? '?force=1' : ''}`), force); }
    catch (error) { showOnline({ configured: true, error: error.message }, force); }
  }
  function installationResult(result) {
    let text = result.gameInstalled ? `Installed game ${result.version || ''}.` : 'Update completed.';
    if (result.launcherStaged) text += ' Launcher replacement staged; close and reopen Life Clicker.';
    if (result.launcherStageError) text += ` Game installed, but launcher staging failed: ${result.launcherStageError}`;
    return text;
  }
  document.querySelectorAll('[data-close]').forEach(button => { button.onclick = () => close(button.dataset.close); });
  for (const id of ['playTop', 'playSide', 'heroPlay']) $(id).onclick = play;
  for (const id of ['backupTop', 'backupSide', 'heroBackup']) $(id).onclick = async () => {
    try { alert(`Backup created: ${(await backup()).name}`); } catch (error) { alert(`Backup failed: ${error.message}`); }
  };
  for (const id of ['updatesTop', 'updateSide']) $(id).onclick = () => open('updateOverlay');
  $('recoverySide').onclick = () => { open('recoveryOverlay'); backups(); };
  $('refreshBackups').onclick = backups;
  $('newBackup').onclick = async () => {
    try { $('recoveryMsg').textContent = `Created ${(await backup()).name}`; await backups(); }
    catch (error) { $('recoveryMsg').textContent = error.message; }
  };
  $('installUpdate').onclick = async () => {
    const button = $('installUpdate');
    try {
      if (devMode) throw new Error('Installation disabled in DEV MODE');
      const file = $('updateFile').files[0]; if (!file) throw new Error('Choose a .lcupdate file first.');
      button.disabled = true;
      const pkg = JSON.parse(await file.text());
      $('updateMsg').textContent = 'Validating package, backing up saves and installing…';
      const result = await post('/api/apply-update', { ...pkg, saveSnapshot: snapshot() });
      $('updateMsg').textContent = installationResult(result); await loadVersion();
      frame.src = `/game/index.html?update=${Date.now()}`; play();
    } catch (error) { $('updateMsg').textContent = error.message; }
    finally { button.disabled = devMode; }
  };
  $('checkOnlineSide').onclick = () => checkOnline(true);
  $('onlineInstall').onclick = async () => {
    if (!onlineInfo?.available || devMode) return;
    $('onlineInstall').disabled = true;
    try {
      $('onlineMsg').textContent = 'Downloading, validating, backing up saves and installing…';
      const result = await post('/api/install-online-update', { saveSnapshot: snapshot() });
      $('onlineMsg').textContent = installationResult(result); await loadVersion();
      if (result.gameInstalled) { frame.src = `/game/index.html?onlineUpdate=${Date.now()}`; play(); }
    } catch (error) { $('onlineMsg').textContent = error.message; }
    finally { $('onlineInstall').disabled = devMode; }
  };
  $('integritySide').onclick = async () => {
    open('integrityOverlay'); $('integrityText').textContent = 'Checking…';
    try {
      const result = await api('/api/integrity');
      const describe = item => item.exists ? `${item.bytes.toLocaleString()} bytes · ${item.sha256}` : 'missing';
      $('integrityText').textContent = `${result.ok ? 'PASS' : 'FAILED'}\nGame: ${describe(result.game)}\nLauncher UI: ${describe(result.launcher)}`;
    } catch (error) { $('integrityText').textContent = error.message; }
  };
  $('dataSide').onclick = () => post('/api/open-folder?target=userdata').catch(error => alert(error.message));
  $('exitTop').onclick = async () => {
    try { await post('/api/shutdown'); document.body.textContent = 'Life Clicker closed. You can close this tab.'; }
    catch (error) { alert(error.message); }
  };
  window.addEventListener('message', event => {
    if (event.origin === location.origin && event.source === frame.contentWindow && event.data?.type === 'LC_REQUEST_UPDATE') open('updateOverlay');
  });
  frame.addEventListener('load', () => {
    if (frame.src.includes('/game/')) setTimeout(() => backup().catch(error => { $('recoveryMsg').textContent = `Automatic backup failed: ${error.message}`; }), 1200);
  });
  loadVersion().then(() => checkOnline(false)).catch(error => {
    $('versionLine').textContent = `Launcher initialization failed: ${error.message}`;
    $('onlineStatus').textContent = 'Startup failed; reload after resolving the error above.';
  });
})();
