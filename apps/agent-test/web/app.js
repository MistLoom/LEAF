let currentSession = null;
let loading = false;
let eventSource = null;

const $ = (id) => document.getElementById(id);

function setStatus(text) {
  $("status").textContent = text;
}

function roleClass(role) {
  if (role === "user") return "user";
  if (role === "assistant") return "assistant";
  if (role === "tool") return "tool";
  if (role === "system") return "system";
  return "assistant";
}

function formatToolJson(text) {
  try {
    const obj = JSON.parse(text);
    if (obj && typeof obj === "object") {
      if (obj.stderr && typeof obj.stderr === "string") return obj.stderr.trim() || text;
      if (obj.stdout && typeof obj.stdout === "string" && !obj.stderr) return obj.stdout.trim() || text;
      if (typeof obj.count === "number" && "catalog" in obj) {
        return `skill_list: ${obj.count} 个技能`;
      }
      return JSON.stringify(obj, null, 2);
    }
  } catch (_) {}
  return text;
}

function contentText(msg) {
  const c = msg.content;
  if (typeof c === "string") {
    if (msg.role === "tool") return formatToolJson(c);
    return c;
  }
  if (c && typeof c === "object" && c.text) return c.text;
  if (msg.tool_calls && msg.tool_calls.length) {
    const names = msg.tool_calls.map((t) => t.function?.name || t.name).join(", ");
    return `调用工具: ${names}`;
  }
  return "";
}

function renderMessages(msgs) {
  const box = $("messages");
  box.innerHTML = "";
  for (const m of msgs) {
    const div = document.createElement("div");
    div.className = `msg ${roleClass(m.role)}`;
    div.textContent = contentText(m);
    box.appendChild(div);
  }
  box.scrollTop = box.scrollHeight;
}

async function api(path, opts = {}) {
  const res = await fetch(path, {
    headers: { "Content-Type": "application/json", ...opts.headers },
    ...opts,
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error || res.statusText);
  return data;
}

async function loadSessions() {
  const sessions = await api("/api/sessions");
  const list = $("session-list");
  list.innerHTML = "";
  const arr = Array.isArray(sessions) ? sessions : [];
  for (const s of arr) {
    const li = document.createElement("li");
    const id = s.id || s;
    li.textContent = (s.title || id).slice(0, 36);
    li.title = id;
    li.dataset.id = id;
    if (id === currentSession) li.classList.add("active");
    li.onclick = () => selectSession(id);
    list.appendChild(li);
  }
  if (!currentSession && arr.length) {
    const first = arr[0].id || arr[0];
    await selectSession(first);
  }
}

async function selectSession(id) {
  currentSession = id;
  document.querySelectorAll("#session-list li").forEach((li) => {
    li.classList.toggle("active", li.dataset.id === id);
  });
  const msgs = await api(`/api/sessions/${id}/messages`);
  renderMessages(Array.isArray(msgs) ? msgs : []);
  setStatus(`会话 ${id.slice(0, 8)}…`);
}

async function newSession() {
  const out = await api("/api/sessions", { method: "POST" });
  currentSession = out.id;
  await loadSessions();
  await selectSession(out.id);
}

function setLoading(on) {
  loading = on;
  document.body.classList.toggle("loading", on);
  $("btn-send").disabled = on;
  $("btn-spawn").disabled = on;
  $("btn-compact").disabled = on;
}

function connectEvents() {
  if (eventSource) eventSource.close();
  eventSource = new EventSource("/api/events");
  const box = $("events");
  eventSource.onmessage = (ev) => {
    try {
      const p = JSON.parse(ev.data);
      const ch = p.channel || "ability";
      const line = `[${ch}:${p.name}] ${JSON.stringify(p.detail)}`;
      const div = document.createElement("div");
      div.textContent = line.slice(0, 200);
      box.appendChild(div);
      while (box.childNodes.length > 8) box.removeChild(box.firstChild);
    } catch (_) {}
  };
}

$("btn-new").onclick = () => newSession().catch(showError);

$("composer").onsubmit = async (e) => {
  e.preventDefault();
  const text = $("prompt").value.trim();
  if (!text || loading) return;
  if (!currentSession) await newSession();
  setLoading(true);
  setStatus("执行中…");
  $("prompt").value = "";
  try {
    const out = await api(`/api/sessions/${currentSession}/chat`, {
      method: "POST",
      body: JSON.stringify({ text }),
    });
    await selectSession(currentSession);
    setStatus(`完成 · ${out.rounds} 轮 · ${out.reason}`);
  } catch (err) {
    showError(err);
  } finally {
    setLoading(false);
  }
};

$("prompt").onkeydown = (e) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    $("composer").requestSubmit();
  }
};

$("btn-spawn").onclick = async () => {
  const text = $("prompt").value.trim();
  if (!text || !currentSession || loading) return;
  setLoading(true);
  setStatus("子代理执行中…");
  try {
    await api(`/api/sessions/${currentSession}/spawn`, {
      method: "POST",
      body: JSON.stringify({ prompt: text }),
    });
    $("prompt").value = "";
    await selectSession(currentSession);
    setStatus("子代理完成");
  } catch (err) {
    showError(err);
  } finally {
    setLoading(false);
  }
};

$("btn-compact").onclick = async () => {
  if (!currentSession || loading) return;
  setLoading(true);
  try {
    await api(`/api/sessions/${currentSession}/compact`, {
      method: "POST",
      body: JSON.stringify({ until_seq: 999999, marker_text: "[上下文已压缩]" }),
    });
    await selectSession(currentSession);
    setStatus("已压缩");
  } catch (err) {
    showError(err);
  } finally {
    setLoading(false);
  }
};

function showError(err) {
  setStatus(String(err.message || err));
  alert(String(err.message || err));
}

(async function init() {
  connectEvents();
  try {
    const h = await api("/api/health");
    setStatus(h.restored_sessions ? `已恢复 ${h.restored_sessions} 个会话` : "就绪");
    await loadSessions();
    if (!currentSession) await newSession();
  } catch (err) {
    showError(err);
  }
})();
