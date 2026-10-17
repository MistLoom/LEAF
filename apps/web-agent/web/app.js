const $ = (id) => document.getElementById(id);

let currentSession = null;
let sessionTitles = new Map();
let busy = false;

const STORAGE_THEME = "web-agent-theme";
const STORAGE_ECO = "web-agent-eco-open";

function applyTheme(theme) {
  const t = theme === "light" ? "light" : "dark";
  document.documentElement.dataset.theme = t;
  localStorage.setItem(STORAGE_THEME, t);
}

function initTheme() {
  const saved = localStorage.getItem(STORAGE_THEME);
  if (saved === "light" || saved === "dark") {
    applyTheme(saved);
    return;
  }
  applyTheme(
    window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark",
  );
}

function toggleTheme() {
  const next = document.documentElement.dataset.theme === "light" ? "dark" : "light";
  applyTheme(next);
}

if (typeof marked !== "undefined") {
  marked.setOptions({ breaks: true, gfm: true });
}

async function api(path, opts = {}) {
  const r = await fetch(path, {
    headers: { "Content-Type": "application/json" },
    ...opts,
  });
  const data = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(data.error || r.statusText);
  return data;
}

function setStatus(t, kind = "") {
  const el = $("status");
  el.textContent = t;
  el.classList.remove("busy", "error");
  if (kind) el.classList.add(kind);
}

