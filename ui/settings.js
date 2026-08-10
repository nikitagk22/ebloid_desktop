(() => {
  const invoke = window.__TAURI__.core.invoke;
  const listen = window.__TAURI__.event.listen;
  let settings;
  let updateAvailable = false;
  let installing = false;

  const elements = {
    notifications: document.getElementById("notifications"),
    autostart: document.getElementById("autostart"),
    minimizeToTray: document.getElementById("minimizeToTray"),
    checkUpdates: document.getElementById("checkUpdates"),
    zoom: document.getElementById("zoom"),
    zoomValue: document.getElementById("zoomValue"),
    status: document.getElementById("generalStatus"),
  };

  const showStatus = (text, error = false) => {
    elements.status.textContent = text;
    elements.status.classList.toggle("error", error);
  };

  const setSection = (section) => {
    const target = document.getElementById(section) ? section : "general";
    document.querySelectorAll(".panel").forEach((panel) => panel.classList.toggle("active", panel.id === target));
    document.querySelectorAll(".nav-button").forEach((button) => button.setAttribute("aria-selected", String(button.dataset.section === target)));
    history.replaceState(null, "", `#${target}`);
  };

  document.querySelectorAll(".nav-button").forEach((button) => button.addEventListener("click", () => setSection(button.dataset.section)));
  window.addEventListener("ebloid-section", (event) => setSection(event.detail));

  function renderSettings() {
    for (const key of ["notifications", "autostart", "minimizeToTray", "checkUpdates"]) {
      elements[key].setAttribute("aria-checked", String(Boolean(settings[key])));
    }
    const zoom = Math.round(settings.zoom * 100);
    elements.zoom.value = String(zoom);
    elements.zoomValue.textContent = `${zoom}%`;
  }

  async function saveSettings(message = "Настройки сохранены") {
    try {
      settings = await invoke("update_settings", { settings });
      renderSettings();
      showStatus(message);
    } catch (error) {
      showStatus(String(error), true);
      throw error;
    }
  }

  for (const key of ["notifications", "autostart", "minimizeToTray", "checkUpdates"]) {
    elements[key].addEventListener("click", async () => {
      settings[key] = !settings[key];
      if (key === "notifications" && settings[key] && "Notification" in window && Notification.permission === "default") {
        await Notification.requestPermission().catch(() => {});
      }
      renderSettings();
      await saveSettings().catch(() => {});
    });
  }

  let zoomTimer;
  elements.zoom.addEventListener("input", () => {
    const value = Number(elements.zoom.value);
    elements.zoomValue.textContent = `${value}%`;
    settings.zoom = value / 100;
    clearTimeout(zoomTimer);
    zoomTimer = setTimeout(() => saveSettings(`Масштаб: ${value}%`).catch(() => {}), 140);
  });

  const confirmationTimers = new WeakMap();
  function confirmAction(button, confirmationText, statusText) {
    if (button.dataset.confirming === "true") {
      clearTimeout(confirmationTimers.get(button));
      button.dataset.confirming = "false";
      return true;
    }

    const originalText = button.textContent;
    button.dataset.confirming = "true";
    button.textContent = confirmationText;
    showStatus(statusText);
    confirmationTimers.set(button, setTimeout(() => {
      button.dataset.confirming = "false";
      button.textContent = originalText;
      showStatus("");
    }, 8000));
    return false;
  }

  document.getElementById("clearCache").addEventListener("click", async (event) => {
    const button = event.currentTarget;
    if (!confirmAction(button, "Подтвердить очистку", "Нажмите ещё раз, чтобы очистить кэш")) return;
    button.disabled = true;
    button.textContent = "Очищаем…";
    showStatus("Очищаем кэш…");
    try {
      await invoke("clear_cache");
      showStatus("Кэш очищен");
    } catch (error) {
      showStatus(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Очистить";
    }
  });

  document.getElementById("logout").addEventListener("click", async (event) => {
    const button = event.currentTarget;
    if (!confirmAction(button, "Подтвердить выход", "Нажмите ещё раз, чтобы выйти и удалить cookies")) return;
    button.disabled = true;
    button.textContent = "Выходим…";
    showStatus("Удаляем cookies и локальные данные…");
    try {
      await invoke("logout_and_clear_cookies");
      showStatus("Вы вышли из аккаунта");
    } catch (error) {
      showStatus(String(error), true);
    } finally {
      button.disabled = false;
      button.textContent = "Выйти";
    }
  });

  const formatBytes = (bytes) => {
    if (!Number.isFinite(bytes) || bytes <= 0) return "0 Б";
    const units = ["Б", "КБ", "МБ", "ГБ"];
    const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    return `${(bytes / 1024 ** index).toFixed(index ? 1 : 0)} ${units[index]}`;
  };

  const statusLabel = (item) => ({ queued: "Ожидает", downloading: "Скачивается", completed: "Готово", failed: "Ошибка", cancelled: "Отменено" })[item.status] || item.status;

  function renderDownloads(downloads) {
    const list = document.getElementById("downloadList");
    const summary = document.getElementById("downloadsSummary");
    summary.textContent = downloads.length ? `${downloads.length} ${downloads.length === 1 ? "файл" : "файлов"}` : "Нет загрузок";
    if (!downloads.length) {
      list.innerHTML = `<div class="empty"><svg viewBox="0 0 48 48"><path d="M24 7v23m0 0 8-8m-8 8-8-8M10 39h28"/></svg><strong>Загрузок пока нет</strong><span>Скачанные файлы появятся здесь</span></div>`;
      return;
    }
    list.innerHTML = downloads.map((item) => {
      const percent = item.totalBytes ? Math.min(100, item.downloadedBytes / item.totalBytes * 100) : (item.status === "completed" ? 100 : 12);
      const details = item.totalBytes ? `${formatBytes(item.downloadedBytes)} из ${formatBytes(item.totalBytes)}` : formatBytes(item.downloadedBytes);
      const action = ["queued", "downloading"].includes(item.status)
        ? `<button class="button" data-action="cancel" data-id="${item.id}">Отменить</button>`
        : item.status === "completed" ? `<button class="button" data-action="reveal" data-id="${item.id}">Показать в папке</button>` : "";
      return `<article class="download"><div><div class="download-name" title="${escapeHtml(item.destination)}">${escapeHtml(item.fileName)}</div><div class="download-meta"><span>${statusLabel(item)}</span><span>${details}</span>${item.error ? `<span>${escapeHtml(item.error)}</span>` : ""}</div><div class="download-progress"><span style="width:${percent}%"></span></div></div><div class="download-actions">${action}</div></article>`;
    }).join("");
  }

  function escapeHtml(value) {
    const element = document.createElement("span");
    element.textContent = String(value || "");
    return element.innerHTML;
  }

  document.getElementById("downloadList").addEventListener("click", async (event) => {
    const button = event.target.closest("button[data-action]");
    if (!button) return;
    button.disabled = true;
    const command = button.dataset.action === "cancel" ? "cancel_download" : "reveal_download";
    try {
      await invoke(command, { id: button.dataset.id });
    } catch (error) {
      button.textContent = String(error);
    } finally {
      button.disabled = false;
    }
  });

  document.getElementById("clearHistory").addEventListener("click", () => invoke("clear_download_history").catch(() => {}));

  async function checkUpdate() {
    const button = document.getElementById("updateButton");
    const status = document.getElementById("updateStatus");
    const notes = document.getElementById("updateNotes");
    if (updateAvailable) return installUpdate();
    button.disabled = true;
    status.textContent = "Проверяем новую версию…";
    try {
      const info = await invoke("check_for_update");
      document.getElementById("version").textContent = `Ebloid ${info.currentVersion}`;
      updateAvailable = info.available;
      if (info.available) {
        status.textContent = `Доступна версия ${info.version}`;
        button.textContent = "Установить";
        notes.textContent = info.notes || "Новая версия готова к установке.";
      } else {
        status.textContent = "У вас последняя версия";
        button.textContent = "Проверить снова";
        notes.textContent = "";
      }
    } catch (error) {
      const rawError = String(error);
      const friendlyError = rawError.includes("valid release JSON")
        ? "Релиз обновления ещё не опубликован. Попробуйте немного позже."
        : rawError;
      status.textContent = `Не удалось проверить: ${friendlyError}`;
      button.textContent = "Повторить";
    } finally {
      button.disabled = false;
    }
  }

  async function installUpdate() {
    if (installing) return;
    installing = true;
    const button = document.getElementById("updateButton");
    const status = document.getElementById("updateStatus");
    button.disabled = true;
    button.textContent = "Устанавливаем…";
    status.textContent = "Скачиваем подписанное обновление";
    try {
      await invoke("install_update");
    } catch (error) {
      installing = false;
      button.disabled = false;
      button.textContent = "Повторить";
      status.textContent = `Ошибка установки: ${error}`;
    }
  }

  document.getElementById("updateButton").addEventListener("click", checkUpdate);
  listen("downloads-changed", (event) => renderDownloads(event.payload));
  listen("update-progress", (event) => {
    const status = document.getElementById("updateStatus");
    const total = event.payload.totalBytes;
    status.textContent = total ? `Скачано ${formatBytes(event.payload.chunkBytes)} из ${formatBytes(total)}` : `Скачано ${formatBytes(event.payload.chunkBytes)}`;
  });

  async function init() {
    setSection(location.hash.slice(1) || "general");
    try {
      settings = await invoke("get_settings");
      renderSettings();
      renderDownloads(await invoke("get_downloads"));
      checkUpdate();
    } catch (error) {
      const message = `Не удалось связаться с клиентом: ${error}`;
      showStatus(message, true);
      const summary = document.getElementById("downloadsSummary");
      const list = document.getElementById("downloadList");
      if (summary) summary.textContent = "Ошибка клиента";
      if (list) list.innerHTML = `<div class="empty"><strong>Загрузки недоступны</strong><span>${escapeHtml(message)}</span></div>`;
      throw error;
    }
  }

  init().catch(() => {});
})();
