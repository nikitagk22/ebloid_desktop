(() => {
  if (window.__ebloidDesktopLoaded) return;
  window.__ebloidDesktopLoaded = true;

  const invoke = (command, args = {}) => {
    const call = window.__TAURI_INTERNALS__?.invoke || window.__TAURI__?.core?.invoke;
    return call ? call(command, args) : Promise.reject(new Error("IPC is unavailable"));
  };

  const absoluteUrl = (value) => {
    try {
      return new URL(value, location.href).href;
    } catch {
      return "";
    }
  };

  const send = (event) => invoke("client_event", { event }).catch(() => {});
  let seededNotifications = false;
  let lastUnread = -1;
  const seenInPage = new Set();

  function scanNotifications() {
    const badge = document.querySelector("#notif-badge");
    const parsed = Number.parseInt((badge?.textContent || "0").trim(), 10);
    const unread = Number.isFinite(parsed) ? parsed : 0;
    if (unread !== lastUnread) {
      lastUnread = unread;
      send({ kind: "unread", unread });
    }

    const items = [...document.querySelectorAll(".notif-item.unread[data-notif-id]")];
    for (const item of items) {
      const notificationId = item.dataset.notifId;
      if (!notificationId || seenInPage.has(notificationId)) continue;
      seenInPage.add(notificationId);
      send({
        kind: seededNotifications ? "notification" : "seedNotification",
        unread,
        notificationId,
        title: item.querySelector(".notif-item-title")?.textContent?.trim() || "Новое уведомление",
        body: item.querySelector(".notif-item-message")?.textContent?.trim() || "",
        url: absoluteUrl(item.getAttribute("href")),
      });
    }
    seededNotifications = true;
  }

  function installNotificationObserver() {
    scanNotifications();
    const observer = new MutationObserver(scanNotifications);
    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      characterData: true,
      attributes: true,
      attributeFilter: ["class", "style"],
    });
    window.addEventListener("pagehide", () => observer.disconnect(), { once: true });
  }

  const offlineMarkup = `
    <section id="ebloid-offline" aria-live="polite" aria-hidden="true">
      <div class="ebloid-offline-mark" aria-hidden="true">
        <svg viewBox="0 0 64 64"><path d="M12 25.5C23 14.2 41 14.2 52 25.5M20 34c6.7-6.8 17.3-6.8 24 0M28 42.5c2.2-2.2 5.8-2.2 8 0"/><circle cx="32" cy="50" r="2.8"/></svg>
      </div>
      <p class="ebloid-offline-kicker">EBLOID DESKTOP</p>
      <h1>Сеть куда-то съебалась</h1>
      <p>Проверьте подключение к интернету. Лента вернётся на это же место, как только связь восстановится.</p>
      <button id="ebloid-retry" type="button">Попробовать снова</button>
      <span id="ebloid-network-state">Ждём подключения…</span>
    </section>`;

  const launcherMarkup = `
    <div id="ebloid-client-launcher">
      <button id="ebloid-client-button" type="button" aria-label="Меню Ebloid Desktop" aria-expanded="false" title="Ebloid Desktop">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 7.5h14M5 12h14M5 16.5h14"/></svg>
        <span>Клиент</span>
      </button>
      <div id="ebloid-client-menu" role="menu" aria-hidden="true">
        <div class="ebloid-client-menu-head"><strong>Ebloid Desktop</strong><span>Управление клиентом</span></div>
        <button type="button" role="menuitem" data-section="general">${icon("M12 15.5A3.5 3.5 0 1 0 12 8a3.5 3.5 0 0 0 0 7.5ZM19 13.5v-3l-2-.7-.7-1.7.9-1.9-2.1-2.1-1.9.9-1.7-.7L10.5 2h-3l-.7 2-1.7.7-1.9-.9-2.1 2.1.9 1.9-.7 1.7-2 .7v3l2 .7.7 1.7-.9 1.9 2.1 2.1 1.9-.9 1.7.7.7 2h3l.7-2 1.7-.7 1.9.9 2.1-2.1-.9-1.9.7-1.7 2-.7Z")}<span>Настройки</span></button>
        <button type="button" role="menuitem" data-section="downloads">${icon("M12 3v12m0 0 4-4m-4 4-4-4M5 20h14")}<span>Загрузки</span><kbd>⌘⇧D</kbd></button>
        <button type="button" role="menuitem" data-section="updates">${icon("M20 7v5h-5M4 17v-5h5M6.1 8.2A7 7 0 0 1 18.6 7M17.9 15.8A7 7 0 0 1 5.4 17")}<span>Обновление</span></button>
      </div>
    </div>`;

  const clientStyle = `
    #ebloid-offline{position:fixed;inset:0;z-index:2147483646;display:none;place-content:center;justify-items:start;padding:clamp(28px,7vw,96px);box-sizing:border-box;background:#1b1b1b;color:#f2f2f2;font-family:inherit;text-align:left}
    #ebloid-offline[aria-hidden="false"]{display:grid}
    #ebloid-offline::after{content:"";position:absolute;right:-12vw;bottom:-22vw;width:min(62vw,760px);aspect-ratio:1;border:1px solid rgba(159,194,54,.28);border-radius:50%;box-shadow:0 0 0 72px rgba(159,194,54,.025),0 0 0 144px rgba(159,194,54,.018);pointer-events:none}
    .ebloid-offline-mark{width:68px;height:68px;display:grid;place-items:center;border:1px solid #9fc236;border-radius:18px;background:#22231f;box-shadow:inset 0 1px rgba(255,255,255,.04)}
    .ebloid-offline-mark svg{width:42px;fill:none;stroke:#9fc236;stroke-width:4;stroke-linecap:round}
    .ebloid-offline-kicker{margin:30px 0 10px!important;color:#9fc236!important;font-size:12px!important;font-weight:700!important;letter-spacing:.16em!important}
    #ebloid-offline h1{max-width:680px;margin:0;font-size:clamp(38px,7vw,80px);line-height:.95;letter-spacing:-.045em;font-weight:800}
    #ebloid-offline>p:not(.ebloid-offline-kicker){max-width:560px;margin:22px 0 28px;color:#aaa;font-size:17px;line-height:1.55}
    #ebloid-retry{min-height:48px;padding:0 22px;border:1px solid #a7ca3b;border-radius:10px;background:#98b832;color:#111;font:700 15px inherit;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),background-color 140ms ease}
    #ebloid-retry:active{transform:scale(.97)}
    #ebloid-retry:disabled{cursor:wait;opacity:.7}
    #ebloid-network-state{margin-top:14px;color:#6f6f6f;font-size:13px}
    #ebloid-context-menu{position:fixed;z-index:2147483647;min-width:230px;padding:6px;border:1px solid #3b3b3b;border-radius:12px;background:#242424;color:#eee;box-shadow:0 18px 50px rgba(0,0,0,.42);transform-origin:var(--origin-x) var(--origin-y);transition:opacity 120ms ease-out,transform 120ms cubic-bezier(.23,1,.32,1)}
    #ebloid-context-menu[data-open="false"]{opacity:0;transform:scale(.97);pointer-events:none}
    #ebloid-context-menu button{display:flex;width:100%;min-height:38px;align-items:center;gap:10px;padding:7px 10px;border:0;border-radius:8px;background:transparent;color:inherit;font:500 14px inherit;text-align:left;cursor:pointer}
    #ebloid-context-menu button:hover{background:#303030}
    #ebloid-context-menu button:active{transform:scale(.98)}
    #ebloid-context-menu svg{width:17px;height:17px;fill:none;stroke:#a7ca3b;stroke-width:1.8;stroke-linecap:round;stroke-linejoin:round}
    #ebloid-client-launcher{position:fixed;right:16px;bottom:16px;z-index:2147483645;font-family:inherit}
    #ebloid-client-button{display:flex;min-height:42px;align-items:center;gap:8px;padding:0 13px;border:1px solid #444;border-radius:12px;background:#242424;color:#eee;box-shadow:0 10px 30px rgba(0,0,0,.3);font:650 13px inherit;cursor:pointer;transition:background-color 120ms ease,transform 120ms ease}
    #ebloid-client-button:hover{background:#2c2c2c}
    #ebloid-client-button:active{transform:scale(.97)}
    #ebloid-client-button svg{width:17px;fill:none;stroke:#a7ca3b;stroke-width:2;stroke-linecap:round}
    #ebloid-client-menu{position:absolute;right:0;bottom:50px;width:260px;padding:6px;border:1px solid #404040;border-radius:14px;background:#242424;color:#eee;box-shadow:0 18px 55px rgba(0,0,0,.48);transform-origin:bottom right;transition:opacity 120ms ease,transform 120ms cubic-bezier(.23,1,.32,1)}
    #ebloid-client-menu[aria-hidden="true"]{opacity:0;transform:scale(.96);pointer-events:none}
    .ebloid-client-menu-head{display:grid;gap:3px;padding:10px 11px 12px;border-bottom:1px solid #373737;margin-bottom:5px}
    .ebloid-client-menu-head strong{font-size:13px}.ebloid-client-menu-head span{color:#8d8d8d;font-size:11px}
    #ebloid-client-menu>button{display:grid;width:100%;min-height:40px;grid-template-columns:19px 1fr auto;align-items:center;gap:10px;padding:7px 10px;border:0;border-radius:9px;background:transparent;color:inherit;font:500 13px inherit;text-align:left;cursor:pointer}
    #ebloid-client-menu>button:hover{background:#303030}#ebloid-client-menu>button:active{transform:scale(.98)}
    #ebloid-client-menu>button svg{width:18px;height:18px;fill:none;stroke:#a7ca3b;stroke-width:1.8;stroke-linecap:round;stroke-linejoin:round}
    #ebloid-client-menu kbd{color:#777;font:11px inherit}
    #ebloid-download-toast{position:fixed;right:16px;bottom:70px;z-index:2147483644;width:min(330px,calc(100vw - 32px));padding:14px;border:1px solid #424242;border-radius:14px;background:#242424;color:#eee;box-shadow:0 16px 46px rgba(0,0,0,.42);font-family:inherit;transition:opacity 140ms ease,transform 140ms cubic-bezier(.23,1,.32,1)}
    #ebloid-download-toast[aria-hidden="true"]{opacity:0;transform:translateY(8px);pointer-events:none}
    .ebloid-download-toast-head{display:flex;align-items:center;justify-content:space-between;gap:12px}.ebloid-download-toast-head strong{overflow:hidden;font-size:13px;text-overflow:ellipsis;white-space:nowrap}.ebloid-download-toast-head button{padding:0;border:0;background:transparent;color:#a7ca3b;font:650 12px inherit;cursor:pointer}
    #ebloid-download-toast-meta{display:block;margin-top:7px;color:#929292;font-size:11px}
    .ebloid-download-toast-progress{height:3px;margin-top:11px;overflow:hidden;border-radius:4px;background:#3a3a3a}.ebloid-download-toast-progress span{display:block;height:100%;background:#a7ca3b;transition:width 120ms linear}
    @media(max-width:600px){#ebloid-client-launcher{right:10px;bottom:10px}#ebloid-client-button span{display:none}#ebloid-client-button{width:42px;justify-content:center;padding:0}}
    @media(prefers-reduced-motion:reduce){#ebloid-context-menu,#ebloid-retry{transition-duration:0ms}}
  `;

  function ensureClientUi() {
    if (!document.getElementById("ebloid-client-style")) {
      const style = document.createElement("style");
      style.id = "ebloid-client-style";
      style.textContent = clientStyle;
      document.head.append(style);
    }
    if (!document.getElementById("ebloid-offline")) {
      document.body.insertAdjacentHTML("beforeend", offlineMarkup);
      document.getElementById("ebloid-retry")?.addEventListener("click", async (event) => {
        const button = event.currentTarget;
        button.disabled = true;
        document.getElementById("ebloid-network-state").textContent = "Подключаемся…";
        try {
          await invoke("retry_connection");
        } catch {
          button.disabled = false;
          document.getElementById("ebloid-network-state").textContent = "Связи всё ещё нет";
        }
      });
    }
    if (!document.getElementById("ebloid-client-launcher")) {
      document.body.insertAdjacentHTML("beforeend", launcherMarkup);
      const launcher = document.getElementById("ebloid-client-launcher");
      const button = document.getElementById("ebloid-client-button");
      const menu = document.getElementById("ebloid-client-menu");
      const setOpen = (open) => {
        button?.setAttribute("aria-expanded", String(open));
        menu?.setAttribute("aria-hidden", String(!open));
      };
      button?.addEventListener("click", () => setOpen(button.getAttribute("aria-expanded") !== "true"));
      menu?.addEventListener("click", (event) => {
        const section = event.target.closest("button[data-section]")?.dataset.section;
        if (!section) return;
        setOpen(false);
        invoke("open_client_settings", { section }).catch(() => {});
      });
      document.addEventListener("pointerdown", (event) => {
        if (!launcher?.contains(event.target)) setOpen(false);
      }, true);
      document.addEventListener("keydown", (event) => {
        if (event.key === "Escape") setOpen(false);
      });
    }
  }

  window.__ebloidClientSetOnline = (online) => {
    ensureClientUi();
    const overlay = document.getElementById("ebloid-offline");
    overlay?.setAttribute("aria-hidden", online ? "true" : "false");
    if (!online) {
      const button = document.getElementById("ebloid-retry");
      if (button) button.disabled = false;
      const state = document.getElementById("ebloid-network-state");
      if (state) state.textContent = "Ждём подключения…";
    }
    send({ kind: "network", online });
  };

  let downloadToastTimer;
  window.__ebloidClientDownload = (item) => {
    if (!item) return;
    let toast = document.getElementById("ebloid-download-toast");
    if (!toast) {
      toast = document.createElement("section");
      toast.id = "ebloid-download-toast";
      toast.setAttribute("aria-live", "polite");
      toast.innerHTML = `<div class="ebloid-download-toast-head"><strong></strong><button type="button">Открыть загрузки</button></div><span id="ebloid-download-toast-meta"></span><div class="ebloid-download-toast-progress"><span></span></div>`;
      toast.querySelector("button")?.addEventListener("click", () => invoke("open_client_settings", { section: "downloads" }).catch(() => {}));
      document.body.append(toast);
    }
    clearTimeout(downloadToastTimer);
    toast.querySelector("strong").textContent = item.fileName || "Загрузка файла";
    const total = Number(item.totalBytes) || 0;
    const downloaded = Number(item.downloadedBytes) || 0;
    const percent = total ? Math.min(100, downloaded / total * 100) : (item.status === "completed" ? 100 : 8);
    const labels = { queued: "Подготовка…", downloading: total ? `Скачано ${Math.round(percent)}%` : "Скачивание…", completed: "Сохранено в папку «Загрузки»", failed: "Ошибка загрузки", cancelled: "Загрузка отменена" };
    toast.querySelector("#ebloid-download-toast-meta").textContent = labels[item.status] || item.status;
    toast.querySelector(".ebloid-download-toast-progress span").style.width = `${percent}%`;
    toast.setAttribute("aria-hidden", "false");
    if (["completed", "failed", "cancelled"].includes(item.status)) {
      downloadToastTimer = setTimeout(() => toast?.setAttribute("aria-hidden", "true"), 6000);
    }
  };

  function icon(path) {
    return `<svg viewBox="0 0 24 24" aria-hidden="true"><path d="${path}"/></svg>`;
  }

  function showContextMenu(event, link, image) {
    document.getElementById("ebloid-context-menu")?.remove();
    const menu = document.createElement("div");
    menu.id = "ebloid-context-menu";
    menu.dataset.open = "false";
    menu.setAttribute("role", "menu");
    const actions = [];
    if (link) {
      actions.push(["browser", "Открыть в браузере", icon("M14 5h5v5M19 5l-9 9M12 7H6a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-6")]);
      actions.push(["copy", "Скопировать ссылку", icon("M9 8h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1ZM6 16H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v1")]);
    }
    if (image) {
      actions.push(["save", "Сохранить изображение", icon("M12 4v11m0 0 4-4m-4 4-4-4M5 19h14")]);
      if (!link) actions.push(["copy", "Скопировать ссылку", icon("M9 8h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1Z")]);
    }
    if (!actions.length) return;
    menu.innerHTML = actions.map(([action, label, svg]) => `<button type="button" role="menuitem" data-action="${action}">${svg}<span>${label}</span></button>`).join("");
    document.body.append(menu);
    const width = 242;
    const estimatedHeight = actions.length * 38 + 12;
    menu.style.left = `${Math.max(8, Math.min(event.clientX, innerWidth - width - 8))}px`;
    menu.style.top = `${Math.max(8, Math.min(event.clientY, innerHeight - estimatedHeight - 8))}px`;
    requestAnimationFrame(() => { menu.dataset.open = "true"; });
    menu.addEventListener("click", async (click) => {
      const action = click.target.closest("button")?.dataset.action;
      const url = action === "save" ? image : (link || image);
      if (action === "browser") await invoke("open_external", { url }).catch(() => {});
      if (action === "copy") await navigator.clipboard.writeText(url).catch(() => {});
      if (action === "save") await invoke("download_url", { url }).catch(() => {});
      menu.remove();
    });
    const close = (closeEvent) => {
      if (!menu.contains(closeEvent.target)) menu.remove();
    };
    setTimeout(() => document.addEventListener("pointerdown", close, { once: true, capture: true }), 0);
  }

  function installContextMenu() {
    document.addEventListener("contextmenu", (event) => {
      const target = event.target instanceof Element ? event.target : null;
      const anchor = target?.closest("a[href]");
      const imageElement = target?.closest("img[src],video[poster]");
      const link = anchor ? absoluteUrl(anchor.getAttribute("href")) : "";
      const image = imageElement ? absoluteUrl(imageElement.getAttribute("src") || imageElement.getAttribute("poster")) : "";
      if (!link && !image) return;
      event.preventDefault();
      showContextMenu(event, link, image);
    }, true);
  }

  function installClipboardFallback() {
    document.addEventListener("paste", (event) => {
      const images = [...(event.clipboardData?.items || [])].filter((item) => item.kind === "file" && item.type.startsWith("image/"));
      if (!images.length || event.defaultPrevented) return;
      queueMicrotask(() => {
        if (event.defaultPrevented) return;
        const input = [...document.querySelectorAll('input[type="file"]')].find((candidate) => {
          const style = getComputedStyle(candidate);
          return !candidate.disabled && style.display !== "none" && style.visibility !== "hidden";
        });
        if (!input || typeof DataTransfer === "undefined") return;
        const transfer = new DataTransfer();
        for (const item of images) {
          const file = item.getAsFile();
          if (file) transfer.items.add(file);
        }
        if (!transfer.files.length) return;
        input.files = transfer.files;
        input.dispatchEvent(new Event("change", { bubbles: true }));
      });
    }, true);
  }

  function activeVideo() {
    const videos = [...document.querySelectorAll("video")];
    return videos.find((video) => !video.paused && !video.ended) || videos.find((video) => video.readyState > 0) || null;
  }

  function installMediaKeys() {
    if (!("mediaSession" in navigator)) return;
    const handle = (action, callback) => {
      try { navigator.mediaSession.setActionHandler(action, callback); } catch {}
    };
    handle("play", () => activeVideo()?.play());
    handle("pause", () => activeVideo()?.pause());
    handle("stop", () => { const video = activeVideo(); if (video) { video.pause(); video.currentTime = 0; } });
    handle("seekbackward", (details) => { const video = activeVideo(); if (video) video.currentTime = Math.max(0, video.currentTime - (details.seekOffset || 10)); });
    handle("seekforward", (details) => { const video = activeVideo(); if (video) video.currentTime = Math.min(video.duration || Infinity, video.currentTime + (details.seekOffset || 10)); });
    handle("seekto", (details) => { const video = activeVideo(); if (video && Number.isFinite(details.seekTime)) video.currentTime = details.seekTime; });
    document.addEventListener("play", () => { navigator.mediaSession.playbackState = "playing"; }, true);
    document.addEventListener("pause", () => { navigator.mediaSession.playbackState = "paused"; }, true);
  }

  function installShortcuts() {
    document.addEventListener("keydown", (event) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      if (event.key === ",") {
        event.preventDefault();
        invoke("open_client_settings", { section: "general" }).catch(() => {});
      }
      if (event.shiftKey && event.key.toLowerCase() === "d") {
        event.preventDefault();
        invoke("open_client_settings", { section: "downloads" }).catch(() => {});
      }
    }, true);
  }

  function boot() {
    ensureClientUi();
    installNotificationObserver();
    installContextMenu();
    installClipboardFallback();
    installMediaKeys();
    installShortcuts();
    window.__ebloidClientSetOnline(navigator.onLine);
    window.addEventListener("online", () => window.__ebloidClientSetOnline(true));
    window.addEventListener("offline", () => window.__ebloidClientSetOnline(false));
    setInterval(() => send({ kind: "heartbeat", online: navigator.onLine }), 10_000);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }
})();