function setBusy(on) {
  busy = on;
  $("btn-send").disabled = on;
  $("btn-stop").hidden = !on;
  $("prompt").disabled = on;
  setStatus(on ? "执行中…" : "就绪", on ? "busy" : "");
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function messageText(msg) {
  const c = msg.content;
  if (c == null) return "";
  if (typeof c === "string") return c;
  if (Array.isArray(c)) {
    return c
      .filter((p) => p && p.type === "text" && p.text)
      .map((p) => p.text)
      .join("\n");
  }
  return "";
}

function normalizeMessages(data) {
  if (Array.isArray(data)) return data;
  if (data?.messages && Array.isArray(data.messages)) return data.messages;
  return [];
}

function renderMarkdown(text) {
  if (typeof marked !== "undefined") {
    try {
      return marked.parse(text);
    } catch {
      /* fallback */
    }
  }
  return `<p>${escapeHtml(text).replace(/\n/g, "<br>")}</p>`;
}

function roleLabel(role) {
  const map = {
    user: "你",
    assistant: "AI",
    tool: "工具",
    system: "系统",
    developer: "Dev",
  };
  return map[role] || role;
}

function renderToolCalls(toolCalls) {
  if (!toolCalls?.length) return null;
  const wrap = document.createElement("div");
  wrap.className = "tool-calls";
  for (const tc of toolCalls) {
    const name = tc.function?.name || tc.name || "tool";
    const args = tc.function?.arguments ?? tc.arguments ?? "";
    const details = document.createElement("details");
    details.className = "tool-call";
    details.open = false;
    const summary = document.createElement("summary");
    summary.textContent = name;
    const pre = document.createElement("pre");
    pre.textContent = typeof args === "string" ? args : JSON.stringify(args, null, 2);
    details.appendChild(summary);
    details.appendChild(pre);
    wrap.appendChild(details);
  }
  return wrap;
}

function appendMessageRow(role, htmlOrText, opts = {}) {
  const row = document.createElement("article");
  row.className = `msg-row ${role}`;

  const avatar = document.createElement("div");
  avatar.className = "msg-avatar";
  avatar.textContent = roleLabel(role).slice(0, 2);
  avatar.setAttribute("aria-hidden", "true");

  const body = document.createElement("div");
  body.className = "msg-body";

  const bubble = document.createElement("div");
  bubble.className = "msg-bubble";
  if (opts.markdown) {
    bubble.classList.add("markdown");
    bubble.innerHTML = htmlOrText;
  } else {
    bubble.textContent = htmlOrText;
  }

  body.appendChild(bubble);
  if (opts.toolCalls) {
    const tc = renderToolCalls(opts.toolCalls);
    if (tc) body.appendChild(tc);
  }

  row.appendChild(avatar);
  row.appendChild(body);
  return row;
}

function renderEmptyState() {
  return `
    <div class="empty-state">
      <h3>开始对话</h3>
      <p>向 web-agent 描述任务，生态 Ability 会在后台参与编排。</p>
      <div class="suggestions">
        <button type="button" class="suggestion-chip" data-prompt="列出当前可用的生态工具并简要说明用途">查看生态工具</button>
        <button type="button" class="suggestion-chip" data-prompt="用一句话介绍你能帮我做什么">你能做什么</button>
      </div>
    </div>`;
}

function scrollChatToBottom() {
  const scroller = document.querySelector(".chat-scroll");
  if (scroller) scroller.scrollTop = scroller.scrollHeight;
}

function renderMessages(msgs, extra = null) {
  const box = $("messages");
  box.innerHTML = "";
  const list = normalizeMessages(msgs);

  if (!list.length && !extra) {
    box.innerHTML = renderEmptyState();
    bindSuggestionChips();
    return;
  }

  for (const msg of list) {
    const role = msg.role || "assistant";
    if (role === "system") continue;
    const text = messageText(msg);
    if (!text && !msg.tool_calls?.length) continue;

    if (role === "assistant") {
      const html = text ? renderMarkdown(text) : "";
      const row = appendMessageRow("assistant", html || "（无文本）", {
        markdown: Boolean(text),
        toolCalls: msg.tool_calls,
      });
      box.appendChild(row);
    } else if (role === "user") {
      box.appendChild(appendMessageRow("user", text || "（空）"));
    } else if (role === "tool") {
      const label = msg.name || msg.tool_call_id || "tool";
      box.appendChild(appendMessageRow("tool", `${label}\n${text}`));
    }
  }

  if (extra === "typing") {
    const row = document.createElement("article");
    row.className = "msg-row assistant";
    row.id = "typing-row";
    row.innerHTML = `
      <div class="msg-avatar" aria-hidden="true">AI</div>
      <div class="msg-body"><div class="typing" aria-label="正在回复"><span></span><span></span><span></span></div></div>`;
    box.appendChild(row);
  }

  if (extra && extra !== "typing" && typeof extra === "string") {
    const err = document.createElement("div");
    err.className = "msg-error";
    err.textContent = extra;
    box.appendChild(err);
  }

  scrollChatToBottom();
}

function bindSuggestionChips() {
  document.querySelectorAll(".suggestion-chip").forEach((btn) => {
    btn.onclick = () => {
      $("prompt").value = btn.dataset.prompt || "";
      autoResizePrompt();
      $("prompt").focus();
    };
  });
}

function updateChatTitle() {
  const title = currentSession
    ? sessionTitles.get(currentSession) || `会话 ${currentSession.slice(0, 8)}`
    : "新对话";
  $("chat-title").textContent = title;
}

function renderEco(flags) {
  const el = $("eco-flags");
  el.innerHTML = "";
  for (const [k, on] of Object.entries(flags)) {
    const s = document.createElement("span");
    s.className = on ? "on" : "";
    s.textContent = `${k}: ${on ? "开" : "关"}`;
    el.appendChild(s);
  }
}

async function loadHealth() {
  const h = await api("/api/health");
  renderEco(h.ecosystem);
}

async function loadEcoTools() {
  const d = await api("/api/dispatch/tools");
  const ul = $("eco-tools");
  ul.innerHTML = "";
  for (const name of d.eco_tools || []) {
    const li = document.createElement("li");
    li.textContent = name;
    ul.appendChild(li);
  }
}

async function loadInventory() {
  const d = await api("/api/ecosystem/inventory");
  $("eco-inventory").textContent = JSON.stringify(d, null, 2);
}

function renderTurnEco(eco) {
  $("eco-turn").textContent = eco ? JSON.stringify(eco, null, 2) : "（尚无本轮数据）";
}

function pushEcoActivity(line) {
  try {
    const o = JSON.parse(line);
    if (
      o.channel === "user" &&
      (o.name === "eco.tool_started" || (o.name === "tool.started" && o.detail?.eco))
    ) {
      const box = $("eco-activity");
      const tag = o.name === "eco.tool_started" ? "生态工具" : "工具";
      box.textContent = `[${tag}] ${o.detail?.name}\n` + box.textContent.slice(0, 2000);
    }
  } catch {
    /* ignore */
  }
}

async function runDoctor() {
  setStatus("Doctor…", "busy");
  const d = await api("/api/ecosystem/doctor");
  $("eco-doctor").textContent = JSON.stringify(d, null, 2);
  setStatus("就绪");
}

async function refreshSessions() {
  const data = await api("/api/sessions");
  const ul = $("session-list");
  ul.innerHTML = "";
  sessionTitles.clear();
  const items = data.sessions || data.items || [];
  for (const s of items) {
    const id = s.id || s.session_id;
    if (!id) continue;
    const title = s.title || id.slice(0, 8);
    sessionTitles.set(id, title);
    const li = document.createElement("li");
    li.textContent = title;
    li.dataset.id = id;
    li.title = title;
    if (id === currentSession) li.classList.add("active");
    li.onclick = () => selectSession(id);
    ul.appendChild(li);
  }
  updateChatTitle();
  return items;
}

async function selectSession(id) {
  currentSession = id;
  closeSidebarMobile();
  await refreshSessions();
  const msgs = await api(`/api/sessions/${id}/messages`);
  renderMessages(msgs);
}

async function newSession() {
  const s = await api("/api/sessions", { method: "POST", body: "{}" });
  currentSession = s.id;
  sessionTitles.set(s.id, s.title || "新对话");
  await refreshSessions();
  renderMessages([]);
}

function autoResizePrompt() {
  const ta = $("prompt");
  ta.style.height = "auto";
  ta.style.height = `${Math.min(ta.scrollHeight, 200)}px`;
}

function toggleEco(open) {
  const app = document.querySelector(".app");
  const on = open ?? !app.classList.contains("eco-open");
  app.classList.toggle("eco-open", on);
  localStorage.setItem(STORAGE_ECO, on ? "1" : "0");
}

function initEcoPanel() {
  if (localStorage.getItem(STORAGE_ECO) === "1") {
    toggleEco(true);
  }
}

function closeSidebarMobile() {
  document.querySelector(".app").classList.remove("sidebar-open");
  $("sidebar-backdrop").hidden = true;
}

function openSidebarMobile() {
  document.querySelector(".app").classList.add("sidebar-open");
  $("sidebar-backdrop").hidden = false;
}

$("btn-new").onclick = () => newSession().catch((e) => setStatus(e.message, "error"));
$("btn-doctor").onclick = () => runDoctor().catch((e) => setStatus(e.message, "error"));
$("btn-inventory").onclick = () => loadInventory().catch((e) => setStatus(e.message, "error"));

$("btn-eco-toggle").onclick = () => toggleEco(true);
$("btn-eco-close").onclick = () => toggleEco(false);
$("btn-sidebar").onclick = () => openSidebarMobile();
$("sidebar-backdrop").onclick = () => closeSidebarMobile();
$("btn-theme").onclick = () => toggleTheme();

$("btn-session-start").onclick = async () => {
  if (!currentSession) await newSession();
  setStatus("session.start…", "busy");
  try {
    const out = await api("/api/ecosystem/invoke", {
      method: "POST",
      body: JSON.stringify({
        ability: "claude_mod",
        tool: "session.start",
        args: { session_id: currentSession },
      }),
    });
    $("eco-doctor").textContent = JSON.stringify(out, null, 2);
    toggleEco(true);
  } catch (e) {
    setStatus(e.message, "error");
    return;
  }
  setStatus("就绪");
};

$("btn-stop").onclick = async () => {
  try {
    await api("/api/cancel", { method: "POST", body: "{}" });
    setStatus("已请求停止", "busy");
  } catch (e) {
    setStatus(e.message, "error");
  }
};

$("prompt").addEventListener("input", autoResizePrompt);
$("prompt").addEventListener("keydown", (ev) => {
  if (ev.key === "Enter" && !ev.shiftKey) {
    ev.preventDefault();
    $("composer").requestSubmit();
  }
});

$("composer").onsubmit = async (e) => {
  e.preventDefault();
  if (busy) return;
  const text = $("prompt").value.trim();
  if (!text) return;
  if (!currentSession) await newSession();

  $("prompt").value = "";
  autoResizePrompt();

  const pending = normalizeMessages(await api(`/api/sessions/${currentSession}/messages`));
  pending.push({ role: "user", content: text });
  renderMessages(pending, "typing");
  setBusy(true);

  try {
    const out = await api(`/api/sessions/${currentSession}/chat`, {
      method: "POST",
      body: JSON.stringify({ text }),
    });
    renderTurnEco(out.ecosystem);
    const msgs = await api(`/api/sessions/${currentSession}/messages`);
    renderMessages(msgs);
    await loadEcoTools();
  } catch (err) {
    renderMessages(pending, `错误: ${err.message}`);
    setStatus(err.message, "error");
    setBusy(false);
    return;
  }
  setBusy(false);
};

function connectEvents() {
  const es = new EventSource("/api/events");
  es.onmessage = (ev) => {
    const line = ev.data;
    pushEcoActivity(line);
    const box = $("events");
    box.textContent = `${line}\n` + box.textContent.slice(0, 4000);
  };
}

(async () => {
  initTheme();
  initEcoPanel();
  renderMessages([]);
  try {
    await loadHealth();
    await loadInventory();
    await loadEcoTools();
    const items = await refreshSessions();
    if (!currentSession && items.length) {
      const first = items[0];
      const id = first.id || first.session_id;
      if (id) await selectSession(id);
    }
    connectEvents();
  } catch (e) {
    setStatus(e.message, "error");
  }
})();
