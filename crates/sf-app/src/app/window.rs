//! Main NSWindow with WKWebView — real LLM streaming + settings

use objc2::rc::Retained;
use objc2_app_kit::{NSWindow, NSWindowStyleMask, NSBackingStoreType, NSView};
use objc2_foundation::{MainThreadMarker, NSRect, NSPoint, NSSize, NSString};
use objc2_web_kit::{WKWebView, WKWebViewConfiguration};

pub fn create_main_window(mtm: MainThreadMarker) -> Retained<NSWindow> {
    let rect = NSRect::new(NSPoint::new(80.0, 80.0), NSSize::new(1320.0, 860.0));
    let style = NSWindowStyleMask::Titled
        | NSWindowStyleMask::Closable
        | NSWindowStyleMask::Miniaturizable
        | NSWindowStyleMask::Resizable
        | NSWindowStyleMask::UnifiedTitleAndToolbar;

    let alloc = mtm.alloc::<NSWindow>();
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            alloc, rect, style, NSBackingStoreType::NSBackingStoreBuffered, false,
        )
    };

    window.setTitle(&NSString::from_str("Software Factory"));
    window.setTitlebarAppearsTransparent(true);

    let cfg = unsafe { WKWebViewConfiguration::new() };
    // Developer extras: enable via Safari > Develop menu (no runtime API in objc2-web-kit 0.2)

    let wv_alloc = mtm.alloc::<WKWebView>();
    let wv = unsafe { WKWebView::initWithFrame_configuration(wv_alloc, rect, &cfg) };

    let html = NSString::from_str(SF_UI_HTML);
    unsafe {
        wv.loadHTMLString_baseURL(&html, None);
        window.setContentView(Some(&*(wv as objc2::rc::Retained<WKWebView>).as_ref() as &NSView));
    }

    window.center();
    unsafe { window.makeKeyAndOrderFront(None) };
    window
}

// ── Embedded UI ───────────────────────────────────────────────────────────────

const SF_UI_HTML: &str = r##"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Software Factory</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{
  --bg:#0f0a1a;--bg2:#1a1128;--bg3:#251a35;--bg4:#2d1f45;
  --purple:#a855f7;--purple-l:#c084fc;--purple-d:#7c3aed;
  --accent:#f78166;--green:#4ade80;--yellow:#fbbf24;--red:#f87171;--blue:#60a5fa;
  --text:#e2d9f3;--text-dim:#9580aa;--text-faint:#6b5b8a;
  --border:#3d2d5a;--border-l:#4d3d6a;
  --font:-apple-system,BlinkMacSystemFont,'SF Pro Text',sans-serif;
  --mono:'SF Mono','JetBrains Mono',monospace;
  --radius:10px;--sidebar:220px
}
body{background:var(--bg);color:var(--text);font-family:var(--font);font-size:14px;
     display:flex;height:100vh;overflow:hidden;-webkit-font-smoothing:antialiased}

/* ── Sidebar ── */
.sidebar{width:var(--sidebar);background:var(--bg2);border-right:1px solid var(--border);
         display:flex;flex-direction:column;overflow:hidden;flex-shrink:0;padding-top:40px}
