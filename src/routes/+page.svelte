<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";

  interface Device {
    serial: string;
    status: string;
  }

  let devices = $state<Device[]>([]);
  let selectedSerial = $state("");
  let destPath = $state("/sdcard/Download/");
  let isConnected = $derived(devices.some(d => d.serial === selectedSerial && d.status === 'device'));
  
  let transferStatus = $state("Drag files here");
  let isProcessing = $state(false);
  let dropActive = $state(false);

  // Browser Modal State
  let showBrowser = $state(false);
  let browserPath = $state("");
  let browserItems = $state<string[]>([]);
  let browserLoading = $state(false);

  async function updateDeviceStatus() {
    try {
      const list: Device[] = await invoke("list_devices");
      devices = list;
      
      const physicalDevices = devices.filter(d => !d.serial.startsWith("emulator-"));
      
      if (physicalDevices.length > 0) {
        if (!selectedSerial || !devices.some(d => d.serial === selectedSerial)) {
          selectedSerial = physicalDevices[0].serial;
        }
      } else if (devices.length > 0) {
        if (!selectedSerial || !devices.some(d => d.serial === selectedSerial)) {
          selectedSerial = devices[0].serial;
        }
      } else {
        selectedSerial = "";
      }
    } catch (e) {
      console.error(e);
    }
  }

  onMount(() => {
    updateDeviceStatus();
    const interval = setInterval(updateDeviceStatus, 3000);

    const unlisten = getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === 'hover') {
        dropActive = true;
      } else if (event.payload.type === 'drop') {
        dropActive = false;
        handleDrop(event.payload.paths);
      } else {
        dropActive = false;
      }
    });

    return () => {
      clearInterval(interval);
      unlisten.then(f => f());
    };
  });

  async function selectFiles() {
    if (!isConnected) {
      transferStatus = "Connect a device first!";
      return;
    }
    
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });
      
      if (selected && Array.isArray(selected)) {
        handleDrop(selected);
      } else if (selected) {
        handleDrop([selected as string]);
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function openBrowser() {
    if (!isConnected) {
      transferStatus = "Connect a device first!";
      return;
    }
    showBrowser = true;
    browserPath = destPath.startsWith('/') ? destPath : '/sdcard/';
    if (!browserPath.endsWith('/')) {
        // Find last slash to get parent dir
        browserPath = browserPath.substring(0, browserPath.lastIndexOf('/') + 1);
    }
    await loadBrowserDir(browserPath);
  }

  async function loadBrowserDir(path: string) {
    browserLoading = true;
    try {
      browserItems = await invoke("list_remote_dirs", { serial: selectedSerial, path });
      browserPath = path;
    } catch (e) {
      console.error(e);
      // fallback to root or sdcard if path fails
      if (path !== "/sdcard/") {
          await loadBrowserDir("/sdcard/");
      }
    } finally {
      browserLoading = false;
    }
  }

  async function navigateBrowser(item: string) {
    let newPath = "";
    if (item === "..") {
      const parts = browserPath.split('/').filter(p => p !== "");
      parts.pop();
      newPath = "/" + parts.join("/");
      if (!newPath.endsWith("/")) newPath += "/";
    } else {
      newPath = browserPath + item;
    }
    await loadBrowserDir(newPath);
  }

  function selectBrowserPath() {
    destPath = browserPath;
    showBrowser = false;
  }

  async function handleDrop(paths: string[]) {
    if (!isConnected) {
      transferStatus = "Connect a device first!";
      return;
    }
    if (isProcessing) return;

    isProcessing = true;
    
    try {
      for (const path of paths) {
        const fileName = path.split('/').pop();
        if (path.toLowerCase().endsWith(".apk")) {
          transferStatus = `Installing ${fileName}...`;
          await invoke("install_apk", { serial: selectedSerial, path });
        } else {
          transferStatus = `Pushing ${fileName}...`;
          let finalDest = destPath;
          if (!finalDest.endsWith('/')) finalDest += '/';
          
          await invoke("push_files", { serial: selectedSerial, sourcePaths: [path], destPath: finalDest });
        }
      }
      transferStatus = "Success!";
    } catch (e) {
      transferStatus = "Error: " + e;
    } finally {
      isProcessing = false;
      setTimeout(() => {
        if (!isProcessing) transferStatus = "Drag files here";
      }, 3000);
    }
  }
</script>

<main class="container">
  <div class="header-controls">
    <div class="status-bar">
      <div class="led" class:connected={isConnected}></div>
      <select bind:value={selectedSerial} class="device-select">
        {#each devices as device}
          <option value={device.serial}>{device.serial} ({device.status})</option>
        {/each}
        {#if devices.length === 0}
          <option value="">No Device Found</option>
        {/if}
      </select>
    </div>

    <div class="path-input-container">
      <div class="path-header">
        <label for="path-input">Destination:</label>
        <button class="browse-btn" onclick={openBrowser}>Browse Device</button>
      </div>
      <input 
        id="path-input" 
        type="text" 
        bind:value={destPath} 
        placeholder="/sdcard/Download/"
        class="path-input"
      />
    </div>
  </div>

  <div 
    class="drop-zone" 
    class:active={dropActive} 
    class:processing={isProcessing}
    class:error={transferStatus.startsWith('Error') || transferStatus.includes('Connect')}
  >
    <div class="icon">
      {#if isProcessing}
        <div class="spinner"></div>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
      {/if}
    </div>
    <p class="transfer-status">{transferStatus}</p>
    
    {#if !isProcessing}
      <button class="select-btn" onclick={selectFiles}>Select Files</button>
    {/if}
  </div>
  
  <p class="hint">APKs will be installed regardless of path.<br/>Folders will have Mac junk files removed.</p>

  {#if showBrowser}
    <div class="modal-backdrop" onclick={() => showBrowser = false} onkeydown={(e) => e.key === 'Escape' && (showBrowser = false)} role="presentation">
      <div class="modal" onclick={(e) => e.stopPropagation()} role="presentation">
        <div class="modal-header">
          <h3>Browse Handheld</h3>
          <button class="close-btn" onclick={() => showBrowser = false}>&times;</button>
        </div>
        <div class="modal-path">{browserPath}</div>
        <div class="modal-content">
          {#if browserLoading}
            <div class="browser-loading">Loading...</div>
          {:else}
            <ul class="browser-list">
              {#each browserItems as item}
                <li>
                  <button class="browser-item" onclick={() => navigateBrowser(item)}>
                    <span class="folder-icon">📁</span> {item}
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
        <div class="modal-footer">
          <button class="confirm-btn" onclick={selectBrowserPath}>Select This Folder</button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background-color: #1a1a1a;
    color: #e0e0e0;
    user-select: none;
  }

  .container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 10px;
    box-sizing: border-box;
    gap: 15px;
    position: relative;
  }

  .header-controls {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 100%;
    max-width: 320px;
  }

  .status-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #2a2a2a;
    padding: 6px 12px;
    border-radius: 12px;
  }

  .device-select {
    background: transparent;
    color: #e0e0e0;
    border: none;
    font-size: 0.85rem;
    font-weight: 500;
    outline: none;
    flex-grow: 1;
  }

  .path-input-container {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: #2a2a2a;
    padding: 8px 12px;
    border-radius: 12px;
  }

  .path-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .path-header label {
    font-size: 0.7rem;
    color: #888;
    text-transform: uppercase;
    font-weight: 700;
  }

  .browse-btn {
    background: transparent;
    color: #3498db;
    border: none;
    font-size: 0.7rem;
    font-weight: 600;
    padding: 0;
    cursor: pointer;
    text-transform: uppercase;
  }

  .path-input {
    background: transparent;
    border: none;
    color: #3498db;
    font-size: 0.85rem;
    font-family: monospace;
    outline: none;
    padding: 0;
    width: 100%;
  }

  .led {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ff4b2b;
    box-shadow: 0 0 5px #ff4b2b;
    flex-shrink: 0;
  }

  .led.connected {
    background: #2ecc71;
    box-shadow: 0 0 8px #2ecc71;
  }

  .drop-zone {
    width: 240px;
    height: 240px;
    border: 2px dashed #444;
    border-radius: 32px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    background: #252525;
    text-align: center;
    padding: 20px;
  }

  .drop-zone.active {
    border-color: #3498db;
    background: #2c3e50;
    transform: scale(1.02);
  }

  .drop-zone.processing {
    border-style: solid;
    border-color: #f1c40f;
  }

  .drop-zone.error {
    border-color: #e74c3c;
  }

  .icon {
    color: #888;
  }

  .active .icon {
    color: #3498db;
  }

  .transfer-status {
    font-size: 0.95rem;
    font-weight: 500;
    margin: 0;
    word-break: break-all;
  }

  .select-btn {
    background: #3498db;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }

  .hint {
    font-size: 0.7rem;
    color: #666;
    text-align: center;
    line-height: 1.4;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255,255,255,0.1);
    border-top-color: #f1c40f;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  /* Modal Styles */
  .modal-backdrop {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0,0,0,0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    border-radius: inherit;
  }

  .modal {
    background: #252525;
    width: 90%;
    height: 80%;
    border-radius: 20px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  }

  .modal-header {
    padding: 15px;
    background: #2a2a2a;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: #888;
    font-size: 1.5rem;
    cursor: pointer;
  }

  .modal-path {
    padding: 10px 15px;
    background: #1a1a1a;
    font-size: 0.75rem;
    color: #3498db;
    font-family: monospace;
    word-break: break-all;
  }

  .modal-content {
    flex-grow: 1;
    overflow-y: auto;
    padding: 5px;
  }

  .browser-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .browser-item {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: #e0e0e0;
    padding: 10px 15px;
    font-size: 0.9rem;
    cursor: pointer;
    border-bottom: 1px solid #2a2a2a;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .browser-item:hover {
    background: #3498db22;
  }

  .folder-icon {
    font-size: 1.1rem;
  }

  .modal-footer {
    padding: 15px;
    background: #2a2a2a;
    display: flex;
    justify-content: center;
  }

  .confirm-btn {
    background: #2ecc71;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 10px;
    font-weight: 600;
    cursor: pointer;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