.logo{padding:0 14px 16px;display:flex;align-items:center;gap:10px;border-bottom:1px solid var(--border)}
.logo-icon{width:34px;height:34px;background:linear-gradient(135deg,var(--purple),var(--purple-d));
           border-radius:9px;display:flex;align-items:center;justify-content:center;
           font-weight:800;font-size:13px;color:#fff;letter-spacing:-.5px;flex-shrink:0}
.logo-name{font-weight:700;font-size:13px;line-height:1.3}
.logo-ver{font-size:10px;color:var(--text-dim)}
.nav{flex:1;overflow-y:auto;padding:10px 0}
.nav-group{padding:14px 14px 4px;font-size:10px;font-weight:600;text-transform:uppercase;
           letter-spacing:.1em;color:var(--text-faint)}
.nav-item{display:flex;align-items:center;gap:9px;padding:8px 14px;cursor:pointer;
          color:var(--text-dim);font-size:13px;border-left:2px solid transparent;
          transition:all .12s;user-select:none;-webkit-user-select:none}
.nav-item:hover{background:var(--bg3);color:var(--text)}
.nav-item.active{background:var(--bg3);color:var(--text);border-left-color:var(--purple)}
.nav-icon{width:16px;text-align:center;font-size:13px;flex-shrink:0}
.nav-badge{margin-left:auto;background:var(--purple);color:#fff;font-size:10px;
           padding:1px 6px;border-radius:8px;font-weight:600}
.dot{width:7px;height:7px;border-radius:50%;flex-shrink:0}
.dot-g{background:var(--green);box-shadow:0 0 5px var(--green)}
.dot-p{background:var(--purple)}
.dot-d{background:var(--text-faint)}
/* Sidebar footer — instances */
.sb-footer{padding:10px;border-top:1px solid var(--border)}
.instance{display:flex;align-items:center;gap:8px;padding:7px 9px;background:var(--bg3);
          border-radius:8px;cursor:pointer;border:1px solid var(--border);margin-bottom:6px;
          transition:border-color .15s}
.instance:hover{border-color:var(--purple-l)}
.instance-info{flex:1;min-width:0}
.inst-name{font-size:12px;font-weight:500;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.inst-url{font-size:10px;color:var(--text-dim)}

/* ── Main ── */
.main{flex:1;display:flex;flex-direction:column;overflow:hidden}
.topbar{height:50px;background:var(--bg2);border-bottom:1px solid var(--border);
        display:flex;align-items:center;padding:0 18px;gap:10px;flex-shrink:0;padding-top:28px}
.topbar-title{font-weight:600;font-size:14px;flex:1}
.tag{padding:3px 9px;border-radius:16px;font-size:11px;font-weight:600;cursor:default}
.tag-mlx{background:#1a1040;color:#a78bfa;border:1px solid #5b21b6}
.tag-ol{background:#0f2318;color:var(--green);border:1px solid #166534}
.btn{padding:5px 13px;border-radius:6px;border:none;cursor:pointer;font-size:12px;
     font-weight:500;display:inline-flex;align-items:center;gap:5px;transition:all .12s;
     -webkit-app-region:no-drag}
.btn-p{background:var(--purple);color:#fff}
.btn-p:hover{background:var(--purple-l)}
.btn-g{background:transparent;color:var(--text-dim);border:1px solid var(--border)}
.btn-g:hover{color:var(--text);border-color:var(--border-l)}
/* Status bar */
.stats{display:flex;align-items:center;gap:0;border-bottom:1px solid var(--border);
       background:var(--bg2);flex-shrink:0;overflow-x:auto}
.stat{padding:6px 16px;display:flex;flex-direction:column;align-items:center;
      border-right:1px solid var(--border)}
.stat-v{font-size:15px;font-weight:700;color:var(--purple)}
.stat-l{font-size:9px;color:var(--text-dim);text-transform:uppercase;letter-spacing:.06em;margin-top:1px}
.stat-reasoning{margin-left:auto;padding:4px 14px;font-size:10px;color:var(--text-faint);
                display:flex;align-items:center;gap:6px}

/* ── Content ── */
.content{flex:1;display:flex;overflow:hidden}
/* Sessions panel */
.sessions-panel{width:260px;border-right:1px solid var(--border);display:flex;
                flex-direction:column;overflow:hidden;flex-shrink:0}
.panel-hdr{padding:10px 14px;border-bottom:1px solid var(--border);display:flex;
           align-items:center;justify-content:space-between}
.panel-title{font-size:11px;font-weight:600;text-transform:uppercase;
             letter-spacing:.07em;color:var(--text-dim)}
.sessions-list{flex:1;overflow-y:auto;padding:6px}
.sess{padding:9px 11px;border-radius:8px;cursor:pointer;margin-bottom:3px;
      border:1px solid transparent;transition:all .12s}
.sess:hover{background:var(--bg3);border-color:var(--border)}
.sess.active{background:var(--bg3);border-color:var(--purple)}
.sess-title{font-size:13px;font-weight:500;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.sess-meta{font-size:11px;color:var(--text-dim);display:flex;align-items:center;gap:5px;margin-top:3px}
.ss{width:6px;height:6px;border-radius:50%}
.ss-a{background:var(--green);box-shadow:0 0 4px var(--green)}
.ss-d{background:var(--text-faint)}

/* ── Chat ── */
.chat-area{flex:1;display:flex;flex-direction:column;overflow:hidden}
.chat-hdr{padding:12px 18px;border-bottom:1px solid var(--border);display:flex;
          align-items:center;gap:12px;background:var(--bg2);flex-shrink:0}
.avatar{width:36px;height:36px;border-radius:50%;background:linear-gradient(135deg,var(--purple),var(--purple-d));
        display:flex;align-items:center;justify-content:center;font-size:13px;
        font-weight:700;color:#fff;flex-shrink:0}
.agent-name{font-weight:600;font-size:14px}
.agent-role{font-size:11px;color:var(--text-dim);margin-top:1px}
.chat-tags{margin-left:auto;display:flex;gap:6px;align-items:center}
/* Messages */
.msgs{flex:1;overflow-y:auto;padding:18px;display:flex;flex-direction:column;gap:14px}
.msg{display:flex;gap:9px;max-width:88%}
.msg.user{align-self:flex-end;flex-direction:row-reverse}
.msg-av{width:28px;height:28px;border-radius:50%;flex-shrink:0;display:flex;
        align-items:center;justify-content:center;font-size:11px;font-weight:600}
.msg-av.ai{background:linear-gradient(135deg,var(--purple),var(--purple-d));color:#fff}
.msg-av.user{background:var(--bg4);color:var(--text);border:1px solid var(--border)}
.bubble{padding:10px 13px;border-radius:12px;font-size:13px;line-height:1.55;
        word-break:break-word;white-space:pre-wrap}
.msg.ai .bubble{background:var(--bg2);border:1px solid var(--border);border-radius:12px 12px 12px 2px}
.msg.user .bubble{background:var(--purple);color:#fff;border-radius:12px 12px 2px 12px}
code{background:var(--bg3);padding:1px 5px;border-radius:4px;font-family:var(--mono);
     font-size:12px;color:var(--purple-l)}
pre{background:var(--bg3);border:1px solid var(--border);border-radius:8px;padding:10px 12px;
    font-family:var(--mono);font-size:12px;overflow-x:auto;margin:6px 0;line-height:1.5}
.tool-block{background:var(--bg3);border:1px solid var(--border-l);border-radius:6px;
            padding:6px 10px;font-family:var(--mono);font-size:11px;margin-top:6px}
.tool-name{color:var(--purple-l);font-weight:600}
.tool-result{border-left:2px solid var(--green);padding-left:8px;margin-top:4px;
             color:var(--green);font-size:11px;font-family:var(--mono);opacity:.9}
/* Typing */
.typing{display:flex;gap:4px;align-items:center;padding:3px 0}
.typing span{width:6px;height:6px;border-radius:50%;background:var(--purple);
             animation:bounce .75s infinite}
.typing span:nth-child(2){animation-delay:.15s}
.typing span:nth-child(3){animation-delay:.3s}
@keyframes bounce{0%,80%,100%{transform:scale(0.7);opacity:.4}40%{transform:scale(1);opacity:1}}
/* Input */
.input-area{padding:12px 18px;border-top:1px solid var(--border);background:var(--bg2);
            display:flex;gap:10px;align-items:flex-end;flex-shrink:0}
.input-wrap{flex:1;background:var(--bg3);border:1px solid var(--border);border-radius:10px;
            display:flex;align-items:flex-end;gap:8px;padding:8px 12px;transition:border-color .15s}
.input-wrap:focus-within{border-color:var(--purple)}
textarea{flex:1;background:transparent;border:none;outline:none;color:var(--text);
         font-size:13px;font-family:var(--font);resize:none;min-height:22px;max-height:120px;line-height:1.5}
textarea::placeholder{color:var(--text-faint)}
.send-btn{padding:6px 14px;background:var(--purple);color:#fff;border:none;border-radius:7px;
          cursor:pointer;font-size:12px;font-weight:600;transition:all .12s;flex-shrink:0}
.send-btn:hover{background:var(--purple-l)}
.send-btn:disabled{opacity:.4;cursor:not-allowed}
/* Model badge */
.model-badge{display:flex;align-items:center;gap:6px;padding:4px 10px;
             background:var(--bg3);border:1px solid var(--border);border-radius:20px;
             font-size:11px;color:var(--text-dim);cursor:pointer}
.model-badge:hover{border-color:var(--purple)}

/* ── Settings modal ── */
.modal-overlay{position:fixed;inset:0;background:rgba(0,0,0,.6);backdrop-filter:blur(4px);
               z-index:100;display:none;align-items:center;justify-content:center}
.modal-overlay.open{display:flex}
.modal{background:var(--bg2);border:1px solid var(--border);border-radius:14px;padding:28px;
       width:480px;box-shadow:0 20px 60px rgba(0,0,0,.5)}
.modal-title{font-size:16px;font-weight:700;margin-bottom:20px;display:flex;
             align-items:center;gap:10px}
.form-group{margin-bottom:16px}
label{display:block;font-size:12px;color:var(--text-dim);margin-bottom:5px;font-weight:500}
input,select{width:100%;padding:8px 11px;background:var(--bg3);border:1px solid var(--border);
             border-radius:7px;color:var(--text);font-size:13px;outline:none;font-family:var(--font)}
input:focus,select:focus{border-color:var(--purple)}
select option{background:var(--bg3)}
.form-row{display:flex;gap:12px}
.form-row .form-group{flex:1}
.modal-footer{display:flex;justify-content:flex-end;gap:10px;margin-top:22px}
.status-dot{width:8px;height:8px;border-radius:50%;transition:all .3s}
.status-dot.ok{background:var(--green);box-shadow:0 0 6px var(--green)}
.status-dot.err{background:var(--red);box-shadow:0 0 6px var(--red)}
.status-dot.checking{background:var(--yellow);animation:pulse 1s infinite}
@keyframes pulse{0%,100%{opacity:.5}50%{opacity:1}}
/* Scrollbar */
::-webkit-scrollbar{width:4px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:var(--border);border-radius:2px}
::-webkit-scrollbar-thumb:hover{background:var(--border-l)}
/* Titlebar drag */
.topbar{-webkit-app-region:drag}
.btn,.nav-item,.send-btn,.input-wrap,.instance,.modal{-webkit-app-region:no-drag}
</style>
</head>
<body>
<!-- ── Sidebar ── -->
<div class="sidebar">
  <div class="logo">
    <div class="logo-icon">SF</div>
    <div>
      <div class="logo-name">Software Factory</div>
      <div class="logo-ver">v3.1.0 · macOS · Rust</div>
    </div>
  </div>
  <div class="nav" id="nav">
    <div class="nav-group">Navigation</div>
    <div class="nav-item active" onclick="setView('chat')">
      <span class="dot dot-p"></span> Chat agents <span class="nav-badge" id="session-count">4</span>
    </div>
    <div class="nav-item" onclick="setView('missions')">
      <span class="dot dot-g"></span> Missions
    </div>
    <div class="nav-item" onclick="setView('cockpit')">
      <span class="dot dot-d"></span> Cockpit
    </div>
    <div class="nav-item" onclick="setView('workflows')">
      <span class="dot dot-d"></span> Workflows
    </div>
    <div class="nav-group">Agents récents</div>
    <div class="nav-item" onclick="selectAgent('Marc Lefevre','ML','Macaron tech architect')">
      <span class="dot dot-p"></span> Marc Lefevre
    </div>
    <div class="nav-item" onclick="selectAgent('Thomas Dubois','TD','Tech architect')">
      <span class="dot dot-p"></span> Thomas Dubois
    </div>
    <div class="nav-item" onclick="selectAgent('Emilie Chen','EC','Angular + Java/Spring')">
      <span class="dot dot-g"></span> Emilie Chen
    </div>
    <div class="nav-item" onclick="selectAgent('Claire Rousseau','CR','QA · E2E tests')">
      <span class="dot dot-d"></span> Claire Rousseau
    </div>
  </div>
  <div class="sb-footer">
    <div class="instance" onclick="openSettings()" title="Configurer">
      <div class="status-dot" id="dot-local"></div>
      <div class="instance-info">
        <div class="inst-name" id="llm-name">MLX LM — Qwen3</div>
        <div class="inst-url" id="llm-url">localhost:8080</div>
      </div>
    </div>
    <div class="instance" style="border-color:#3730a3">
      <div class="dot" style="background:#818cf8;box-shadow:0 0 5px #818cf8"></div>
      <div class="instance-info">
        <div class="inst-name">SF OVH</div>
        <div class="inst-url">sf.internal.app</div>
      </div>
    </div>
    <div class="instance" style="border-color:#1e3a2f">
      <div class="dot dot-g"></div>
      <div class="instance-info">
        <div class="inst-name">SF Azure</div>
        <div class="inst-url">az.macaron.app</div>
      </div>
    </div>
  </div>
</div>

<!-- ── Main ── -->
<div class="main">
  <div class="topbar">
    <div class="topbar-title">Software Factory</div>
    <div class="model-badge" onclick="openSettings()" id="model-badge">
      <div class="status-dot checking" id="conn-dot"></div>
      <span id="model-label">mlx-community/Qwen3</span>
    </div>
    <button class="btn btn-g" onclick="newSession()">+ Session</button>
    <button class="btn btn-p" onclick="newMission()">Mission</button>
    <button class="btn btn-g" onclick="openSettings()" title="Préférences">⚙</button>
  </div>

  <div class="stats">
    <div class="stat"><div class="stat-v">192</div><div class="stat-l">Agents</div></div>
    <div class="stat"><div class="stat-v">1286</div><div class="stat-l">Skills</div></div>
    <div class="stat"><div class="stat-v">10</div><div class="stat-l">Patterns</div></div>
    <div class="stat"><div class="stat-v">46</div><div class="stat-l">Workflows</div></div>
    <div class="stat"><div class="stat-v" style="color:var(--green)" id="token-count">0</div><div class="stat-l">Tokens</div></div>
    <div class="stat-reasoning">
      <span style="color:var(--purple-l);font-family:var(--mono);font-size:9px">arXiv:2603.01896</span>
      <span>P→T→V Reasoning</span>
    </div>
  </div>

  <div class="content">
    <!-- Sessions list -->
    <div class="sessions-panel">
      <div class="panel-hdr">
        <span class="panel-title">Sessions</span>
        <button class="btn btn-g" style="padding:2px 8px;font-size:11px" onclick="newSession()">+</button>
      </div>
      <div class="sessions-list" id="sessions-list">
        <div class="sess active" onclick="selectSession(this,'Architecture review API')">
          <div class="sess-title">Architecture review API</div>
          <div class="sess-meta"><div class="ss ss-a"></div>Marc Lefevre · 14:32</div>
        </div>
        <div class="sess" onclick="selectSession(this,'Migration PostgreSQL')">
          <div class="sess-title">Migration PostgreSQL → SF</div>
          <div class="sess-meta"><div class="ss ss-a"></div>Thomas Dubois · 13:15</div>
        </div>
        <div class="sess" onclick="selectSession(this,'CI/CD pipeline debug')">
          <div class="sess-title">CI/CD pipeline debug</div>
          <div class="sess-meta"><div class="ss ss-d"></div>Emilie Chen · hier</div>
        </div>
        <div class="sess" onclick="selectSession(this,'Unit tests adversarial')">
          <div class="sess-title">Unit tests adversarial guard</div>
          <div class="sess-meta"><div class="ss ss-d"></div>Claire Rousseau · hier</div>
        </div>
      </div>
    </div>

    <!-- Chat -->
    <div class="chat-area">
      <div class="chat-hdr">
        <div class="avatar" id="agent-av">ML</div>
        <div>
          <div class="agent-name" id="agent-name">Marc Lefevre</div>
          <div class="agent-role" id="agent-role">Macaron tech architect — FastAPI, HTMX, SSE, patterns</div>
        </div>
        <div class="chat-tags">
          <span class="tag tag-mlx" id="pattern-tag">sequential</span>
          <span class="tag" style="background:#150d2e;color:#c084fc;border:1px solid #5b21b6">L0/L1 guard</span>
        </div>
      </div>

      <div class="msgs" id="msgs">
        <div class="msg ai">
          <div class="msg-av ai" id="first-av">ML</div>
          <div class="bubble">
Bonjour. Je suis Marc Lefevre, architecte technique de la Software Factory.<br><br>
Je tourne sur <strong id="model-in-msg">MLX LM / Qwen3</strong> — directement sur votre Apple Silicon.<br><br>
Comment puis-je vous aider ? Architecture, code, déploiement, patterns agentiques…
          </div>
        </div>
      </div>

      <div class="input-area">
        <div class="input-wrap">
          <textarea id="input" placeholder="Message… (↵ envoyer · ⇧↵ nouvelle ligne)"
            onkeydown="handleKey(event)" oninput="autoResize(this)" rows="1"></textarea>
        </div>
        <button class="send-btn" id="send-btn" onclick="sendMsg()">Envoyer</button>
      </div>
    </div>
  </div>
</div>

<!-- ── Settings modal ── -->
<div class="modal-overlay" id="settings-modal" onclick="closeSettingsOutside(event)">
  <div class="modal" onclick="event.stopPropagation()">
    <div class="modal-title">
      ⚙ Préférences — Software Factory
    </div>
    <div class="form-row">
      <div class="form-group" style="flex:2">
        <label>Endpoint LLM (OpenAI-compatible)</label>
        <input id="s-endpoint" value="http://localhost:8080" placeholder="http://localhost:8080">
      </div>
      <div class="form-group" style="flex:0 0 auto;padding-top:20px">
        <div style="display:flex;align-items:center;gap:8px;padding-top:5px">
          <div class="status-dot" id="s-dot"></div>
          <span id="s-status" style="font-size:11px;color:var(--text-dim)">Non testé</span>
          <button class="btn btn-g" style="font-size:11px;padding:4px 10px" onclick="testConn()">Tester</button>
        </div>
      </div>
    </div>
    <div class="form-group">
      <label>Modèle</label>
      <input id="s-model" value="qwen3:8b" placeholder="qwen3:8b ou mlx-community/Qwen3-8B-4bit">
    </div>
    <div class="form-row">
      <div class="form-group">
        <label>Température (0.0 – 2.0)</label>
        <input id="s-temp" type="number" value="0.7" min="0" max="2" step="0.1">
      </div>
      <div class="form-group">
        <label>Max tokens</label>
        <input id="s-maxtok" type="number" value="4096" min="256" max="32768" step="256">
      </div>
    </div>
    <div class="form-group">
      <label>System prompt (optionnel — remplace le persona agent)</label>
      <input id="s-system" placeholder="Laissez vide pour utiliser le persona agent par défaut">
    </div>
    <div class="modal-footer">
      <button class="btn btn-g" onclick="closeSettings()">Annuler</button>
      <button class="btn btn-p" onclick="saveSettings()">Enregistrer</button>
    </div>
  </div>
</div>

<script>
// ── State ───────────────────────────────────────────────────────────────────
const CFG = {
  endpoint: localStorage.getItem('sf_endpoint') || 'http://localhost:8080',
  model:    localStorage.getItem('sf_model')    || 'qwen3:8b',
  temp:     parseFloat(localStorage.getItem('sf_temp')    || '0.7'),
  maxTok:   parseInt(  localStorage.getItem('sf_maxtok')  || '4096'),
  system:   localStorage.getItem('sf_system')  || '',
};
let isStreaming = false;
let totalTokens = 0;

// ── Current agent persona ────────────────────────────────────────────────
const AGENTS = {
  'Marc Lefevre': {
    initials:'ML',
    role:'Macaron tech architect — FastAPI, HTMX, SSE, patterns',
    system:`Tu es Marc Lefèvre, architecte technique de la Software Factory Macaron.
Stack: FastAPI, HTMX, SSE, Python, SQLite/PostgreSQL, Docker, Azure, OVH.
Réponds en français. Sois précis, concis, code-oriented. Cite les fichiers quand pertinent.`
  },
  'Thomas Dubois': {
    initials:'TD',
    role:'Technical architect, code quality guardian',
    system:`Tu es Thomas Dubois, architecte technique et gardien de la qualité de code.
Expert: architecture logicielle, clean code, design patterns, code review.
Réponds en français.`
  },
  'Emilie Chen': {
    initials:'EC',
    role:'Angular + Java/Spring, module migration, tests',
    system:`Tu es Emilie Chen, développeuse fullstack Angular + Java/Spring.
Experte en migration de modules, tests unitaires et intégration.
Réponds en français.`
  },
  'Claire Rousseau': {
    initials:'CR',
    role:'QA · E2E tests · golden files ISO 100%',
    system:`Tu es Claire Rousseau, ingénieure QA spécialisée en tests E2E.
Tu valides la qualité logicielle avec rigueur. Rappelle les bonnes pratiques de test.
Réponds en français.`
  },
};
let currentAgent = 'Marc Lefevre';
let history = []; // OpenAI message history

// ── Init ─────────────────────────────────────────────────────────────────
window.addEventListener('DOMContentLoaded', () => {
  applySettings();
  checkConnection();
  setInterval(checkConnection, 30000);
  document.getElementById('input').focus();
});

function applySettings() {
  document.getElementById('llm-url').textContent =
    CFG.endpoint.replace('http://','').replace('https://','');
  document.getElementById('model-label').textContent = CFG.model;
  document.getElementById('model-in-msg').textContent =
    'MLX LM / ' + CFG.model;
}

// ── Connection check ─────────────────────────────────────────────────────
async function checkConnection() {
  const dot = document.getElementById('conn-dot');
  const dotLocal = document.getElementById('dot-local');
  dot.className = 'status-dot checking';
  try {
    const r = await fetch(CFG.endpoint + '/v1/models', {signal: AbortSignal.timeout(3000)});
    if (r.ok) {
      dot.className = 'status-dot ok';
      dotLocal.className = 'status-dot ok';
      const data = await r.json();
      if (data.data && data.data[0]) {
        const modelName = data.data[0].id;
        // Auto-update model if not manually set
        const stored = localStorage.getItem('sf_model');
        if (!stored) {
          CFG.model = modelName;
          document.getElementById('model-label').textContent = modelName;
          document.getElementById('model-in-msg').textContent = 'MLX LM / ' + modelName;
        }
      }
    } else { dot.className = 'status-dot err'; dotLocal.className = 'status-dot err'; }
  } catch(e) { dot.className = 'status-dot err'; dotLocal.className = 'status-dot err'; }
}

// ── Chat ─────────────────────────────────────────────────────────────────
function handleKey(e) {
  if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMsg(); }
}

async function sendMsg() {
  if (isStreaming) return;
  const input = document.getElementById('input');
  const text = input.value.trim();
  if (!text) return;
  input.value = '';
  input.style.height = 'auto';

  appendMsg('user', text);
  history.push({role:'user', content: text});

  isStreaming = true;
  document.getElementById('send-btn').disabled = true;
  document.getElementById('send-btn').textContent = '…';

  // Build messages with system prompt
  const agentDef = AGENTS[currentAgent] || AGENTS['Marc Lefevre'];
  const systemPrompt = CFG.system || agentDef.system;
  const messages = [{role:'system', content: systemPrompt}, ...history];

  // Show typing
  const typingId = 'typing-' + Date.now();
  appendTyping(typingId);
  scrollToBottom();

  try {
    const resp = await fetch(CFG.endpoint + '/v1/chat/completions', {
      method: 'POST',
      headers: {'Content-Type':'application/json', 'Accept':'text/event-stream'},
      body: JSON.stringify({
        model: CFG.model,
        messages: messages,
        temperature: CFG.temp,
        max_tokens: CFG.maxTok,
        stream: true,
      })
    });

    removeTyping(typingId);

    if (!resp.ok) {
      const err = await resp.text();
      appendError(`Erreur LLM (${resp.status}): ${err.slice(0,200)}`);
      finishStream();
      return;
    }

    // Stream SSE tokens
    const msgId = appendMsg('ai', '');
    const bubble = document.getElementById(msgId);
    let fullContent = '';
    let inTokens = 0, outTokens = 0;

    const reader = resp.body.getReader();
    const dec = new TextDecoder();
    let buf = '';

    while(true) {
      const {done, value} = await reader.read();
      if (done) break;
      buf += dec.decode(value, {stream:true});

      const lines = buf.split('\n');
      buf = lines.pop(); // keep incomplete last line

      for (const line of lines) {
        if (!line.startsWith('data: ')) continue;
        const data = line.slice(6).trim();
        if (data === '[DONE]') break;
        try {
          const chunk = JSON.parse(data);
          const delta = chunk.choices?.[0]?.delta?.content || '';
          if (delta) {
            fullContent += delta;
            bubble.innerHTML = mdRender(fullContent);
            scrollToBottom();
          }
          if (chunk.usage) {
            inTokens = chunk.usage.prompt_tokens || 0;
            outTokens = chunk.usage.completion_tokens || 0;
          }
        } catch(e) {}
      }
    }

    if (!fullContent) fullContent = '(réponse vide)';
    bubble.innerHTML = mdRender(fullContent);
    history.push({role:'assistant', content: fullContent});

    totalTokens += inTokens + outTokens;
    document.getElementById('token-count').textContent = totalTokens.toLocaleString();

  } catch(e) {
    removeTyping(typingId);
    appendError(
      e.name === 'AbortError' || e.name === 'TypeError'
        ? `Impossible de joindre ${CFG.endpoint}\n\nVérifiez que MLX LM tourne :\n  mlx_lm.server --model mlx-community/Qwen2.5-7B-Instruct-4bit\nou via Ollama :\n  ollama serve`
        : `Erreur : ${e.message}`
    );
  }
  finishStream();
}

function finishStream() {
  isStreaming = false;
  const btn = document.getElementById('send-btn');
  btn.disabled = false;
  btn.textContent = 'Envoyer';
  document.getElementById('input').focus();
}

// ── DOM helpers ───────────────────────────────────────────────────────────
function appendMsg(role, content) {
  const msgs = document.getElementById('msgs');
  const id = 'msg-' + Date.now() + '-' + Math.random().toString(36).slice(2);
  const av = role === 'ai'
    ? `<div class="msg-av ai">${AGENTS[currentAgent]?.initials || 'AI'}</div>`
    : `<div class="msg-av user">U</div>`;
  const bClass = 'bubble';
  msgs.insertAdjacentHTML('beforeend',
    `<div class="msg ${role}">${av}<div class="${bClass}" id="${id}">${mdRender(content)}</div></div>`
  );
  scrollToBottom();
  return id;
}

function appendTyping(id) {
  const msgs = document.getElementById('msgs');
  const av = `<div class="msg-av ai">${AGENTS[currentAgent]?.initials || 'AI'}</div>`;
  msgs.insertAdjacentHTML('beforeend',
    `<div class="msg ai" id="${id}">${av}<div class="bubble"><div class="typing"><span></span><span></span><span></span></div></div></div>`
  );
}
function removeTyping(id) {
  const el = document.getElementById(id);
  if (el) el.remove();
}

function appendError(msg) {
  const msgs = document.getElementById('msgs');
  msgs.insertAdjacentHTML('beforeend',
    `<div class="msg ai"><div class="msg-av ai" style="background:var(--red)">!</div>
     <div class="bubble" style="border-color:var(--red);color:var(--red)">${escHtml(msg)}</div></div>`
  );
  scrollToBottom();
}

function scrollToBottom() {
  const msgs = document.getElementById('msgs');
  msgs.scrollTop = msgs.scrollHeight;
}

// ── Minimal markdown ──────────────────────────────────────────────────────
function mdRender(s) {
  if (!s) return '';
  return s
    .replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')
    // code blocks
    .replace(/```(\w*)\n?([\s\S]*?)```/g, (_,lang,code) =>
      `<pre><code>${code.trim()}</code></pre>`)
    // inline code
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    // bold
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    // italic
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    // headers
    .replace(/^### (.+)$/gm, '<strong style="font-size:14px">$1</strong>')
    .replace(/^## (.+)$/gm,  '<strong style="font-size:15px">$1</strong>')
    .replace(/^# (.+)$/gm,   '<strong style="font-size:16px">$1</strong>')
    // list items
    .replace(/^[•\-\*] (.+)$/gm, '&nbsp;• $1')
    // line breaks
    .replace(/\n/g, '<br>');
}
function escHtml(s) {
  return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/\n/g,'<br>');
}

// ── Navigation ────────────────────────────────────────────────────────────
function setView(v) {
  document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
  event.currentTarget.classList.add('active');
  if (v !== 'chat') {
    appendMsg('ai', `Vue **${v}** — non disponible en mode local.\nConnectez une instance SF distante pour accéder à cette vue.`);
  }
}

function selectAgent(name, initials, role) {
  currentAgent = name;
  history = []; // reset history for new agent
  document.getElementById('agent-name').textContent = name;
  document.getElementById('agent-role').textContent = role;
  document.getElementById('agent-av').textContent = initials;
  document.getElementById('first-av').textContent = initials;
  document.getElementById('msgs').innerHTML = `
    <div class="msg ai">
      <div class="msg-av ai">${initials}</div>
      <div class="bubble">
Bonjour. Je suis <strong>${name}</strong> — ${role}.<br><br>
Je tourne sur <strong id="model-in-msg">${CFG.model}</strong>. Comment puis-je vous aider ?
      </div>
    </div>`;
}

function selectSession(el, title) {
  document.querySelectorAll('.sess').forEach(s => s.classList.remove('active'));
  el.classList.add('active');
}

function newSession() {
  const title = 'Session ' + new Date().toLocaleTimeString('fr');
  const list = document.getElementById('sessions-list');
  const id = 'sess-' + Date.now();
  list.insertAdjacentHTML('afterbegin',
    `<div class="sess active" id="${id}" onclick="selectSession(this,'${title}')">
       <div class="sess-title">${title}</div>
       <div class="sess-meta"><div class="ss ss-a"></div>${currentAgent} · maintenant</div>
     </div>`);
  document.querySelectorAll('.sess').forEach(s => s.classList.remove('active'));
  document.getElementById(id).classList.add('active');
  history = [];
  document.getElementById('msgs').innerHTML = '';
  const count = document.querySelectorAll('.sess').length;
  document.getElementById('session-count').textContent = count;
}

function newMission() {
  appendMsg('ai', `**Nouvelle mission** — fonctionnalité disponible en Phase 3 (connexion instance distante SF).\n\nEn attendant, décrivez votre mission ici et je vous aide à la décomposer en tâches.`);
}

// ── Settings ─────────────────────────────────────────────────────────────
function openSettings() {
  document.getElementById('s-endpoint').value = CFG.endpoint;
  document.getElementById('s-model').value    = CFG.model;
  document.getElementById('s-temp').value     = CFG.temp;
  document.getElementById('s-maxtok').value   = CFG.maxTok;
  document.getElementById('s-system').value   = CFG.system;
  document.getElementById('settings-modal').classList.add('open');
  document.getElementById('s-status').textContent = 'Non testé';
  document.getElementById('s-dot').className = 'status-dot';
}
function closeSettings() {
  document.getElementById('settings-modal').classList.remove('open');
}
function closeSettingsOutside(e) {
  if (e.target.id === 'settings-modal') closeSettings();
}
function saveSettings() {
  CFG.endpoint = document.getElementById('s-endpoint').value.replace(/\/$/, '');
  CFG.model    = document.getElementById('s-model').value.trim();
  CFG.temp     = parseFloat(document.getElementById('s-temp').value);
  CFG.maxTok   = parseInt(document.getElementById('s-maxtok').value);
  CFG.system   = document.getElementById('s-system').value.trim();
  localStorage.setItem('sf_endpoint', CFG.endpoint);
  localStorage.setItem('sf_model',    CFG.model);
  localStorage.setItem('sf_temp',     CFG.temp);
  localStorage.setItem('sf_maxtok',   CFG.maxTok);
  localStorage.setItem('sf_system',   CFG.system);
  applySettings();
  closeSettings();
  checkConnection();
}
async function testConn() {
  const ep = document.getElementById('s-endpoint').value.replace(/\/$/, '');
  const dot = document.getElementById('s-dot');
  const status = document.getElementById('s-status');
  dot.className = 'status-dot checking';
  status.textContent = 'Test en cours…';
  try {
    const r = await fetch(ep + '/v1/models', {signal: AbortSignal.timeout(4000)});
    if (r.ok) {
      const data = await r.json();
      const models = (data.data || []).map(m => m.id).join(', ');
      dot.className = 'status-dot ok';
      status.textContent = `OK — ${models || 'connecté'}`;
    } else {
      dot.className = 'status-dot err';
      status.textContent = `HTTP ${r.status}`;
    }
  } catch(e) {
    dot.className = 'status-dot err';
    status.textContent = 'Inaccessible';
  }
}

function autoResize(el) {
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 120) + 'px';
}
</script>
</body>
</html>"##;
