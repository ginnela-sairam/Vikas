//! VIKAS STUDIO — the application template (raw string, placeholder-injected).
//! Keep in sync with studio.rs placeholders:
//! __TITLE__ __SIGS__ __CHG__ __END__ __NEV__ __CTX__ __BUILD__

pub const APP_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>VIKAS Studio — __TITLE__</title>
<style>
/* ============ VIKAS STUDIO design system ============ */
*{margin:0;padding:0;box-sizing:border-box}
:root{
  --bg:#0b0f1a;--bg2:#0d1322;--panel:#101829;--panel2:#141d31;--raised:#182342;
  --edge:rgba(148,163,184,.14);--edge2:rgba(148,163,184,.22);
  --txt:#e2e8f0;--dim:#94a3b8;--faint:#64748b;
  --accent:#22d3ee;--accent2:#38bdf8;--ok:#34d399;--warn:#fbbf24;--bad:#f87171;--gold:#e8b93f;
  --sel:#1d3a5f;--sel2:#265081;--cursorA:#f0f6ff;--cursorB:#34d399;
  --wave:#89e05a;--bus:#38bdf8;--clk:#22d3ee;--xcol:#ef7373;--zcol:#a98ff0;
  --shadow:0 12px 40px rgba(0,0,0,.55);
  --font:-apple-system,'Segoe UI',Roboto,Inter,system-ui,sans-serif;
  --mono:'JetBrains Mono',ui-monospace,'Cascadia Mono',Menlo,Consolas,monospace;
}
html[data-theme=verdi]{--bg:#171c26;--bg2:#1a212d;--panel:#1f2735;--panel2:#242e3f;--raised:#2b3649;
  --edge:rgba(200,210,225,.13);--txt:#d9dfeb;--dim:#8f9bb0;--accent:#e2b93b;--accent2:#f0d47a;--sel:#3a3320;--sel2:#4d4425}
html[data-theme=light]{--bg:#eef1f5;--bg2:#e8ecf2;--panel:#ffffff;--panel2:#f6f8fb;--raised:#eef2f8;
  --edge:#dde3ec;--edge2:#cbd5e1;--txt:#16233b;--dim:#5b6b83;--faint:#8b98ad;
  --accent:#0369a1;--accent2:#0284c7;--sel:#d8ebfa;--sel2:#c0dcf6;--cursorA:#123;--cursorB:#059669;
  --wave:#2f8f0f;--bus:#026fa5;--xcol:#c23a3a;--zcol:#6d54b8;--shadow:0 12px 32px rgba(30,50,90,.18)}
html[data-theme=synth]{--bg:#07070f;--bg2:#0a0b16;--panel:#0f1120;--panel2:#141630;--raised:#1b1e3d;
  --edge:rgba(150,120,255,.18);--txt:#eae6ff;--dim:#9d96c9;--accent:#ff2fb3;--accent2:#00e5ff;
  --sel:#33184a;--sel2:#48206b;--ok:#00ffb2;--warn:#ffd166;--bad:#ff5c7a;--gold:#ffd166;
  --wave:#00e5a7;--bus:#7aa2ff;--clk:#00e5ff}
body{background:var(--bg);color:var(--txt);font:12.5px/1.45 var(--font);height:100vh;overflow:hidden;
  -webkit-font-smoothing:antialiased;user-select:none;-webkit-user-select:none}
#app{display:flex;flex-direction:column;height:100vh}
::-webkit-scrollbar{width:9px;height:9px}
::-webkit-scrollbar-thumb{background:rgba(148,163,184,.25);border-radius:5px;border:2px solid transparent;background-clip:content-box}
::-webkit-scrollbar-thumb:hover{background:rgba(148,163,184,.45);background-clip:content-box;border:2px solid transparent}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-corner{background:transparent}

/* ============ header ============ */
#titlebar{display:flex;align-items:center;gap:2px;height:30px;background:var(--panel);border-bottom:1px solid var(--edge);padding:0 8px;flex:none}
#brandmark{display:flex;align-items:center;gap:7px;padding:0 10px 0 2px;margin-right:4px;border-right:1px solid var(--edge);height:100%}
#brandmark .bm-name{font-weight:700;font-size:12.5px;letter-spacing:.12em;color:var(--txt)}
#brandmark .bm-name b{color:var(--accent);font-weight:800}
#brandmark .bm-sub{font-size:9.5px;color:var(--faint);letter-spacing:.06em;margin-top:-2px}
#hdrchips{display:flex;gap:6px;align-items:center;margin-left:10px}
.hchip{display:inline-flex;align-items:center;gap:5px;font-size:10.5px;color:var(--dim);
  background:var(--panel2);border:1px solid var(--edge);border-radius:4px;padding:2px 8px;white-space:nowrap}
.hchip svg{width:11px;height:11px}
.hchip b{color:var(--txt);font-weight:600;font-family:var(--mono)}
.badge-verdict{display:inline-flex;align-items:center;gap:5px;font-size:10.5px;font-weight:700;letter-spacing:.05em;
  border-radius:4px;padding:2.5px 10px;text-transform:uppercase}
.bv-ok{background:rgba(52,211,153,.13);color:var(--ok);border:1px solid rgba(52,211,153,.45)}
.bv-warn{background:rgba(251,191,36,.12);color:var(--warn);border:1px solid rgba(251,191,36,.4)}
.bv-bad{background:rgba(248,113,113,.13);color:var(--bad);border:1px solid rgba(248,113,113,.45)}
#hdrsp{flex:1}
#hdrbuild{font-size:10px;color:var(--faint);font-family:var(--mono);padding-right:8px;white-space:nowrap}

/* ============ tabs ============ */
#tabstrip{display:none;align-items:center;gap:4px;height:32px;background:var(--bg2);border-bottom:1px solid var(--edge);padding:0 10px;flex:none}
.has-hub #tabstrip{display:flex}
.vtab{display:inline-flex;align-items:center;gap:6px;font-size:11.5px;color:var(--dim);padding:6px 14px;border-radius:6px 6px 0 0;cursor:pointer;border:1px solid transparent;border-bottom:none;margin-top:5px}
.vtab:hover{color:var(--txt);background:var(--panel)}
.vtab.on{color:var(--accent);background:var(--panel);border-color:var(--edge);font-weight:600}
.vtab svg{width:13px;height:13px}

/* ============ menubar ============ */
#menubar{display:flex;align-items:center;height:28px;background:var(--panel);border-bottom:1px solid var(--edge);padding:0 6px;flex:none}
.menu-item{position:relative}
.menu-btn{font-size:12px;color:var(--txt);padding:4px 11px;border-radius:5px;cursor:default;letter-spacing:.01em}
.menu-btn:hover,.menu-open>.menu-btn{background:var(--sel);color:#fff}
html[data-theme=light] .menu-btn:hover,html[data-theme=light] .menu-open>.menu-btn{color:#123}
.menu-drop{display:none;position:absolute;top:100%;left:0;min-width:230px;background:var(--panel2);
  border:1px solid var(--edge2);border-radius:8px;box-shadow:var(--shadow);padding:5px;z-index:300}
.menu-open>.menu-drop{display:block}
.mi{display:flex;align-items:center;gap:9px;padding:5.5px 10px;border-radius:5px;font-size:12px;color:var(--txt);cursor:default;white-space:nowrap}
.mi:hover{background:var(--sel)}
.mi.disabled{color:var(--faint);pointer-events:none}
.mi svg{width:14px;height:14px;color:var(--dim);flex:none}
.mi .sc{margin-left:auto;font-size:10px;color:var(--faint);font-family:var(--mono);padding-left:18px}
.mi .sub{margin-left:auto;color:var(--faint);padding-left:14px}
.msep{height:1px;background:var(--edge);margin:5px 7px}

/* ============ toolbar ============ */
#toolbar{display:flex;align-items:center;gap:3px;height:42px;background:var(--panel);border-bottom:1px solid var(--edge);padding:0 8px;flex:none}
.tbtn{width:30px;height:30px;display:inline-flex;align-items:center;justify-content:center;border-radius:6px;
  cursor:pointer;color:var(--dim);position:relative;border:1px solid transparent;background:none}
.tbtn:hover{background:var(--raised);color:var(--txt)}
.tbtn.on{background:var(--sel);color:var(--accent);border-color:var(--edge2)}
.tbtn:active{transform:translateY(1px)}
.tbtn svg{width:17px;height:17px}
.tsep{width:1px;height:20px;background:var(--edge);margin:0 5px;flex:none}
.tlabel{display:inline-flex;align-items:center;gap:6px;font-size:11.5px;color:var(--dim);cursor:pointer;
  padding:5px 10px;border-radius:6px;border:1px solid var(--edge);background:var(--panel2)}
.tlabel:hover{background:var(--raised);color:var(--txt)}
.tlabel svg{width:13px;height:13px}
#searchwrap{display:flex;align-items:center;background:var(--bg2);border:1px solid var(--edge2);border-radius:7px;height:30px;width:330px;margin-left:6px;transition:width .15s}
#searchwrap:focus-within{width:400px;border-color:var(--accent)}
#searchwrap svg{width:14px;height:14px;color:var(--dim);margin:0 8px;flex:none}
#searchwrap input{flex:1;background:none;border:none;outline:none;color:var(--txt);font:11.5px var(--mono);min-width:0}
#searchwrap input::placeholder{color:var(--faint)}
#searchmode{font-size:9.5px;color:var(--dim);border-left:1px solid var(--edge);padding:0 8px;height:100%;display:flex;align-items:center;gap:4px;cursor:pointer}
#searchmode:hover{color:var(--accent)}
#toolsp{flex:1}
#statsmini{display:flex;gap:10px;font:10.5px var(--mono);color:var(--dim);align-items:center;padding-right:6px}
#statsmini b{color:var(--txt);font-weight:600}
.tsel{background:var(--bg2);color:var(--txt);border:1px solid var(--edge2);border-radius:6px;height:30px;font:11px var(--font);padding:0 4px;outline:none;cursor:pointer}
.tsel:hover{border-color:var(--accent)}
.tsel option{background:var(--panel2);color:var(--txt)}

/* ============ main layout ============ */
#main{flex:1;display:flex;min-height:0;position:relative}
#treedock{width:270px;background:var(--panel);border-right:1px solid var(--edge);display:flex;flex-direction:column;min-width:180px}
#propdock{width:288px;background:var(--panel);border-left:1px solid var(--edge);display:flex;flex-direction:column;min-width:0}
.dock-title{display:flex;align-items:center;gap:7px;height:30px;padding:0 10px;font-size:10.5px;font-weight:700;
  letter-spacing:.1em;color:var(--dim);border-bottom:1px solid var(--edge);text-transform:uppercase;flex:none;background:var(--panel)}
.dock-title svg{width:13px;height:13px;color:var(--accent)}
.dock-title .dt-btns{margin-left:auto;display:flex;gap:2px}
.dt-btn{width:22px;height:22px;display:inline-flex;align-items:center;justify-content:center;border-radius:5px;cursor:pointer;color:var(--faint)}
.dt-btn:hover{background:var(--raised);color:var(--txt)}
.dt-btn svg{width:13px;height:13px}
#treefilter{margin:8px 8px 4px;display:flex;align-items:center;gap:6px;background:var(--bg2);border:1px solid var(--edge);border-radius:6px;height:26px;padding:0 8px}
#treefilter svg{width:12px;height:12px;color:var(--dim);flex:none}
#treefilter input{flex:1;background:none;border:none;outline:none;color:var(--txt);font:11px var(--mono)}
#treefilter input::placeholder{color:var(--faint)}
#treestats{font-size:9.5px;color:var(--faint);padding:3px 12px 4px;font-family:var(--mono);flex:none}
#tree{flex:1;overflow-y:auto;overflow-x:hidden;padding:2px 4px 30px}
.scope{margin-top:2px}
.scope-h{display:flex;align-items:center;gap:5px;padding:3px 6px;border-radius:5px;font-size:11px;color:var(--accent2);cursor:pointer;white-space:nowrap;font-weight:600}
.scope-h:hover{background:var(--raised)}
.scope-h svg{width:11px;height:11px;transition:transform .12s}
.scope.closed .scope-h svg{transform:rotate(-90deg)}
.scope.closed .scope-kids{display:none}
.scope-h .sc-n{color:var(--faint);font-weight:400;font-size:10px;margin-left:auto;font-family:var(--mono)}
.sigrow{display:flex;align-items:center;gap:7px;padding:2.5px 6px 2.5px 24px;border-radius:5px;cursor:pointer;
  white-space:nowrap;font:11px var(--mono);color:var(--txt);overflow:hidden}
.sigrow>span:first-of-type{flex:0 0 auto}
.sigrow>span:nth-of-type(2){flex:0 1 auto;overflow:hidden;text-overflow:ellipsis}
.sigrow:hover{background:var(--raised)}
.sigrow.inview{background:var(--sel);box-shadow:inset 2px 0 0 var(--accent)}
.sigrow .kdot{width:7px;height:7px;border-radius:2px;flex:none}
.sigrow .sr-w{color:var(--faint);font-size:9.5px;margin-left:auto;padding-left:8px;flex:none}
.sigrow .sr-add{opacity:0;width:16px;height:16px;border-radius:4px;display:inline-flex;align-items:center;justify-content:center;color:var(--dim);flex:none}
.sigrow:hover .sr-add{opacity:1}
.sigrow .sr-add:hover{background:var(--accent);color:#012}
.sigrow .sr-add svg{width:11px;height:11px}

/* ============ wave panel ============ */
#wavepanel{flex:1;display:flex;flex-direction:column;min-width:0;background:var(--bg)}
#canvaswrap{flex:1;position:relative;min-height:0;overflow:hidden}
#wv{display:block;cursor:default}
#minimap{height:46px;background:var(--panel);border-top:1px solid var(--edge);position:relative;flex:none;cursor:pointer}
#minimap canvas{display:block;width:100%;height:100%}
#mmhint{position:absolute;right:8px;top:4px;font-size:9px;color:var(--faint);letter-spacing:.06em;pointer-events:none}
#gauge{height:22px;background:var(--panel);border-top:1px solid var(--edge);display:flex;align-items:center;
  font:10px var(--mono);color:var(--dim);flex:none;overflow:hidden}
#gauge .gc{padding:0 10px;border-right:1px solid var(--edge);height:100%;display:flex;align-items:center;gap:6px;white-space:nowrap}
#gauge .gc b{color:var(--txt);font-weight:600}
#gauge .gsp{flex:1}
#gauge .ghint{color:var(--faint);padding-right:10px;white-space:nowrap}
.kbd{display:inline-block;background:var(--bg2);border:1px solid var(--edge2);border-bottom-width:2px;border-radius:4px;
  padding:0 5px;font:9.5px var(--mono);color:var(--dim);margin:0 1px}

/* ============ status bar ============ */
#statusbar{display:flex;align-items:center;height:25px;background:var(--panel);border-top:1px solid var(--edge);
  font:10.5px var(--mono);color:var(--dim);padding:0 4px;flex:none}
.sb{display:flex;align-items:center;gap:6px;padding:0 10px;height:100%;border-right:1px solid var(--edge);white-space:nowrap}
.sb svg{width:11px;height:11px}
.sb b{color:var(--txt);font-weight:600}
.sb.ok b{color:var(--ok)}.sb.warn b{color:var(--warn)}.sb.bad b{color:var(--bad)}
#sbsp{flex:1}
#sbhint{color:var(--faint);padding-right:10px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:480px}

/* ============ properties dock ============ */
#props{flex:1;overflow-y:auto;padding:10px}
.psec{margin-bottom:14px}
.psec-h{display:flex;align-items:center;gap:7px;font-size:10px;font-weight:700;letter-spacing:.1em;color:var(--accent);
  text-transform:uppercase;margin-bottom:7px;padding-bottom:4px;border-bottom:1px solid var(--edge)}
.psec-h svg{width:12px;height:12px}
.prow{display:flex;justify-content:space-between;gap:10px;padding:3.5px 2px;font-size:11px;border-radius:4px}
.prow:hover{background:var(--raised)}
.prow .pk{color:var(--dim);white-space:nowrap}
.prow .pv{color:var(--txt);font-family:var(--mono);text-align:right;word-break:break-all}
.prow .pv.big{font-size:13px;font-weight:600;color:var(--accent)}
.pbtnrow{display:flex;gap:6px;margin-top:8px;flex-wrap:wrap}
.pbtn{display:inline-flex;align-items:center;gap:6px;font-size:11px;padding:5px 11px;border-radius:6px;cursor:pointer;
  background:var(--panel2);border:1px solid var(--edge2);color:var(--txt)}
.pbtn:hover{background:var(--raised);border-color:var(--accent);color:var(--accent)}
.pbtn svg{width:12px;height:12px}
.swatch-row{display:flex;gap:5px;flex-wrap:wrap}
.swatch{width:20px;height:20px;border-radius:5px;cursor:pointer;border:1px solid var(--edge2)}
.swatch:hover{transform:scale(1.15)}
.pval-table{width:100%;border-collapse:collapse;font:11px var(--mono)}
.pval-table td{padding:3px 6px;border-bottom:1px solid var(--edge)}
.pval-table td:first-child{color:var(--dim);width:64px}
.pval-table td:last-child{color:var(--txt);text-align:right;word-break:break-all}

/* ============ console dock ============ */
#consoledock{height:0;background:var(--panel);border-top:1px solid transparent;display:flex;flex-direction:column;overflow:hidden;transition:height .15s;flex:none}
#consoledock.open{height:160px;border-top-color:var(--edge)}
#consoleout{flex:1;overflow-y:auto;padding:8px 12px;font:11px var(--mono);color:var(--dim);user-select:text;-webkit-user-select:text}
#consoleout .cline{padding:1px 0}
#consoleout .c-ok{color:var(--ok)}#consoleout .c-bad{color:var(--bad)}#consoleout .c-warn{color:var(--warn)}#consoleout .c-dim{color:var(--faint)}

/* ============ overlays ============ */
#ctxmenu{display:none;position:fixed;z-index:500;background:var(--panel2);border:1px solid var(--edge2);
  border-radius:8px;box-shadow:var(--shadow);padding:5px;min-width:190px}
#ctxmenu .mi .sub{font-size:12px}
#tip{position:fixed;z-index:400;background:var(--panel2);border:1px solid var(--edge2);border-radius:8px;
  padding:8px 11px;font:11px var(--mono);pointer-events:none;display:none;box-shadow:var(--shadow);max-width:380px}
#tip .tn{color:var(--accent);font-weight:700;margin-bottom:3px;word-break:break-all}
#tip .tv{color:var(--ok)}#tip .td{color:var(--dim)}#tip .tx{color:var(--faint);margin-top:3px;font-size:10px}
#toasts{position:fixed;top:38px;right:12px;z-index:600;display:flex;flex-direction:column;gap:8px;pointer-events:none}
.toast{display:flex;align-items:center;gap:9px;background:var(--panel2);border:1px solid var(--edge2);
  border-left:3px solid var(--accent);border-radius:8px;padding:9px 14px;font-size:12px;color:var(--txt);
  box-shadow:var(--shadow);animation:tin .18s ease-out;max-width:420px}
.toast svg{width:14px;height:14px;color:var(--accent);flex:none}
.toast.ok{border-left-color:var(--ok)}.toast.ok svg{color:var(--ok)}
.toast.bad{border-left-color:var(--bad)}.toast.bad svg{color:var(--bad)}
.toast.warn{border-left-color:var(--warn)}.toast.warn svg{color:var(--warn)}
@keyframes tin{from{opacity:0;transform:translateX(16px)}to{opacity:1;transform:none}}
.toast.out{animation:tout .18s ease-in forwards}
@keyframes tout{to{opacity:0;transform:translateX(16px)}}
dialog{border:none;background:var(--panel);color:var(--txt);border-radius:12px;box-shadow:var(--shadow);
  padding:0;max-width:86vw}
dialog::backdrop{background:rgba(2,6,14,.66)}
.dlg-h{display:flex;align-items:center;gap:10px;padding:13px 18px;border-bottom:1px solid var(--edge);background:var(--panel2);border-radius:12px 12px 0 0}
.dlg-h svg{width:16px;height:16px;color:var(--accent)}
.dlg-h .dlg-t{font-size:13.5px;font-weight:700;letter-spacing:.02em}
.dlg-h .dlg-x{margin-left:auto;cursor:pointer;color:var(--faint);width:24px;height:24px;display:flex;align-items:center;justify-content:center;border-radius:6px}
.dlg-h .dlg-x:hover{background:var(--raised);color:var(--txt)}
.dlg-b{padding:16px 20px;max-height:74vh;overflow:auto}
.dlg-f{display:flex;justify-content:flex-end;gap:8px;padding:12px 18px;border-top:1px solid var(--edge)}
table.keys{border-collapse:collapse;width:100%}
table.keys td{padding:4.5px 10px;border-bottom:1px solid var(--edge);font-size:12px;color:var(--dim)}
table.keys td:first-child{white-space:nowrap}
table.keys td:last-child{color:var(--txt)}
table.keys tr:hover td{background:var(--raised)}
.kgroup{margin:14px 0 5px;font-size:10.5px;font-weight:700;letter-spacing:.1em;color:var(--accent);text-transform:uppercase}
.kgroup:first-child{margin-top:0}

/* about */
#aboutlogo{display:flex;flex-direction:column;align-items:center;gap:10px;padding:26px 10px 8px}
#aboutname{font-size:26px;font-weight:800;letter-spacing:.2em}
#aboutname b{color:var(--accent)}
#aboutsub{font-size:11.5px;color:var(--dim);letter-spacing:.08em;text-align:center;line-height:1.7;padding:0 24px 8px}
#aboutstats{display:flex;gap:14px;justify-content:center;padding:14px;margin:10px 18px 4px;background:var(--bg2);border-radius:10px;border:1px solid var(--edge)}
#aboutstats div{text-align:center}
#aboutstats .as-v{font:700 17px var(--mono);color:var(--accent)}
#aboutstats .as-k{font-size:9.5px;color:var(--faint);letter-spacing:.08em;text-transform:uppercase;margin-top:2px}

/* palette */
#palwrap{display:none;position:fixed;inset:0;z-index:700;background:rgba(2,6,14,.5)}
#palette{position:absolute;left:50%;top:14%;transform:translateX(-50%);width:560px;max-width:92vw;background:var(--panel2);
  border:1px solid var(--edge2);border-radius:12px;box-shadow:var(--shadow);overflow:hidden}
#palin{display:flex;align-items:center;gap:10px;padding:13px 16px;border-bottom:1px solid var(--edge)}
#palin svg{width:16px;height:16px;color:var(--accent)}
#palin input{flex:1;background:none;border:none;outline:none;color:var(--txt);font:13px var(--font)}
#pallist{max-height:380px;overflow-y:auto;padding:6px}
.palitem{display:flex;align-items:center;gap:11px;padding:8px 12px;border-radius:7px;font-size:12.5px;cursor:pointer;color:var(--dim)}
.palitem svg{width:15px;height:15px;flex:none}
.palitem .psc{margin-left:auto;font:10px var(--mono);color:var(--faint)}
.palitem.on{background:var(--sel);color:var(--txt)}
.palitem.on svg{color:var(--accent)}
.palgroup{font-size:9.5px;font-weight:700;letter-spacing:.12em;color:var(--faint);text-transform:uppercase;padding:8px 12px 3px}

/* results panel */
#resultsdock{display:none;position:absolute;top:0;right:0;bottom:0;width:330px;background:var(--panel);
  border-left:1px solid var(--edge);z-index:150;flex-direction:column}
#resultsdock.open{display:flex}
#reslist{flex:1;overflow-y:auto;padding:6px}
.resitem{display:flex;gap:9px;padding:6px 9px;border-radius:6px;cursor:pointer;font:11px var(--mono);align-items:baseline}
.resitem:hover{background:var(--raised)}
.resitem .rt{color:var(--accent);min-width:64px}
.resitem .rd{color:var(--dim);font-size:10px}

/* hub */
#hubscroll{flex:1;overflow-y:auto;padding:22px 26px}
#hubhead{display:flex;align-items:baseline;gap:14px;margin-bottom:4px}
#hubhead h2{font-size:19px;font-weight:800;letter-spacing:.02em}
#hubhead .hub-sub{color:var(--dim);font-size:12px}
#hubgrid{display:grid;grid-template-columns:repeat(auto-fill,minmax(310px,1fr));gap:16px;margin-top:16px}
.hubcard{background:var(--panel);border:1px solid var(--edge);border-radius:12px;padding:15px 16px;position:relative;overflow:hidden}
.hubcard::before{content:'';position:absolute;left:0;top:0;bottom:0;width:3px;background:var(--accent);opacity:.75}
.hubcard.h-ok::before{background:var(--ok)}.hubcard.h-warn::before{background:var(--warn)}.hubcard.h-bad::before{background:var(--bad)}
.hc-h{display:flex;align-items:center;gap:9px;margin-bottom:10px}
.hc-h svg{width:16px;height:16px;color:var(--accent)}
.hc-h .hc-t{font-size:12.5px;font-weight:700;letter-spacing:.03em}
.hc-h .hc-v{margin-left:auto;font:700 15px var(--mono);color:var(--txt)}
.hc-big{display:flex;align-items:baseline;gap:8px;margin:2px 0 8px}
.hc-big .n{font:800 30px var(--mono);color:var(--accent);line-height:1}
.hc-big .u{font-size:11px;color:var(--dim)}
.hc-note{font-size:11px;color:var(--dim);line-height:1.6}
.hbar{height:7px;background:var(--bg2);border-radius:4px;overflow:hidden;margin:4px 0}
.hbar>div{height:100%;border-radius:4px;background:var(--accent)}
.hrow{display:flex;align-items:center;gap:8px;font:10.5px var(--mono);color:var(--dim);padding:2.5px 0}
.hrow .hn{flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--txt)}
.hrow .hv{color:var(--accent);min-width:56px;text-align:right}
.hrow .hbar{flex:0 0 84px;margin:0}
table.hubtab{width:100%;border-collapse:collapse;font:11px var(--mono)}
table.hubtab th{text-align:left;color:var(--dim);font-weight:600;padding:4px 8px;border-bottom:1px solid var(--edge2);font-size:10px;letter-spacing:.06em;text-transform:uppercase;font-family:var(--font)}
table.hubtab td{padding:4px 8px;border-bottom:1px solid var(--edge);color:var(--txt);vertical-align:top}
table.hubtab td.dim{color:var(--dim)}
table.hubtab tr:hover td{background:var(--raised)}
.tag{display:inline-block;font-size:9.5px;padding:1px 7px;border-radius:4px;background:var(--bg2);border:1px solid var(--edge);color:var(--dim);font-family:var(--font)}
.tag.ok{color:var(--ok);border-color:rgba(52,211,153,.4)}
.tag.bad{color:var(--bad);border-color:rgba(248,113,113,.4)}
.tag.warn{color:var(--warn);border-color:rgba(251,191,36,.35)}
#hub-sec-h{display:flex;align-items:center;gap:10px;margin:26px 0 8px}
#hub-sec-h svg{width:15px;height:15px;color:var(--accent)}
#hub-sec-h h3{font-size:13px;font-weight:700;letter-spacing:.05em;text-transform:uppercase;color:var(--txt)}
#hub-sec-h .sec-line{flex:1;height:1px;background:var(--edge)}
.hub-empty{color:var(--faint);font-size:11.5px;padding:14px;text-align:center;border:1px dashed var(--edge2);border-radius:10px;line-height:1.8}
@media (max-width:1100px){#propdock{display:none}#searchwrap{width:220px}#searchwrap:focus-within{width:280px}}
</style>
</head>
<body>
<div id="app">

<!-- ============ title bar ============ -->
<div id="titlebar">
  <div id="brandmark">
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
      <path d="M12 2 L21 7 L21 15 L12 22 L3 15 L3 7 Z" stroke="var(--accent)"/>
      <path d="M7.5 8.5 L12 15.5 L16.5 8.5" stroke="var(--accent)"/>
    </svg>
    <div>
      <div class="bm-name">VIKAS <b>STUDIO</b></div>
      <div class="bm-sub">__TITLE__</div>
    </div>
  </div>
  <div id="hdrchips"></div>
  <div id="hdrsp"></div>
  <span id="hdrbuild">__BUILD__</span>
</div>

<!-- ============ tab strip ============ -->
<div id="tabstrip">
  <div class="vtab on" data-tab="wave"><span data-ic="activity"></span>Waveforms</div>
  <div class="vtab" data-tab="hub"><span data-ic="grid"></span>Analysis Hub</div>
  <div class="vtab" data-tab="intel"><span data-ic="database"></span>Design Intel</div>
</div>

<!-- ============ menu bar ============ -->
<div id="menubar"></div>

<!-- ============ toolbar ============ -->
<div id="toolbar">
  <span class="tbtn" data-ic="open"   data-act="open"        title="Open dump (.vwf .vcd .json layout)  Ctrl+O"></span>
  <span class="tbtn" data-ic="save"   data-act="save-layout" title="Save view layout  Ctrl+S"></span>
  <span class="tsep"></span>
  <span class="tbtn" data-ic="fit"    data-act="fit"         title="Fit all  F"></span>
  <span class="tbtn" data-ic="zin"    data-act="zin"         title="Zoom in  +"></span>
  <span class="tbtn" data-ic="zout"   data-act="zout"        title="Zoom out  -"></span>
  <span class="tbtn" data-ic="region" data-act="region"     title="Marquee zoom: drag on wave area  Z"></span>
  <span class="tsep"></span>
  <span class="tbtn" data-ic="prev"  data-act="prev-edge"   title="Previous edge of selected  ←"></span>
  <span class="tbtn" data-ic="next"  data-act="next-edge"   title="Next edge of selected  →"></span>
  <span class="tbtn" data-ic="cursor"   data-act="cursor-a"  title="Place cursor A: click wave  (click)"></span>
  <span class="tbtn" data-ic="measure" data-act="cursor-b"  title="Place cursor B: shift-click wave · Δt + f HUD"></span>
  <span class="tbtn" data-ic="cursorClear" data-act="clear-cursors" title="Clear cursors  C"></span>
  <span class="tsep"></span>
  <span class="tbtn" data-ic="group"   data-act="group"      title="Group selected rows  G"></span>
  <span class="tbtn" data-ic="ungroup" data-act="ungroup"   title="Ungroup selected rows  U"></span>
  <span class="tbtn" data-ic="expand"  data-act="expand"    title="Expand bus into bits  E"></span>
  <span class="tbtn" data-ic="collapse" data-act="collapse" title="Collapse expanded bus  Shift+E"></span>
  <span class="tsep"></span>
  <span class="tbtn" data-ic="analog" data-act="analog"     title="Toggle analog rendering for selected bus  A"></span>
  <span class="tbtn" data-ic="palette" data-act="color"     title="Set color of selected rows"></span>
  <div id="searchwrap">
    <span data-ic="search"></span>
    <input id="q" placeholder="search signal · boolean expr · goto:123ns" spellcheck="false"/>
    <span id="searchmode" title="toggle search mode">SIGNAL</span>
  </div>
  <span class="tbtn" data-ic="play" data-act="search-go" title="Run search  Enter"></span>
  <span class="tbtn" data-ic="prev" data-act="prev-match" style="width:24px" title="Previous result  ["></span>
  <span class="tbtn" data-ic="next" data-act="next-match" style="width:24px" title="Next result  ]"></span>
  <span class="tsep"></span>
  <span class="tbtn" data-ic="download" data-act="export" title="Export PNG / SVG / CSV / JSON / WaveDrom"></span>
  <span class="tbtn" data-ic="terminal" data-act="console" title="Toggle console  `"></span>
  <span class="tbtn" data-ic="moon" data-act="theme" title="Cycle theme: VIKAS Dark · Verdi · Light · Synth  T"></span>
  <span id="toolsp"></span>
  <div id="statsmini"></div>
  <select id="radixsel" class="tsel" title="Default radix for new rows">
    <option value="hex">0x hex</option><option value="bin">0b bin</option>
    <option value="dec">u decimal</option><option value="signed">signed</option>
    <option value="oct">0o oct</option><option value="ascii">ASCII</option>
  </select>
  <span class="tbtn" data-ic="help" data-act="help" title="Keyboard shortcuts  F1"></span>
  <span class="tbtn" data-ic="zap" data-act="palette-open" title="Command palette  Ctrl+K" style="color:var(--accent)"></span>
</div>

<!-- ============ main ============ -->
<div id="main">

  <!-- signal tree dock -->
  <div id="treedock">
    <div class="dock-title"><span data-ic="list"></span>Signals
      <div class="dt-btns">
        <span class="dt-btn" data-ic="check" data-act="add-all" title="Add all signals to view"></span>
        <span class="dt-btn" data-ic="plus" data-act="add-top" title="Add top-level I/O + clocks"></span>
        <span class="dt-btn" data-ic="trash" data-act="clear-view" title="Clear wave view"></span>
      </div>
    </div>
    <div id="treefilter"><span data-ic="search"></span><input id="tf" placeholder="filter signals (regex ok)"/></div>
    <div id="treestats"></div>
    <div id="tree"></div>
  </div>

  <!-- wave panel -->
  <div id="wavepanel">
    <div id="canvaswrap"><canvas id="wv"></canvas></div>
    <div id="minimap"><canvas id="mm"></canvas><span id="mmhint">NAVIGATOR — drag viewport · density histogram</span></div>
    <div id="gauge"></div>
    <div id="resultsdock">
      <div class="dock-title"><span data-ic="search"></span><span id="restitle">Results</span>
        <div class="dt-btns"><span class="dt-btn" data-ic="close" data-act="close-results" title="Close"></span></div>
      </div>
      <div id="reslist"></div>
    </div>
  </div>

  <!-- properties dock -->
  <div id="propdock">
    <div class="dock-title"><span data-ic="info"></span>Properties
      <div class="dt-btns"><span class="dt-btn" data-ic="up" data-act="row-up" title="Move selection up"></span>
      <span class="dt-btn" data-ic="down" data-act="row-down" title="Move selection down"></span></div>
    </div>
    <div id="props"></div>
  </div>
</div>

<!-- ============ console + status ============ -->
<div id="consoledock">
  <div class="dock-title"><span data-ic="terminal"></span>Console
    <div class="dt-btns"><span class="dt-btn" data-ic="close" data-act="console" title="Close console"></span></div>
  </div>
  <div id="consoleout"></div>
</div>
<div id="statusbar">
  <div class="sb" id="sb-time"><span data-ic="clock"></span>t = <b>0</b> ns</div>
  <div class="sb" id="sb-cursor"><span data-ic="cursor"></span>A: <b>—</b></div>
  <div class="sb" id="sb-dt"><span data-ic="measure"></span>Δt: <b>—</b></div>
  <div class="sb" id="sb-zoom"><span data-ic="fit"></span>zoom <b>100%</b></div>
  <div class="sb" id="sb-events"><span data-ic="activity"></span><b>0</b> events</div>
  <div id="sbsp"></div>
  <div id="sbhint"></div>
</div>
</div>

<!-- ============ overlays ============ -->
<div id="ctxmenu"></div>
<div id="tip"></div>
<div id="toasts"></div>

<div id="palwrap">
  <div id="palette">
    <div id="palin"><span data-ic="zap"></span><input id="palq" placeholder="type a command…" spellcheck="false"/></div>
    <div id="pallist"></div>
  </div>
</div>

<dialog id="dlg-help">
  <div class="dlg-h"><span data-ic="help"></span><span class="dlg-t">Keyboard &amp; Mouse</span><span class="dlg-x" data-ic="close"></span></div>
  <div class="dlg-b" id="helpbody"></div>
</dialog>
<dialog id="dlg-about">
  <div class="dlg-h"><span data-ic="shield"></span><span class="dlg-t">About VIKAS Studio</span><span class="dlg-x" data-ic="close"></span></div>
  <div class="dlg-b" id="aboutbody"></div>
</dialog>
<dialog id="dlg-export">
  <div class="dlg-h"><span data-ic="download"></span><span class="dlg-t">Export</span><span class="dlg-x" data-ic="close"></span></div>
  <div class="dlg-b" id="exportbody"></div>
</dialog>
<script>
'use strict';
/* ================= data (placeholder-injected) ================= */
const SIGS = __SIGS__;
const CHG  = "__CHG__";
let END  = __END__ || 1;
let NEV  = __NEV__ || 0;
const CTX  = __CTX__;

/* ================= 40-icon professional SVG set ================= */
const ICONS = {
logo:'<path d="M12 2 21 7 21 15 12 22 3 15 3 7Z"/><path d="M7.5 8.5 12 15.5 16.5 8.5"/>',
open:'<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M3 11h18"/>',
save:'<path d="M5 3h11l3 3v13a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z"/><path d="M8 3v5h6V3"/><rect x="7" y="13" width="10" height="6" rx="1"/>',
fit:'<path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M16 3h3a2 2 0 0 1 2 2v3"/><path d="M8 21H5a2 2 0 0 1-2-2v-3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/>',
zin:'<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/><path d="M8 11h6M11 8v6"/>',
zout:'<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/><path d="M8 11h6"/>',
region:'<path d="M5 5h2M12 5h2M19 5h-2M5 12v2M5 19v-2M12 19h2M19 19h-2M19 12v2" stroke-dasharray="2 3"/><rect x="8" y="8" width="8" height="8" rx="1"/>',
cursor:'<circle cx="12" cy="12" r="7"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4"/>',
cursorClear:'<circle cx="12" cy="12" r="7"/><path d="M12 2v4M12 18v4M2 12h4M18 12h4"/><path d="M4 4l16 16"/>',
measure:'<path d="M3 17 17 3l4 4L7 21z"/><path d="M8 12l1.5 1.5M12 8l1.5 1.5"/>',
prev:'<path d="M15 6l-6 6 6 6"/><path d="M18 6v12"/>',
next:'<path d="M9 6l6 6-6 6"/><path d="M6 6v12"/>',
search:'<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/>',
group:'<path d="M8 4H6a2 2 0 0 0-2 2v2M16 4h2a2 2 0 0 1 2 2v2M8 20H6a2 2 0 0 1-2-2v-2M16 20h2a2 2 0 0 0 2-2v-2"/><rect x="8" y="8" width="8" height="8" rx="1.5"/>',
ungroup:'<path d="M8 4H6a2 2 0 0 0-2 2v2M16 4h2a2 2 0 0 1 2 2v2M8 20H6a2 2 0 0 1-2-2v-2M16 20h2a2 2 0 0 0 2-2v-2"/><rect x="8" y="8" width="8" height="8" rx="1.5"/><path d="M3 21 21 3"/>',
expand:'<path d="M9 9h6v6H9z"/><path d="M4 9h3M4 15h3M17 9h3M17 15h3"/><path d="M10 2l-2 2 2 2M14 2l2 2-2 2M10 22l-2-2 2-2M14 22l2-2-2-2"/>',
collapse:'<path d="M9 9h6v6H9z"/><path d="M4 9h3M4 15h3M17 9h3M17 15h3"/>',
palette:'<circle cx="12" cy="12" r="9"/><circle cx="8.5" cy="9" r="1.2"/><circle cx="15" cy="9" r="1.2"/><circle cx="9" cy="15" r="1.2"/><circle cx="15" cy="15" r="1.2"/>',
analog:'<path d="M2 12c1.8-5 3.4-5 5.2 0s3.4 5 5.2 0 3.4-5 5.2 0 3.4 5 5.2 0"/>',
download:'<path d="M12 3v11"/><path d="M7 10l5 5 5-5"/><path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/>',
moon:'<path d="M20 13A8 8 0 1 1 11 4a6.5 6.5 0 0 0 9 9z"/>',
sun:'<circle cx="12" cy="12" r="4.5"/><path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M4.6 4.6l1.8 1.8M17.6 17.6l1.8 1.8M19.4 4.6l-1.8 1.8M6.4 17.6l-1.8 1.8"/>',
help:'<circle cx="12" cy="12" r="9"/><path d="M9.5 9.2a2.6 2.6 0 0 1 5 1c0 1.8-2.5 2.2-2.5 3.8"/><circle cx="12" cy="17.3" r="0.4" fill="currentColor"/>',
close:'<path d="M6 6l12 12M18 6L6 18"/>',
chevD:'<path d="M6 9l6 6 6-6"/>',
chevR:'<path d="M9 6l6 6-6 6"/>',
plus:'<path d="M12 5v14M5 12h14"/>',
minus:'<path d="M5 12h14"/>',
trash:'<path d="M4 7h16"/><path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/><path d="M6 7l1 13a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-13"/><path d="M10 11v6M14 11v6"/>',
up:'<path d="M12 20V5"/><path d="M6 11l6-6 6 6"/>',
down:'<path d="M12 4v15"/><path d="M6 13l6 6 6-6"/>',
panelL:'<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>',
panelR:'<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M15 4v16"/>',
refresh:'<path d="M20 11A8 8 0 0 0 5.6 6.4L4 8"/><path d="M4 4v4h4"/><path d="M4 13a8 8 0 0 0 14.4 4.6L20 16"/><path d="M20 20v-4h-4"/>',
play:'<path d="M7 4.5l12 7.5-12 7.5z"/>',
flag:'<path d="M5 21V4"/><path d="M5 4c4-2 8 2 13 0v9c-5 2-9-2-13 0"/>',
check:'<path d="M4.5 12.5l5 5L19.5 7"/>',
warn:'<path d="M12 3 2.5 20h19z"/><path d="M12 9.5V14"/><circle cx="12" cy="16.8" r="0.4" fill="currentColor"/>',
info:'<circle cx="12" cy="12" r="9"/><path d="M12 11v5.5"/><circle cx="12" cy="7.7" r="0.4" fill="currentColor"/>',
cpu:'<rect x="6" y="6" width="12" height="12" rx="2"/><rect x="9.5" y="9.5" width="5" height="5" rx="1"/><path d="M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3"/>',
zap:'<path d="M13 2 4.5 13.5H11L10 22l8.5-11.5H12z"/>',
activity:'<path d="M2 12h4l2.5-7 5 14L16 12h6"/>',
shield:'<path d="M12 2 4.5 5v7c0 4.6 3.2 8 7.5 10 4.3-2 7.5-5.4 7.5-10V5z"/><path d="M9 12l2.2 2.2L15.5 9.7"/>',
list:'<path d="M8 6h13M8 12h13M8 18h13"/><circle cx="4" cy="6" r="1" fill="currentColor"/><circle cx="4" cy="12" r="1" fill="currentColor"/><circle cx="4" cy="18" r="1" fill="currentColor"/>',
grid:'<rect x="3.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="3.5" width="7" height="7" rx="1.5"/><rect x="3.5" y="13.5" width="7" height="7" rx="1.5"/><rect x="13.5" y="13.5" width="7" height="7" rx="1.5"/>',
file:'<path d="M6 2.5h8l4 4V21a.5.5 0 0 1-.5.5h-11A.5.5 0 0 1 6 21z"/><path d="M14 2.5v4h4"/>',
layers:'<path d="M12 2 2 7l10 5 10-5z"/><path d="M2 12l10 5 10-5"/><path d="M2 17l10 5 10-5"/>',
terminal:'<path d="M4 4h16a1 1 0 0 1 1 1v14a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1z"/><path d="M7 9l3.5 3L7 15"/><path d="M12.5 15h4"/>',
target:'<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1.2" fill="currentColor"/>',
clock:'<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3.5 2"/>',
database:'<ellipse cx="12" cy="5.5" rx="8" ry="3"/><path d="M4 5.5v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/><path d="M4 11.5v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/>',
radar:'<path d="M12 3 21 20H3z"/><path d="M12 3v17"/>',
flow:'<circle cx="5" cy="6" r="2.5"/><circle cx="19" cy="18" r="2.5"/><path d="M7.5 6h6A4.5 4.5 0 0 1 18 10.5v5"/><path d="M16.5 15.5 18 17l1.5-1.5"/>',
};
function svgIcon(n){ const body=ICONS[n]||ICONS.info;
  return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">'+body+'</svg>'; }

/* ================= tiny DOM + fmt helpers ================= */
const $=s=>document.querySelector(s), $$=s=>[...document.querySelectorAll(s)];
const esc=s=>String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
function fmtT(t){ t=Math.round(t*1000)/1000; return (Math.abs(t-Math.round(t))<0.001? Math.round(t): t)+''; }
function fmtInt(n){ return n>=1e6? (n/1e6).toFixed(2)+'M' : n>=1e3? (n/1e3).toFixed(1)+'k' : n+''; }
function pad(s,w){ return (s.length>=w? s : '0'.repeat(w-s.length)+s); }
const KIND_COLOR={rtl:'#89e05a',clock:'#22d3ee',physics:'#fbbf24',causality:'#fb7185',stochastic:'#a78bfa',taint:'#2dd4bf',latch:'#f87171',reset:'#fb923c'};
function kindColor(k){ return KIND_COLOR[k]||'#89e05a'; }

/* ================= blob decode: base64 -> per-signal steps ================= */
function b64u8(b64){ const bin=atob(b64); const u=new Uint8Array(bin.length);
  for(let i=0;i<bin.length;i++) u[i]=bin.charCodeAt(i); return u; }
const ST_CH=['0','1','z','x'];
/* steps[si] = {t:Float64Array, v:Array, n, glitch:Set, glitchN} */
let STEPS=[];
function decodeBlob(){
  const u8=b64u8(CHG); let p=0;
  function vint(){ let r=0,s=0,b; do{ b=u8[p++]; r+=(b&0x7f)*Math.pow(2,s); s+=7; }while(b&0x80); return r; }
  const nch=vint();
  const tA=SIGS.map(()=>[]), vA=SIGS.map(()=>[]);
  const glA=SIGS.map(()=>new Set());
  let lastT=0;
  for(let i=0;i<nch;i++){
    const dt=vint(), si=vint(), w=vint();
    const nb=(w+3)>>2; const base=p; p+=nb;
    const t=lastT+dt; lastT=t;
    if(dt===0 && tA[si].length && tA[si][tA[si].length-1]===t) glA[si].add(tA[si].length-1);
    let v;
    if(w===1){ v=ST_CH[u8[base]&3]; }
    else{
      let allDef=true, num=0, str='';
      for(let b=0;b<w;b++){
        const d=(u8[base+(b>>2)]>>((b&3)*2))&3;
        if(d>1) allDef=false;
        str+=ST_CH[d];
        if(allDef && b<32) num=num*2+d;
      }
      if(allDef && w<=32) v=num;
      else if(allDef){ let n2=0; for(let b=0;b<w;b++) n2=n2*2+Number(str[b]); v= (n2<=Number.MAX_SAFE_INTEGER)? n2 : str; }
      else v=str;  // MSB-first 01xz string
    }
    tA[si].push(t); vA[si].push(v);
  }
  STEPS=SIGS.map((s,i)=>{
    const t=new Float64Array(tA[i]);
    let last=-1, uniq=0;
    for(let k=0;k<vA[i].length;k++){ if(vA[i][k]!==last){ last=vA[i][k]; uniq++; } }
    return {t, v:vA[i], n:vA[i].length, glitch:[...glA[i]], uniq, minT:t.length?t[0]:0, maxT:t.length?t[t.length-1]:0};
  });
}
/* value at time (binary search) — returns char | number | bitstring */
function valAt(si,t){
  const s=STEPS[si]; if(!s||!s.n) return SIGS[si].w===1?'x':'';
  let lo=0,hi=s.n-1,r=s.v[0];
  while(lo<=hi){ const m=(lo+hi)>>1; if(s.t[m]<=t){ r=s.v[m]; lo=m+1; } else hi=m-1; }
  return r;
}
/* next change strictly after t */
function nextChange(si,t){ const s=STEPS[si]; if(!s) return null;
  let lo=0,hi=s.n-1,r=null; while(lo<=hi){ const m=(lo+hi)>>1; if(s.t[m]>t){ r=s.t[m]; hi=m-1; } else lo=m+1; } return r; }
function prevChange(si,t){ const s=STEPS[si]; if(!s) return null;
  let lo=0,hi=s.n-1,r=null; while(lo<=hi){ const m=(lo+hi)>>1; if(s.t[m]<t){ r=s.t[m]; lo=m+1; } else hi=m-1; } return r; }

/* ================= radix formatting ================= */
function bitsToStr(v,w){ // any value -> '01xz' string (MSB-first)
  if(typeof v==='string' && /[^0-9]/.test(v)===true && /[xz]/.test(v)) return pad(v,w); // already 01xz string
  if(typeof v==='string') return v; // 01 string for wide clean
  if(typeof v==='number'){ let s=(v>>>0).toString(2); return pad(s,w); }
  return 'x'.repeat(w);
}
function strToHex(str,w){ let out=''; const padw=Math.ceil(w/4);
  for(let i=0;i<padw;i++){
    let nib='', allDef=true, anyX=false;
    for(let b=0;b<4;b++){ const idx=(padw-1-i)*4+(3-b); const c=(idx>=str.length)?'0':str[idx];
      if(c==='x'||c==='z'){ allDef=false; anyX=true; } nib+= (c==='1'?'1':'0'); }
    if(!allDef){ out+=anyX?'x':'z'; continue; }
    out+=parseInt(nib,2).toString(16);
  }
  return out.replace(/^0+(?=.)/,'');
}
function fmtRadix(v,w,radix){
  if(w===1) return v===undefined?'x':String(v);
  if(v===undefined||v==='') return 'x'.repeat(Math.min(w,8))+(w>8?'…':'');
  switch(radix){
    case 'bin':{ const s=bitsToStr(v,w); return s.length>24? s.slice(0,24)+'…': s; }
    case 'dec':{ if(typeof v==='number') return v.toString(10);
      const s=bitsToStr(v,w); if(/[xz]/.test(s)) return s;
      let n=0n; for(const c of s) n=n*2n+(c==='1'?1n:0n); return n.toString(10); }
    case 'signed':{ if(typeof v==='number'){ const half=Math.pow(2,w-1); return (v>=half? v-Math.pow(2,w): v)+''; }
      const s=bitsToStr(v,w); if(/[xz]/.test(s)) return s;
      let n=0n; for(const c of s) n=n*2n+(c==='1'?1n:0n); const half=2n**BigInt(w-1);
      return (n>=half? n-(2n**BigInt(w)) : n)+''; }
    case 'oct':{ const s=bitsToStr(v,w); return /[xz]/.test(s)? s : '0o'+BigInt('0b'+s).toString(8); }
    case 'ascii':{ const s=bitsToStr(v,w); if(/[xz]/.test(s)) return 'x'+(w>>3);
      let out=''; for(let i=0;i<s.length;i+=8){ const byte=s.slice(i,i+8); if(byte.length<8) break;
        const ch=parseInt(byte,2); out+= (ch>=32&&ch<127)? String.fromCharCode(ch): '.'; } return out||'·'; }
    case 'hex': default:{ if(typeof v==='number') return '0x'+v.toString(16);
      return '0x'+strToHex(bitsToStr(v,w),w); }
  }
}

/* ================= app state ================= */
const S={
  rows:[],            // [{type:'sig',si,radix,color,expanded,analog} | {type:'group',name,open,children:[]}]
  sel:new Set(),      // selected row objects (refs)
  t0:0, tspan:Math.max(END,1),
  curA:null, curB:null,
  hoverT:null, hoverRow:-1,
  theme:'dark', defRadix:'hex',
  nameW:230, valW:96,
  scrollY:0, rowH:26, groupH:22, bitH:15,
  regionMode:false, marquee:null,
  results:[], resIdx:0, searchMode:'signal',
  measureMode:false,
  fileName:'VIKAS session',
  dirty:false,
};
function setTheme(t){ S.theme=t; document.documentElement.setAttribute('data-theme',t);
  try{ localStorage.setItem('vikas-studio-theme',t); }catch(e){}
  draw(); drawMM(); }
function cycleTheme(){ const order=['dark','verdi','light','synth'];
  setTheme(order[(order.indexOf(S.theme)+1)%order.length]);
  toast('Theme: '+({dark:'VIKAS Dark',verdi:'Verdi Classic',light:'Studio Light',synth:'Synth Neon'})[S.theme],'palette'); }

/* ================= rows ================= */
function addRow(si, at){ const row={type:'sig',si,radix:null,color:null,expanded:false,analog:false};
  if(at===undefined||at>=S.rows.length) S.rows.push(row); else S.rows.splice(at,0,row);
  S.sel.clear(); S.sel.add(row); syncTree(); layout(); draw(); return row; }
function addAllRows(){ S.rows=SIGS.map((_,i)=>({type:'sig',si:i,radix:null,color:null,expanded:false,analog:false})); syncTree(); layout(); draw(); }
function clearRows(){ S.rows=[]; S.sel.clear(); syncTree(); layout(); draw(); }
function removeSel(){ const flat=[]; (function walk(rs){ for(const r of rs){ if(S.sel.has(r)) continue;
  if(r.type==='group'){ flat.push(r); walk(r.children);} else flat.push(r);} })(S.rows);
  S.rows=flat; S.sel.clear(); syncTree(); layout(); draw(); }
function findRow(si){ let out=null; (function walk(rs){ for(const r of rs){ if(r.type==='sig'&&r.si===si) out=r;
  if(r.type==='group') walk(r.children); } })(S.rows); return out; }
function flatRows(){ const out=[]; (function walk(rs,depth){ for(const r of rs){ out.push({r,depth});
  if(r.type==='group'&&r.open) walk(r.children,depth+1);} })(S.rows,0); return out; }

/* group / ungroup on the top level selection window */
function groupSel(){ const idx=[]; S.rows.forEach((r,i)=>{ if(S.sel.has(r)) idx.push(i); });
  if(idx.length<2){ toast('Select 2+ adjacent rows to group','warn'); return; }
  const first=idx[0], last=idx[idx.length-1];
  const kids=S.rows.splice(first,last-first+1);
  const g={type:'group',name:kids[0].type==='sig'? shortName(kids[0].si):'group',open:true,children:kids};
  S.rows.splice(first,0,g); S.sel.clear(); S.sel.add(g); layout(); draw(); syncTree();
  toast('Grouped '+kids.length+' signals','group'); }
function ungroupSel(){ let did=false;
  const expand=(rs)=>rs.flatMap(r=> (r.type==='group'&&S.sel.has(r)) ? (did=true,r.children) : r.type==='group'? [Object.assign({},r,{children:expand(r.children)})] : [r]);
  S.rows=expand(S.rows); if(did){ S.sel.clear(); layout(); draw(); toast('Ungrouped','ungroup'); } else toast('No selected group','warn'); }
function shortName(si){ const n=SIGS[si].n; return n.split('.').pop()||n; }

/* ================= menus ================= */
const MENUS=[
 ['File',[
   ['open','Open Dump / Layout…','Ctrl+O','openDump'],
   ['save','Save View Layout','Ctrl+S','saveLayout'],
   'sep',
   ['download','Export PNG (view)','Ctrl+P','()=>exportPNG(false)'],
   ['download','Export PNG (all rows)','Ctrl+Shift+P','()=>exportPNG(true)'],
   ['file','Export SVG (view)','','exportSVG'],
   ['file','Export CSV transitions','','exportCSV'],
   ['file','Export JSON steps','','exportJSON'],
   ['file','Export WaveDrom JSON (window)','','exportWavedrom'],
   'sep',
   ['refresh','Reload from source','','reloadSelf'],
 ]],
 ['Edit',[
   ['check','Add All Signals','','addAllRows'],
   ['trash','Clear View','','clearRows'],
   'sep',
   ['up','Move Selection Up','Alt+↑','moveSel(-1)'],
   ['down','Move Selection Down','Alt+↓','moveSel(1)'],
   ['close','Remove Selected','Del','removeSel'],
 ]],
 ['View',[
   ['fit','Fit Timeline','F','fit'],
   ['zin','Zoom In','+','zoom(0.6)'],
   ['zout','Zoom Out','-','zoom(1.6)'],
   ['region','Marquee Zoom Mode','Z','toggleRegion'],
   'sep',
   ['panelL','Toggle Signal Tree','Ctrl+1','toggleDock("#treedock")'],
   ['panelR','Toggle Properties','Ctrl+2','toggleDock("#propdock")'],
   ['panelR','Toggle Results Panel','Ctrl+3','toggleDock("#resultsdock",true)'],
   ['panelL','Toggle Console','`','toggleConsole'],
   'sep',
   ['moon','Theme: VIKAS Dark','','setTheme("dark")'],
   ['moon','Theme: Verdi Classic','','setTheme("verdi")'],
   ['sun','Theme: Studio Light','','setTheme("light")'],
   ['moon','Theme: Synth Neon','','setTheme("synth")'],
 ]],
 ['Signals',[
   ['group','Group Selected','G','groupSel'],
   ['ungroup','Ungroup Selected','U','ungroupSel'],
   ['expand','Expand Bus to Bits','E','expandSel(true)'],
   ['collapse','Collapse Bus','Shift+E','expandSel(false)'],
   ['analog','Toggle Analog Trace','A','toggleAnalog'],
   'sep',
   ['radar','Radix: Hex','','setRadixAll("hex")'],
   ['radar','Radix: Binary','','setRadixAll("bin")'],
   ['radar','Radix: Decimal','','setRadixAll("dec")'],
   ['radar','Radix: Signed','','setRadixAll("signed")'],
   ['radar','Radix: ASCII','','setRadixAll("ascii")'],
 ]],
 ['Analysis',[
   ['search','Find Signal / Boolean Expr…','Ctrl+F','focusSearch'],
   ['play','Run Search','Enter','runSearch'],
   ['prev','Previous Result','[','jumpResult(-1)'],
   ['next','Next Result',']','jumpResult(1)'],
   'sep',
   ['cursor','Cursor A at View Center','c','()=>setCursorA(S.t0+S.tspan/2)'],
   ['measure','Measure Mode (click = cursor B)','m','toggleMeasure'],
   ['cursorClear','Clear Cursors','C','clearCursors'],
   'sep',
   ['grid','Open Analysis Hub','','showTab("hub")'],
   ['database','Open Design Intel','','showTab("intel")'],
 ]],
 ['Help',[
   ['help','Keyboard & Mouse Reference','F1','helpDialog'],
   ['zap','Command Palette','Ctrl+K','openPalette'],
   ['info','About VIKAS Studio','','aboutDialog'],
 ]],
];
function buildMenubar(){
  const mb=$('#menubar'); mb.innerHTML='';
  for(const [name,items] of MENUS){
    const mi=document.createElement('div'); mi.className='menu-item';
    mi.innerHTML='<div class="menu-btn">'+name+'</div><div class="menu-drop"></div>';
    const drop=mi.querySelector('.menu-drop');
    for(const it of items){
      if(it==='sep'){ const s=document.createElement('div'); s.className='msep'; drop.appendChild(s); continue; }
      const [ic,label,sc,code]=it;
      const d=document.createElement('div'); d.className='mi';
      d.innerHTML=svgIcon(ic)+'<span>'+label+'</span>'+(sc?'<span class="sc">'+sc+'</span>':'');
      d.onclick=()=>{ closeMenus(); act(code); };
      drop.appendChild(d);
    }
    mi.querySelector('.menu-btn').onclick=(e)=>{ e.stopPropagation();
      const was=mi.classList.contains('menu-open'); closeMenus(); if(!was) mi.classList.add('menu-open'); };
    mi.querySelector('.menu-btn').onmouseenter=()=>{ if($('.menu-open')){ closeMenus(); mi.classList.add('menu-open'); } };
    mb.appendChild(mi);
  }
  document.addEventListener('click',closeMenus);
}
function closeMenus(){ $$('.menu-item').forEach(m=>m.classList.remove('menu-open')); }

/* act(): executes a code string or function */
function act(code){
  if(typeof code==='function'){ code(); return; }
  try{ (new Function('return ('+code+')'))(); }catch(e){ console.error(e); toast('action failed: '+esc(code),'bad'); }
}
function toggleDock(sel,force){ const d=$(sel); const show=force!==undefined? force : d.style.display==='none';
  d.style.display= show? (sel==='#resultsdock'? 'flex':'flex'):'none';
  if(!show&&sel==='#resultsdock') d.classList.remove('open');
  if(show&&sel==='#resultsdock') d.classList.add('open');
  layout(); draw(); }
function setRadixAll(rx){ S.defRadix=rx; $('#radixsel').value=rx; for(const {r} of flatRows()) if(r.type==='sig') r.radix=null;
  draw(); toast('Default radix: '+rx,'radar'); }
function expandSel(on){ let n=0; for(const {r} of flatRows()) if(r.type==='sig'&&S.sel.has(r)&&SIGS[r.si].w>1){ r.expanded=on; n++; }
  if(!n){ toast(on?'Select a bus (width>1) to expand':'Nothing to collapse','warn'); return; }
  layout(); draw(); toast((on?'Expanded ':'Collapsed ')+n+' bus(es)','expand'); }
function toggleAnalog(){ let n=0; for(const {r} of flatRows()) if(r.type==='sig'&&S.sel.has(r)&&SIGS[r.si].w>=8){ r.analog=!r.analog; n++; }
  if(!n){ toast('Select a bus (width>=8) for analog trace','warn'); return; } layout(); draw(); }
function toggleRegion(){ S.regionMode=!S.regionMode; $('[data-act=region]').classList.toggle('on',S.regionMode);
  $('#canvaswrap').style.cursor=S.regionMode?'crosshair':'default';
  toast(S.regionMode?'Marquee zoom: drag on the wave area':'Pan mode: drag to pan','region'); }
function toggleMeasure(){ S.measureMode=!S.measureMode; $('[data-act=cursor-b]').classList.toggle('on',S.measureMode);
  toast(S.measureMode?'Measure mode: click on wave = cursor B':'Measure mode off','measure'); }
function toggleConsole(){ const c=$('#consoledock'); c.classList.toggle('open'); layout(); }
function moveSel(d){ if(!S.sel.size) return;
  const idxs=[]; S.rows.forEach((r,i)=>{ if(S.sel.has(r)) idxs.push(i); });
  if(d<0){ const i=idxs[0]; if(i===0) return; const [r]=S.rows.splice(i,1); S.rows.splice(i-1,0,r); }
  else{ const i=idxs[idxs.length-1]; if(i>=S.rows.length-1) return; const [r]=S.rows.splice(i,1); S.rows.splice(i+1,0,r); }
  layout(); draw(); }
function clearCursors(){ S.curA=S.curB=null; draw(); sbUpdate(); }
function setCursorA(t){ S.curA=Math.max(0,Math.min(END,t)); draw(); sbUpdate(); updProps(); }
function toggleColorMenu(){ ctxMenu(colorMenuItems(), lastMouse.x, lastMouse.y); }

/* ================= canvas engine ================= */
const cv=$('#wv'), mm=$('#mm'), wrap=$('#canvaswrap');
let ctx=cv.getContext('2d'), mctx=mm.getContext('2d');
let W=800, H=600, dpr=1;
let LAYOUT=[];   // visible row layout entries for hit-test
let COLORS={};
function refreshColors(){ const cs=getComputedStyle(document.documentElement);
  const g=n=>cs.getPropertyValue(n).trim();
  COLORS={bg:g('--bg'),panel:g('--panel'),panel2:g('--panel2'),raised:g('--raised'),
    edge:g('--edge'),edge2:g('--edge2'),txt:g('--txt'),dim:g('--dim'),faint:g('--faint'),
    accent:g('--accent'),accent2:g('--accent2'),ok:g('--ok'),warn:g('--warn'),bad:g('--bad'),
    sel:g('--sel'),cursorA:g('--cursorA'),cursorB:g('--cursorB'),
    wave:g('--wave'),bus:g('--bus'),xcol:g('--xcol'),zcol:g('--zcol')}; }
let hatchPat=null;
function makeHatch(){ const pc=document.createElement('canvas'); pc.width=6; pc.height=6;
  const p=pc.getContext('2d'); p.strokeStyle=COLORS.xcol; p.lineWidth=1.0; p.globalAlpha=.66;
  p.beginPath(); p.moveTo(-1.5,4.5); p.lineTo(4.5,-1.5); p.moveTo(1.5,7.5); p.lineTo(7.5,1.5); p.stroke();
  hatchPat=ctx.createPattern(pc,'repeat'); }
let zPat=null;
function makeZPat(){ const pc=document.createElement('canvas'); pc.width=6; pc.height=6;
  const p=pc.getContext('2d'); p.strokeStyle=COLORS.zcol; p.lineWidth=1;
  p.setLineDash([2,2]); p.beginPath(); p.moveTo(0,3); p.lineTo(6,3); p.stroke();
  zPat=ctx.createPattern(pc,'repeat'); }
function layout(){
  dpr=window.devicePixelRatio||1;
  W=wrap.clientWidth; H=wrap.clientHeight;
  cv.width=W*dpr; cv.height=H*dpr; cv.style.width=W+'px'; cv.style.height=H+'px';
  ctx.setTransform(dpr,0,0,dpr,0,0);
  refreshColors(); makeHatch(); makeZPat();
  const mw=mm.clientWidth, mh=mm.clientHeight;
  mm.width=mw*dpr; mm.height=mh*dpr; mctx.setTransform(dpr,0,0,dpr,0,0);
}
const AX=30, SB=10; // axis height, scrollbar width
function waveX0(){ return S.nameW+S.valW; }
function waveX1(){ return W-SB; }
function timeToX(t){ return waveX0()+(t-S.t0)/S.tspan*(waveX1()-waveX0()); }
function xToTime(x){ return S.t0+(x-waveX0())/(waveX1()-waveX0())*S.tspan; }
function rowH(r){ if(r.type==='group') return S.groupH;
  if(r.expanded) return S.rowH + Math.max(1,SIGS[r.si].w)*S.bitH;
  return S.rowH; }
function contentH(){ let h=0; for(const {r} of flatRows()) h+=rowH(r); return h; }
function buildLayout(){
  LAYOUT=[]; let y=AX-S.scrollY;
  for(const {r,depth} of flatRows()){
    const h=rowH(r);
    LAYOUT.push({r,y,h,depth,kind:r.type});
    if(r.type==='sig'&&r.expanded){
      for(let b=SIGS[r.si].w-1;b>=0;b--) LAYOUT.push({r,y:y+S.rowH+(SIGS[r.si].w-1-b)*S.bitH,h:S.bitH,depth:depth+1,kind:'bit',bit:b});
    }
    y+=h;
  }
}
function niceStep(span,n){ const raw=span/n; const p=Math.pow(10,Math.floor(Math.log10(raw)));
  for(const m of [1,2,5,10]){ if(m*p>=raw) return m*p; } return 10*p; }

function draw(){
  buildLayout();
  ctx.clearRect(0,0,W,H);
  ctx.fillStyle=COLORS.bg; ctx.fillRect(0,0,W,H);
  /* left panel bg */
  ctx.fillStyle=COLORS.panel; ctx.fillRect(0,0,S.nameW+S.valW,H);
  /* row bands */
  let alt=0;
  for(const e of LAYOUT){
    if(e.y>H||e.y+e.h<0) continue;
    if(e.kind==='group'||e.kind==='bit') continue;
    alt^=1;
    ctx.fillStyle=alt? COLORS.bg2||COLORS.panel2 : COLORS.bg;
    ctx.fillRect(0,e.y,W,e.h);
    if(S.sel.has(e.r)){ ctx.fillStyle=COLORS.sel; ctx.fillRect(0,e.y,W,e.h);
      ctx.fillStyle=COLORS.accent; ctx.fillRect(0,e.y,2,e.h); }
  }
  /* grid + axis */
  const step=niceStep(S.tspan,10);
  ctx.font='10px '+mono();
  const tStart=Math.max(0,Math.floor(S.t0/step)*step);
  for(let t=tStart;t<=S.t0+S.tspan;t+=step){
    const x=timeToX(t);
    ctx.strokeStyle=COLORS.edge; ctx.lineWidth=1;
    ctx.beginPath(); ctx.moveTo(x+.5,AX); ctx.lineTo(x+.5,H); ctx.stroke();
    ctx.fillStyle=COLORS.dim; ctx.fillText(fmtT(t),x+3,AX-8);
  }
  /* axis strip */
  ctx.fillStyle=COLORS.panel; ctx.fillRect(0,0,S.nameW+S.valW,AX);
  ctx.strokeStyle=COLORS.edge2; ctx.beginPath(); ctx.moveTo(0,AX-.5); ctx.lineTo(W,AX-.5); ctx.stroke();
  ctx.fillStyle=COLORS.faint; ctx.font='9.5px '+mono();
  ctx.fillText('VALUE @ A',S.nameW+8,12);
  ctx.fillText('TIME (ns)',waveX0()+4,12);
  /* rows */
  for(const e of LAYOUT){ if(e.y>H||e.y+e.h<0) continue;
    if(e.kind==='group') drawGroup(e); else if(e.kind==='bit') drawBitLane(e); else drawRow(e); }
  /* splitters */
  ctx.strokeStyle=COLORS.edge2; ctx.beginPath(); ctx.moveTo(S.nameW+.5,0); ctx.lineTo(S.nameW+.5,H); ctx.stroke();
  ctx.beginPath(); ctx.moveTo(S.nameW+S.valW+.5,0); ctx.lineTo(S.nameW+S.valW+.5,H); ctx.stroke();
  drawCursors(); drawHover(); drawMarquee(); drawScrollbar(); updGauge(); sbUpdate();
}
function mono(){ return getComputedStyle(document.documentElement).getPropertyValue('--mono'); }
function nameStyle(){ return getComputedStyle(document.documentElement).getPropertyValue('--font'); }

function drawGroup(e){
  const g=e.r;
  ctx.fillStyle=COLORS.raised; ctx.fillRect(0,e.y,W,e.h);
  ctx.fillStyle=COLORS.accent; ctx.fillRect(0,e.y,W,1.5);
  const x=6+e.depth*12;
  ctx.fillStyle=COLORS.accent; ctx.font='10px '+mono();
  ctx.fillText((g.open?'▾':'▸')+' ⊞ '+g.name, x, e.y+e.h-7);
  ctx.fillStyle=COLORS.faint; ctx.font='9.5px '+mono();
  const n=countSigs(g);
  ctx.fillText(n+' sig', W-30-ctx.measureText(n+' sig').width-SB, e.y+e.h-7);
}
function countSigs(g){ let n=0; (function w(k){ for(const c of k.children){ c.type==='group'? w(c): n++; } })(g); return n; }

function drawRow(e){
  const r=e.r, sig=SIGS[r.si];
  const y=e.y, h=e.h;
  /* name cell */
  const col=r.color||kindColor(sig.k);
  ctx.fillStyle=col; ctx.beginPath(); ctx.roundRect(6,y+h/2-3.5,7,7,2); ctx.fill();
  ctx.fillStyle=COLORS.txt; ctx.font='11px '+mono();
  const label=truncate(sig.n, S.nameW-34);
  ctx.fillText(label, 20, y+h/2+4);
  ctx.fillStyle=COLORS.faint; ctx.font='9px '+mono();
  ctx.fillText(sig.w+'b', S.nameW-22, y+h/2+4);
  if(r.expanded){ ctx.fillStyle=COLORS.accent; ctx.font='9px '+mono(); ctx.fillText('⊕',S.nameW-34,y+h/2+4); }
  /* value cell at cursor A (or view start) */
  const vt=S.curA!==null? S.curA : S.t0;
  const v=valAt(r.si,vt);
  const rad=r.radix||S.defRadix;
  const vs=fmtRadix(v,sig.w,rad);
  ctx.fillStyle=(typeof v==='string'&&/[xz]/.test(v)&&sig.w>1)? COLORS.xcol : COLORS.txt;
  ctx.font='10px '+mono();
  ctx.fillText(truncate(vs,S.valW-10), S.nameW+8, y+h/2+4);
  /* wave */
  const wx0=Math.max(timeToX(Math.max(0,S.t0)),waveX0()), wx1=Math.min(timeToX(S.t0+S.tspan),waveX1());
  if(wx1<=wx0) return;
  if(sig.w===1) drawBit(r.si,wx0,wx1,y,h,col);
  else if(r.analog) drawAnalog(r.si,wx0,wx1,y,h,col);
  else drawBus(r.si,wx0,wx1,y,h,col,r.radix||S.defRadix);
  /* transition count + glitches */
  const s=STEPS[r.si];
  if(s&&s.n){
    ctx.fillStyle=COLORS.faint; ctx.font='9px '+mono();
    const cn=fmtInt(s.n)+(s.glitch.length? ' ⚡'+s.glitch.length:'');
    ctx.fillText(cn, W-SB-6-ctx.measureText(cn).width, y+h/2+4);
  }
}
function drawBit(si,wx0,wx1,y,h,col){
  const s=STEPS[si]; if(!s||!s.n){ /* flat X */
    ctx.fillStyle=hatchPat; ctx.fillRect(wx0,y+4,wx1-wx0,h-8); return; }
  const y1=y+6, y0=y+h-6, ym=(y1+y0)/2;
  /* baseline guide */
  ctx.strokeStyle=COLORS.edge; ctx.lineWidth=1;
  ctx.beginPath(); ctx.moveTo(wx0,y0+.5); ctx.lineTo(wx1,y0+.5); ctx.stroke();
  ctx.strokeStyle=col; ctx.lineWidth=1.5; ctx.beginPath();
  let px=wx0, py=y1, cur=valAt(si,S.t0);
  const T=STEPS[si].t;
  let lo=0,hi=T.length-1,start=0;
  while(lo<=hi){ const m=(lo+hi)>>1; if(T[m]<S.t0){ start=m+1; lo=m+1; } else hi=m-1; }
  if(start>0) start--;
  for(let i=start;i<T.length&&T[i]<=S.t0+S.tspan+1;i++){
    const t=T[i], v=s.v[i], x=timeToX(t);
    const ny=v==='1'? y1 : v==='0'? y0 : ym;
    if(px<x){ ctx.lineTo(x,py); ctx.lineTo(x,ny); }
    px=x; py=ny; cur=v;
  }
  ctx.lineTo(wx1,py); ctx.stroke(); ctx.lineWidth=1;
  /* X / Z fills on matching segments */
  for(let i=start;i<T.length&&T[i]<=S.t0+S.tspan+1;i++){
    const v=s.v[i];
    if(v==='x'||v==='z'){ const t=T[i]; const te=(i+1<T.length)? T[i+1] : S.t0+S.tspan;
      const xa=Math.max(wx0,timeToX(t)), xe=Math.min(wx1,timeToX(te));
      if(xe>xa){ ctx.fillStyle=v==='x'? hatchPat: zPat; ctx.fillRect(xa,y+4,xe-xa,h-8);
        ctx.strokeStyle=COLORS.faint; ctx.strokeRect(xa+.5,y+4.5,xe-xa-1,h-9); } }
  }
}
function drawBus(si,wx0,wx1,y,h,col,radix){
  const s=STEPS[si]; const yt=y+5, yb=y+h-5, ym=(yt+yb)/2;
  if(!s||!s.n){ ctx.fillStyle=hatchPat; ctx.fillRect(wx0,yt,wx1-wx0,yb-yt);
    ctx.strokeStyle=COLORS.faint; ctx.strokeRect(wx0+.5,yt+.5,wx1-wx0-1,yb-yt-1); return; }
  const T=s.t; let lo=0,hi=T.length-1,start=0;
  while(lo<=hi){ const m=(lo+hi)>>1; if(T[m]<S.t0){ start=m+1; lo=m+1; } else hi=m-1; }
  if(start>0) start--;
  ctx.lineWidth=1.3; ctx.strokeStyle=col;
  ctx.beginPath();
  ctx.moveTo(wx0,yt); ctx.lineTo(wx0,yb);
  let lastXe=wx0;
  for(let i=start;i<T.length&&T[i]<=S.t0+S.tspan+1;i++){
    const t=T[i]; const te=(i+1<T.length)? T[i+1] : S.t0+S.tspan+1;
    const xa=Math.max(wx0,timeToX(t)), xe=Math.min(wx1,timeToX(te));
    if(xe<=wx0) continue;
    const v=s.v[i];
    if(typeof v==='string'&&/[xz]/.test(v)){
      ctx.strokeStyle=COLORS.faint; ctx.strokeRect(lastXe+.5,yt+.5,Math.max(1,xa-lastXe-1),yb-yt-1);
      ctx.fillStyle=hatchPat; ctx.fillRect(lastXe,yt,xa-lastXe,yb-yt);
      ctx.strokeStyle=col;
    } else if(xa>lastXe){
      /* hexagon value cell */
      ctx.beginPath(); ctx.moveTo(lastXe,yt); ctx.lineTo(xa-4,yt); ctx.lineTo(xa,ym); ctx.lineTo(xa-4,yb);
      ctx.lineTo(lastXe,yb); ctx.lineTo(lastXe+4,ym); ctx.closePath(); ctx.stroke();
      if(xa-lastXe>38){ ctx.fillStyle=COLORS.txt; ctx.font='10px '+mono();
        const txt=fmtRadix(v,SIGS[si].w,radix);
        const half=(xa-lastXe)/2;
        ctx.fillText(truncate(txt,xa-lastXe-14), lastXe+7+Math.max(0,(half-30-ctx.measureText(truncate(txt,xa-lastXe-14)).width)/2), ym+3.5); }
    }
    lastXe=xa;
  }
  if(wx1>lastXe){ const v=valAt(si,S.t0+S.tspan);
    if(typeof v==='string'&&/[xz]/.test(v)){ ctx.fillStyle=hatchPat; ctx.fillRect(lastXe,yt,wx1-lastXe,yb-yt); }
    else { ctx.strokeStyle=col; ctx.beginPath(); ctx.moveTo(lastXe,yt); ctx.lineTo(wx1-4,yt); ctx.lineTo(wx1,ym);
      ctx.lineTo(wx1-4,yb); ctx.lineTo(lastXe,yb); ctx.lineTo(lastXe+4,ym); ctx.closePath(); ctx.stroke();
      if(wx1-lastXe>38){ ctx.fillStyle=COLORS.txt; ctx.font='10px '+mono();
        const txt=fmtRadix(v,SIGS[si].w,radix);
        ctx.fillText(truncate(txt,wx1-lastXe-14), lastXe+7, ym+3.5); } } }
  ctx.lineWidth=1;
  /* glitch diamonds */
  ctx.fillStyle=COLORS.bad;
  for(const gi of s.glitch){ const t=T[gi]; if(t<S.t0||t>S.t0+S.tspan) continue; const x=timeToX(t);
    ctx.beginPath(); ctx.moveTo(x,yt-2); ctx.lineTo(x+4,ym); ctx.lineTo(x,yb+2); ctx.lineTo(x-4,ym); ctx.closePath(); ctx.fill(); }
}
function drawAnalog(si,wx0,wx1,y,h,col){
  const s=STEPS[si]; if(!s||!s.n) return;
  const T=s.t; const yt=y+5, yb=y+h-5;
  let vmin=Infinity, vmax=-Infinity;
  for(let i=0;i<T.length;i++){ const v=s.v[i]; if(typeof v!=='number') continue;
    if(v<vmin) vmin=v; if(v>vmax) vmax=v; }
  if(!isFinite(vmin)){ drawBus(si,wx0,wx1,y,h,col,'hex'); return; }
  if(vmax-vmin<1e-9){ vmax=vmin+1; }
  const Y=v=> yb-(v-vmin)/(vmax-vmin)*(yb-yt);
  const grad=ctx.createLinearGradient(0,yt,0,yb);
  grad.addColorStop(0,col+'55'); grad.addColorStop(1,col+'00');
  ctx.beginPath(); let pen=false, started=false;
  for(let i=0;i<T.length;i++){ if(T[i]<S.t0-1) continue; if(T[i]>S.t0+S.tspan+1) break;
    const v=s.v[i]; if(typeof v!=='number'){ pen=false; continue; }
    const x=timeToX(T[i]), yy=Y(v);
    if(!started){ ctx.moveTo(wx0,yy); ctx.lineTo(x,yy); started=true; } else if(pen) ctx.lineTo(x,yy);
    else { ctx.moveTo(x,yy); pen=true; } }
  ctx.strokeStyle=col; ctx.lineWidth=1.4; ctx.stroke();
  /* fill under last stroke via re-walk */
  ctx.beginPath(); started=false;
  for(let i=0;i<T.length;i++){ if(T[i]<S.t0-1) continue; if(T[i]>S.t0+S.tspan+1) break;
    const v=s.v[i]; if(typeof v!=='number') continue;
    const x=timeToX(T[i]), yy=Y(v);
    if(!started){ ctx.moveTo(x,yy); started=true; } else ctx.lineTo(x,yy); }
  ctx.lineTo(wx1,yb); ctx.lineTo(wx0,yb); ctx.closePath(); ctx.fillStyle=grad; ctx.fill();
  ctx.lineWidth=1;
  const lbl='analog ['+vmin+' … '+vmax+']';
  ctx.fillStyle=COLORS.faint; ctx.font='9px '+mono(); ctx.fillText(lbl, waveX0()+4, yt+9);
}
function drawBitLane(e){
  const r=e.r, b=e.bit, y=e.y, h=S.bitH;
  ctx.fillStyle=COLORS.bg2||COLORS.panel2; ctx.fillRect(0,y,W,h);
  ctx.fillStyle=COLORS.faint; ctx.font='9px '+mono();
  ctx.fillText('  ['+b+']', S.nameW-34, y+h-4);
  const t=S.curA!==null? S.curA: S.t0;
  const full=valAt(r.si,t);
  let bit='x';
  if(typeof full==='number') bit=((full>>>b)&1)+'';
  else if(typeof full==='string'&&!/[xz]/.test(full)&&full.length>b) bit=full[full.length-1-b]==='1'?'1':'0';
  ctx.fillStyle=COLORS.txt; ctx.fillText(bit, S.nameW+8, y+h-4);
  const wx0=Math.max(timeToX(Math.max(0,S.t0)),waveX0()), wx1=Math.min(timeToX(S.t0+S.tspan),waveX1());
  /* mini trace for this bit */
  const s=STEPS[r.si]; if(!s||!s.n) return;
  const y1=y+2, y0=y+h-2, ym=(y1+y0)/2;
  ctx.strokeStyle=COLORS.bus; ctx.lineWidth=1; ctx.beginPath();
  let px=wx0, py=ym;
  for(let i=0;i<s.n;i++){ if(s.t[i]>S.t0+S.tspan) break;
    const v=s.v[i];
    const bitv=(typeof v==='number')? ((v>>>b)&1) : ((typeof v==='string'&&v.length>b&&!/[xz]/.test(v))? (v[v.length-1-b]==='1'?1:0) : null);
    if(bitv===null) continue;
    const ny=bitv? y1:y0; const x=timeToX(s.t[i]);
    if(x<wx0){ px=x; py=ny; continue; }
    if(px<x){ ctx.lineTo(x,py); ctx.lineTo(x,ny); }
    px=x; py=ny;
  }
  ctx.lineTo(wx1,py); ctx.stroke();
}
function truncate(s,maxw){ if(ctx.measureText(s).width<=maxw) return s;
  let out=''; for(const ch of s){ if(ctx.measureText(out+'…').width>maxw-6) return out+'…'; out+=ch; } return out+'…'; }

/* ================= cursors / hover / marquee / scrollbar ================= */
function drawCursors(){
  if(S.curA!==null){ const x=timeToX(S.curA);
    ctx.strokeStyle=COLORS.cursorA; ctx.setLineDash([5,3]); ctx.lineWidth=1.2;
    ctx.beginPath(); ctx.moveTo(x+.5,AX); ctx.lineTo(x+.5,H); ctx.stroke(); ctx.setLineDash([]);
    ctx.fillStyle=COLORS.cursorA; ctx.font='9.5px '+mono();
    const lb='A '+fmtT(S.curA); ctx.fillText(lb,x+4,AX+11);
  }
  if(S.curB!==null){ const x=timeToX(S.curB);
    ctx.strokeStyle=COLORS.cursorB; ctx.setLineDash([3,3]); ctx.lineWidth=1.2;
    ctx.beginPath(); ctx.moveTo(x+.5,AX); ctx.lineTo(x+.5,H); ctx.stroke(); ctx.setLineDash([]);
    ctx.fillStyle=COLORS.cursorB; ctx.font='9.5px '+mono();
    const lb='B '+fmtT(S.curB); ctx.fillText(lb,x+4,AX+23);
  }
  if(S.curA!==null&&S.curB!==null){
    const dt=Math.abs(S.curB-S.curA);
    const f=dt>0? (1000/dt):0;
    const x1=Math.min(timeToX(S.curA),timeToX(S.curB)), x2=Math.max(timeToX(S.curA),timeToX(S.curB));
    ctx.fillStyle=COLORS.cursorB+'22'; ctx.fillRect(x1,AX,x2-x1,H-AX);
    ctx.fillStyle=COLORS.panel2; ctx.strokeStyle=COLORS.edge2;
    const txt='Δt '+fmtT(dt)+' ns · f '+(f?f.toFixed(3):'∞')+' MHz';
    const tw=ctx.measureText(txt).width+16;
    const bx=(x1+x2)/2-tw/2, by=H-34;
    ctx.beginPath(); ctx.roundRect(bx,by,tw,20,5); ctx.fill(); ctx.stroke();
    ctx.fillStyle=COLORS.cursorB; ctx.font='10px '+mono(); ctx.fillText(txt,bx+8,by+13);
  }
}
let lastMouse={x:0,y:0,lx:0,ly:0,ly0:0,inWave:false};
function drawHover(){
  if(!lastMouse.inWave||S.marquee||dragState) return;
  const x=lastMouse.x;
  ctx.strokeStyle=COLORS.dim+'55'; ctx.beginPath(); ctx.moveTo(x+.5,AX); ctx.lineTo(x+.5,H); ctx.stroke();
}
function drawMarquee(){
  if(!S.marquee) return;
  const {x0,x1}=S.marquee;
  ctx.fillStyle=COLORS.accent+'18';
  const a=Math.min(x0,x1), b=Math.max(x0,x1);
  ctx.fillRect(a,AX,b-a,H-AX);
  ctx.strokeStyle=COLORS.accent; ctx.setLineDash([3,2]);
  ctx.strokeRect(a+.5,AX+.5,b-a-1,H-AX-1); ctx.setLineDash([]);
  const t1=xToTime(a), t2=xToTime(b);
  ctx.fillStyle=COLORS.accent; ctx.font='10px '+mono();
  ctx.fillText('zoom → ['+fmtT(Math.min(t1,t2))+' … '+fmtT(Math.max(t1,t2))+']  ('+fmtT(Math.abs(t2-t1))+' ns)', a+4, AX+12);
}
function drawScrollbar(){
  const ch=contentH()+AX;
  if(ch<=H) return;
  const vh=H*H/ch, vy=AX+(H-AX)*(S.scrollY/(ch-AX-(H-AX)));
  ctx.fillStyle=COLORS.edge2;
  ctx.beginPath(); ctx.roundRect(W-SB+2.5,vy+AX*0,SB-5,vh,3); ctx.fill();
}
function scrollIntoView(rowObj){
  for(const e of LAYOUT){ if(e.r===rowObj||e.kind==='sig'&&e.r===rowObj){
    if(e.y<AX){ S.scrollY-= (AX-e.y); return true; }
    if(e.y+e.h>H){ S.scrollY+= e.y+e.h-H; return true; } } }
  return false;
}
/* ================= minimap ================= */
let mmHist=null;
function buildHist(){
  const N=400; mmHist=new Float32Array(N);
  for(let si=0;si<STEPS.length;si++){ const T=STEPS[si].t;
    for(let i=0;i<T.length;i++){ const b=Math.min(N-1,Math.floor(T[i]/Math.max(END,1)*N)); mmHist[b]++; } }
  let mx=0; for(const v of mmHist) mx=Math.max(mx,v);
  for(let i=0;i<N;i++) mmHist[i]/=mx||1;
}
function drawMM(){
  const w=mm.clientWidth,h=mm.clientHeight;
  mctx.clearRect(0,0,w,h);
  mctx.fillStyle=COLORS.bg2; mctx.fillRect(0,0,w,h);
  if(mmHist){ mctx.fillStyle=COLORS.accent+'33';
    const N=mmHist.length, bw=w/N;
    for(let i=0;i<N;i++){ const bh=mmHist[i]*(h-8); if(bh>0) mctx.fillRect(i*bw,h-4-bh,Math.max(1,bw-0.4),bh); } }
  const x0=timeToX(0), x1=timeToX(END);
  const vx=Math.max(0,x0/(W-SB)*w), vw=Math.max(6,((x1-x0)/(W-SB))*w);
  mctx.fillStyle=(COLORS.accent||'#22d3ee')+'38'; mctx.fillRect(vx,2,vw,h-4);
  mctx.strokeStyle=COLORS.accent; mctx.lineWidth=1.5; mctx.strokeRect(vx+.5,2.5,vw-1,h-5); mctx.lineWidth=1;
  mctx.fillStyle=COLORS.accent; mctx.fillRect(vx,0,vw,2); mctx.fillRect(vx,h-2,vw,2);
}
function sbUpdate(){
  const f=(id)=>$(id);
  f('#sb-time').innerHTML=svgIcon('clock')+' t = <b>'+fmtT(S.curA!==null?S.curA:Math.round(S.t0))+'</b> ns';
  f('#sb-cursor').innerHTML=svgIcon('cursor')+' A: <b>'+(S.curA!==null? fmtT(S.curA):'—')+'</b> · B: <b>'+(S.curB!==null? fmtT(S.curB):'—')+'</b>';
  const dt=(S.curA!==null&&S.curB!==null)? Math.abs(S.curB-S.curA):null;
  f('#sb-dt').innerHTML=svgIcon('measure')+' Δt: <b>'+(dt!==null? fmtT(dt)+' ns':'—')+'</b>'+(dt?' · '+(1000/dt).toFixed(3)+' MHz':'');
  f('#sb-zoom').innerHTML=svgIcon('fit')+' zoom <b>'+Math.round(END/S.tspan*100)+'%</b>';
  f('#sb-events').innerHTML=svgIcon('activity')+' <b>'+fmtInt(NEV)+'</b> events · <b>'+SIGS.length+'</b> sigs';
  f('#statsmini').innerHTML='<span>sig <b>'+flatRows().filter(e=>e.r.type==='sig').length+'</b></span><span>range <b>'+fmtT(S.t0)+' … '+fmtT(S.t0+S.tspan)+'</b> ns</span>';
}
function updGauge(){
  const g=$('#gauge'); const f=LAYOUT.find(e=>e.kind==='sig'&&S.sel.has(e.r));
  let html='';
  if(f){ const sig=SIGS[f.r.si];
    const va=S.curA!==null? valAt(f.r.si,S.curA):'—';
    const vb=S.curB!==null? valAt(f.r.si,S.curB):'—';
    html='<div class="gc">sel <b>'+esc(truncateTxt(sig.n,26))+'</b></div>'
      +'<div class="gc">A <b>'+(va==='—'?'—':esc(fmtRadix(va,sig.w,f.r.radix||S.defRadix)))+'</b></div>'
      +'<div class="gc">B <b>'+(vb==='—'?'—':esc(fmtRadix(vb,sig.w,f.r.radix||S.defRadix)))+'</b></div>'
      +'<div class="gc">edges <b>'+fmtInt(STEPS[f.r.si].n)+'</b></div>';
  } else html='<div class="gc">no selection — click a row</div>';
  html+='<div class="gsp"></div><div class="ghint"><span class="kbd">wheel</span> zoom · <span class="kbd">drag</span> pan · <span class="kbd">click</span> cursor · <span class="kbd">shift+click</span> cursor B · <span class="kbd">right-click</span> menu · <span class="kbd">Ctrl+K</span> palette</div>';
  g.innerHTML=html;
}
function truncateTxt(s,w){ return s.length>w? s.slice(0,w-1)+'…':s; }

/* ================= zoom / nav ================= */
function fit(){ S.t0=0; S.tspan=Math.max(END,1); draw(); drawMM(); }
function zoom(f,cx){ const c=cx!==undefined? xToTime(cx): S.t0+S.tspan/2;
  const oldSpan=S.tspan;
  S.tspan=Math.min(Math.max(END,1)*2, Math.max(1, S.tspan*f));
  const ratio=S.tspan/oldSpan;
  S.t0=Math.max(0, c-(c-S.t0)*ratio);
  draw(); drawMM(); }
function gotoT(t,center){ const t2=Math.max(0,Math.min(END,t));
  if(center) S.t0=Math.max(0,t2-S.tspan/2);
  draw(); drawMM(); }
function edgeJump(dir){ const f=LAYOUT.find(e=>e.kind==='sig'&&S.sel.has(e.r)); if(!f){ toast('Select a row first','warn'); return; }
  const ref=S.curA!==null? S.curA: S.t0;
  const t=dir<0? prevChange(f.r.si,ref): nextChange(f.r.si,ref);
  if(t===null||t===undefined){ toast(dir<0?'no earlier edge':'no later edge','warn'); return; }
  S.curA=t; if(t<S.t0+S.tspan*0.05||t>S.t0+S.tspan*0.95) gotoT(t,true); draw(); sbUpdate(); }

/* ================= interactions ================= */
let dragState=null; // {mode:'pan'|'marquee'|'split'|'mm'|'sb'|'row', ...}
let hoverTimer=null;
function hitRow(y){ for(const e of LAYOUT){ if(y>=e.y&&y<e.y+e.h) return e; } return null; }
function waveHit(x,y){ // what zone: 'name'|'value'|'wave'|'axis'
  if(y<AX) return 'axis';
  if(x<S.nameW) return 'name'; if(x<S.nameW+S.valW) return 'value'; return 'wave'; }
function waveLocal(ev){ const r=cv.getBoundingClientRect();
  return {x:ev.clientX-r.left, y:ev.clientY-r.top}; }
cv.addEventListener('mousedown',ev=>{
  const {x,y}=waveLocal(ev); const zone=waveHit(x,y);
  closeCtxMenu(); hideTip();
  if(ev.button===2) return; // context menu handled elsewhere
  if(zone==='axis'){ dragState={mode:'marquee',x0:x,x1:x}; S.marquee=dragState; }
  else if(zone==='wave'&&S.regionMode){ dragState={mode:'marquee',x0:x,x1:x}; S.marquee=dragState; }
  else if(zone==='wave'){ dragState={mode:'pan',x,t:S.t0,y}; cv.style.cursor='grabbing'; }
  else if(zone==='name'){ const e=hitRow(y);
    if(e&&(e.kind==='sig'||e.kind==='group')){
      if(ev.shiftKey||ev.ctrlKey||ev.metaKey){ S.sel.has(e.r)? S.sel.delete(e.r): S.sel.add(e.r); }
      else if(!S.sel.has(e.r)){ S.sel.clear(); S.sel.add(e.r); }
      dragState={mode:'row',row:e.r,y0:y,moved:false};
      draw(); updProps();
    } }
});
cv.addEventListener('mousemove',ev=>{
  const {x,y}=waveLocal(ev); lastMouse={x,y,lx:x,ly:y,inWave:x>=S.nameW+S.valW};
  if(dragState){
    if(dragState.mode==='pan'){ S.t0=Math.max(0,dragState.t-(x-dragState.x)/((waveX1()-waveX0()))*S.tspan);
      if(S.t0===0&&dragState.t===0){} draw(); drawMM(); }
    else if(dragState.mode==='marquee'){ dragState.x1=x; draw(); }
    else if(dragState.mode==='row'){ if(Math.abs(y-dragState.y0)>6) dragState.moved=true; }
    return;
  }
  const e=hitRow(y);
  cv.style.cursor= e? 'pointer': 'crosshair';
  clearTimeout(hoverTimer);
  if(e&&e.kind==='sig'){ hoverTimer=setTimeout(()=>showTip(x,y,e),260); }
  draw();
});
cv.addEventListener('mouseleave',()=>{ clearTimeout(hoverTimer); hideTip(); lastMouse.inWave=false; draw(); });
window.addEventListener('mouseup',ev=>{
  if(!dragState) return;
  if(dragState.mode==='marquee'&&S.marquee){
    const a=Math.min(S.marquee.x0,S.marquee.x1), b=Math.max(S.marquee.x0,S.marquee.x1);
    if(b-a>8){ const t1=xToTime(a),t2=xToTime(b);
      S.t0=Math.max(0,Math.min(t1,t2)); S.tspan=Math.max(1,Math.abs(t2-t1)); toast('Zoomed to ['+fmtT(S.t0)+' … '+fmtT(S.t0+S.tspan)+']','region'); }
    S.marquee=null;
  }
  dragState=null; cv.style.cursor='default'; draw(); drawMM();
});
cv.addEventListener('wheel',ev=>{
  ev.preventDefault();
  const {x,y}=waveLocal(ev);
  if(ev.shiftKey){ S.scrollY=Math.max(0,Math.min(Math.max(0,contentH()+AX-H),S.scrollY+(ev.deltaY>0?60:-60))); draw(); return; }
  const f=ev.deltaY>0? 1.18: 1/1.18;
  zoom(f,x);
},{passive:false});
cv.addEventListener('click',ev=>{
  const {x,y}=waveLocal(ev); const zone=waveHit(x,y);
  if(zone!=='wave'&&zone!=='value') return;
  const t=Math.max(0,Math.min(END,xToTime(x)));
  if(S.measureMode||ev.shiftKey){ S.curB=t; toast('Cursor B @ '+fmtT(t)+' ns — Δt '+(S.curA!==null?fmtT(Math.abs(t-S.curA))+' ns':'?'),'measure'); }
  else { S.curA=t; S.curB=null; }
  draw(); sbUpdate(); updProps();
});
cv.addEventListener('dblclick',ev=>{
  const {x,y}=waveLocal(ev); const e=hitRow(y);
  if(e&&e.kind==='sig'&&SIGS[e.r.si].w>1){ e.r.expanded=!e.r.expanded; layout(); draw(); }
  else if(e&&e.kind==='sig'&&SIGS[e.r.si].w>=8){ e.r.analog=!e.r.analog; layout(); draw(); }
});
cv.addEventListener('contextmenu',ev=>{
  ev.preventDefault();
  const {x,y}=waveLocal(ev);
  const e=hitRow(y);
  const zone=waveHit(x,y);
  const ct=(zone==='wave'||zone==='value')? Math.max(0,Math.min(END,xToTime(x))):null;
  ctxMenu(buildCtxMenu(e,zone,ct), ev.clientX,ev.clientY);
});
/* splitters + tree/prop dock resize */
function bindSplit(){
  // name/value splitter via dedicated hot zones on the axis strip
  cv.addEventListener('mousedown',ev=>{ const {x}=waveLocal(ev);
    if(ev.clientY-cv.getBoundingClientRect().top>AX) return;
    if(Math.abs(x-S.nameW)<4){ dragState={mode:'split',which:'nameW'}; }
    else if(Math.abs(x-(S.nameW+S.valW))<4){ dragState={mode:'split',which:'valW'}; } });
  window.addEventListener('mousemove',ev=>{ if(dragState&&dragState.mode==='split'){
    const r=cv.getBoundingClientRect(); const x=ev.clientX-r.left;
    if(dragState.which==='nameW'){ S.nameW=Math.max(120,Math.min(520,x)); }
    else { S.valW=Math.max(56,Math.min(260,x-S.nameW)); } draw(); } });
}
bindSplit();
/* minimap drag */
mm.addEventListener('mousedown',ev=>{ const r=mm.getBoundingClientRect();
  dragState={mode:'mm',x:ev.clientX-r.left,w:r.width}; mmJump(ev); });
mm.addEventListener('mousemove',ev=>{ if(dragState&&dragState.mode==='mm') mmJump(ev); });
function mmJump(ev){ const frac=dragState.x/dragState.w;
  const c=frac*END; S.t0=Math.max(0,Math.min(END-S.tspan,c-S.tspan/2)); draw(); drawMM(); }

/* ================= keyboard ================= */
window.addEventListener('keydown',ev=>{
  const tag=(ev.target.tagName||'').toLowerCase();
  if(tag==='input'||tag==='textarea'){ 
    if(ev.key==='Escape') ev.target.blur();
    return; }
  const k=ev.key;
  if((ev.ctrlKey||ev.metaKey)){
    switch(k){
      case 'k': ev.preventDefault(); openPalette(); return;
      case 'o': ev.preventDefault(); openDump(); return;
      case 's': ev.preventDefault(); saveLayout(); return;
      case 'p': ev.preventDefault(); exportPNG(ev.shiftKey); return;
      case 'f': ev.preventDefault(); focusSearch(); return;
      case '1': ev.preventDefault(); toggleDock('#treedock'); return;
      case '2': ev.preventDefault(); toggleDock('#propdock'); return;
      case '3': ev.preventDefault(); toggleDock('#resultsdock',true); return;
    }
    return;
  }
  switch(k){
    case 'F1': ev.preventDefault(); helpDialog(); return;
    case '+': case '=': zoom(0.6); return;
    case '-': case '_': zoom(1.6); return;
    case 'f': case 'F': fit(); return;
    case 'ArrowLeft': if(ev.shiftKey){ edgeJump(-1); } else { S.t0=Math.max(0,S.t0-S.tspan*0.1); draw(); drawMM(); } return;
    case 'ArrowRight': if(ev.shiftKey){ edgeJump(1); } else { S.t0=Math.max(0,Math.min(END-S.tspan,S.t0+S.tspan*0.1)); draw(); drawMM(); } return;
    case 'ArrowUp': ev.preventDefault(); selShift(-1); return;
    case 'ArrowDown': ev.preventDefault(); selShift(1); return;
    case 'Home': S.t0=0; draw(); drawMM(); return;
    case 'End': S.t0=Math.max(0,END-S.tspan); draw(); drawMM(); return;
    case 'PageUp': S.scrollY=Math.max(0,S.scrollY-H*.8); draw(); return;
    case 'PageDown': S.scrollY=Math.min(Math.max(0,contentH()+AX-H),S.scrollY+H*.8); draw(); return;
    case '[': jumpResult(-1); return;
    case ']': jumpResult(1); return;
    case 'Delete': case 'x': case 'X': if(S.sel.size){ removeSel(); toast('Removed '+S.sel.size+' row(s)','trash'); } return;
    case 'g': case 'G': groupSel(); return;
    case 'u': case 'U': ungroupSel(); return;
    case 'e': expandSel(true); return;
    case 'E': expandSel(false); return;
    case 'a': case 'A': toggleAnalog(); return;
    case 'c': setCursorA(S.t0+S.tspan/2); return;
    case 'C': clearCursors(); return;
    case 'm': case 'M': toggleMeasure(); return;
    case 'z': case 'Z': toggleRegion(); return;
    case 't': case 'T': cycleTheme(); return;
    case '`': toggleConsole(); return;
    case '?': helpDialog(); return;
    case 'Escape': closeCtxMenu(); hideTip(); $('#palwrap').style.display='none'; return;
  }
  // PageUp/Down via keys handled; letters b/r for radix
  if(k==='b'){ setRadixAll('bin'); return; }
  if(k==='h'){ setRadixAll('hex'); return; }
});
function selShift(d){
  const flat=flatRows(); if(!flat.length) { addRow(0); return; }
  let idx=flat.findIndex(e=>S.sel.has(e.r));
  if(idx<0) idx=0; else idx=(idx+d+flat.length)%flat.length;
  S.sel.clear(); S.sel.add(flat[idx].r);
  if(scrollIntoView(flat[idx].r)) {} draw(); updProps();
}
/* ================= context menu ================= */
function ctxMenu(items,x,y){
  const m=$('#ctxmenu'); m.innerHTML='';
  for(const it of items){
    if(it==='sep'){ const d=document.createElement('div'); d.className='msep'; m.appendChild(d); continue; }
    const d=document.createElement('div'); d.className='mi';
    d.innerHTML=svgIcon(it[0])+'<span>'+it[1]+'</span>'+(it[3]? '<span class="sub">'+it[3]+'▸</span>':'');
    d.onclick=(e)=>{ e.stopPropagation(); closeCtxMenu(); it[2](); };
    m.appendChild(d);
  }
  m.style.display='block';
  const mw=m.offsetWidth, mh=m.offsetHeight;
  m.style.left=Math.min(x,innerWidth-mw-8)+'px';
  m.style.top=Math.min(y,innerHeight-mh-8)+'px';
}
function closeCtxMenu(){ $('#ctxmenu').style.display='none'; }
document.addEventListener('click',closeCtxMenu);
function buildCtxMenu(e,zone,t){
  const items=[];
  if(t!==null&&t!==undefined){
    items.push(['cursor','Cursor A @ '+fmtT(t),()=>{S.curA=t;S.curB=null;draw();sbUpdate();}]);
    items.push(['measure','Cursor B @ '+fmtT(t),()=>{S.curB=t;draw();sbUpdate();}]);
    items.push(['measure','Δ measure from A',()=>{S.curB=t;toast('Δt = '+fmtT(Math.abs(t-(S.curA||0)))+' ns','measure');draw();sbUpdate();}]);
    items.push('sep');
  }
  if(e&&e.kind==='sig'){
    const sig=SIGS[e.r.si];
    items.push(['expand',e.r.expanded?'Collapse bus':'Expand bus ['+(sig.w-1)+':0]',()=>{e.r.expanded=!e.r.expanded;layout();draw();},'E']);
    if(sig.w>=8) items.push(['analog',e.r.analog?'Digital trace':'Analog trace',()=>{e.r.analog=!e.r.analog;layout();draw();},'A']);
    items.push(['group','Group from here',()=>{ groupFrom(e.r); }]);
    items.push('sep');
    items.push(['radar','Radix…',()=>ctxMenu([['radar','Hex',()=>{e.r.radix='hex';draw();updProps();}],['radar','Binary',()=>{e.r.radix='bin';draw();updProps();}],['radar','Decimal',()=>{e.r.radix='dec';draw();updProps();}],['radar','Signed',()=>{e.r.radix='signed';draw();updProps();}],['radar','ASCII',()=>{e.r.radix='ascii';draw();updProps();}],['radar','Default',()=>{e.r.radix=null;draw();updProps();}]],lastMouse.x+40,lastMouse.y),'R▸']);
    items.push(['palette','Color…',()=>colorMenuFor(e.r),'▸']);
    items.push('sep');
    items.push(['trash','Remove row',()=>{ removeOne(e.r); },'Del']);
  } else if(e&&e.kind==='group'){
    items.push(['ungroup','Ungroup',()=>{S.sel.add(e.r);ungroupSel();},'U']);
    items.push(['chevD',e.r.open?'Close group':'Open group',()=>{e.r.open=!e.r.open;layout();draw();}]);
  } else {
    items.push(['check','Add all signals',addAllRows]);
  }
  if(zone==='wave') items.push(['fit','Fit timeline',fit,'F']);
  if(S.results.length) items.push(['next','Next result',()=>jumpResult(1),']']);
  return items;
}
function removeOne(r){ const rm=(rs)=>rs.filter(x=>{ if(x.type==='group'){ x.children=rm(x.children); return x.children.length||rs.length===0&&false; } return x!==r; });
  S.rows=(function rec(rs){ const out=[]; for(const x of rs){ if(x===r) continue;
    if(x.type==='group'){ x.children=rec(x.children); if(!x.children.length) continue; }
    out.push(x);} return out; })(S.rows);
  S.sel.delete(r); syncTree(); layout(); draw(); }
function groupFrom(r){ const i=S.rows.indexOf(r); if(i<0||i>=S.rows.length-1) return;
  const kids=S.rows.splice(i,2); const g={type:'group',name:shortName(kids[0].si),open:true,children:kids};
  S.rows.splice(i,0,g); S.sel.clear(); S.sel.add(g); layout(); draw(); }
const PALETTE12=['#89e05a','#22d3ee','#38bdf8','#fbbf24','#fb7185','#a78bfa','#2dd4bf','#f87171','#e8b93f','#f472b6','#60a5fa','#a3e635'];
function colorMenuFor(r){ ctxMenu(PALETTE12.map(c=>['palette','color',()=>{r.color=c;draw();updProps();},null]).map(it=>{it[0]='palette';return it;}),lastMouse.x,lastMouse.y);
  // replace items with swatch rendering
  const m=$('#ctxmenu'); m.innerHTML='';
  const grid=document.createElement('div'); grid.className='swatch-row'; grid.style.padding='6px';
  for(const c of PALETTE12){ const s=document.createElement('div'); s.className='swatch'; s.style.background=c;
    s.onclick=()=>{ r.color=c; closeCtxMenu(); draw(); updProps(); }; grid.appendChild(s); }
  m.appendChild(grid);
  const clr=document.createElement('div'); clr.className='mi'; clr.innerHTML=svgIcon('refresh')+'<span>Reset color</span>';
  clr.onclick=()=>{ r.color=null; closeCtxMenu(); draw(); updProps(); }; m.appendChild(clr);
  m.style.display='block';
}
/* ================= tooltip ================= */
function showTip(x,y,e){
  const si=e.r.si, sig=SIGS[si];
  const t=Math.max(0,Math.min(END,xToTime(x)));
  const v=valAt(si,t);
  const nxt=nextChange(si,t);
  const tip=$('#tip');
  const isX=typeof v==='string'&&/[xz]/.test(v);
  tip.innerHTML='<div class="tn">'+esc(sig.n)+'</div>'
    +'<div>value <span class="'+(isX?'tx':'tv')+'">'+esc(fmtRadix(v,sig.w,e.r.radix||S.defRadix))+'</span> <span class="td">'+sig.w+'b · '+sig.k+'</span></div>'
    +'<div class="td">t = '+fmtT(t)+' ns</div>'
    +(nxt!==null? '<div class="td">next Δ '+fmtT(nxt-t)+' ns @ '+fmtT(nxt)+'</div>':'<div class="td">stable to end</div>')
    +'<div class="tx">edges '+fmtInt(STEPS[si].n)+' · unique '+fmtInt(STEPS[si].uniq)+'</div>';
  const r=cv.getBoundingClientRect();
  let px=r.left+x+16, py=r.top+y+14;
  tip.style.display='block';
  const tw=tip.offsetWidth, th=tip.offsetHeight;
  if(px+tw>innerWidth-8) px=r.left+x-tw-16;
  if(py+th>innerHeight-8) py=r.top+y-th-12;
  tip.style.left=px+'px'; tip.style.top=py+'px';
}
function hideTip(){ $('#tip').style.display='none'; }
/* ================= toasts + console ================= */
function toast(msg,ic,kind){
  const t=document.createElement('div'); t.className='toast '+(kind||'');
  t.innerHTML=svgIcon(ic||'info')+'<span>'+msg+'</span>';
  $('#toasts').appendChild(t);
  logC('['+(ic||'info')+'] '+msg.replace(/<[^>]+>/g,''));
  setTimeout(()=>{ t.classList.add('out'); setTimeout(()=>t.remove(),200); },2600);
}
function logC(line,cls){
  const c=$('#consoleout'); const d=document.createElement('div');
  d.className='cline '+(cls||''); d.textContent=line; c.appendChild(d);
  if(c.children.length>400) c.removeChild(c.firstChild);
  c.scrollTop=c.scrollHeight;
}
/* ================= properties ================= */
function updProps(){
  const P=$('#props');
  const f=LAYOUT.find(e=>e.kind==='sig'&&S.sel.has(e.r));
  if(!f){ P.innerHTML='<div class="psec"><div class="psec-h">'+svgIcon('info')+'No selection</div>'
    +'<div class="prow"><span class="pk">hint</span><span class="pv">click a row · ctrl-click multi</span></div>'
    +'<div class="prow"><span class="pk">stats</span><span class="pv">'+SIGS.length+' signals · '+fmtInt(NEV)+' events</span></div></div>'
    +'<div class="psec"><div class="psec-h">'+svgIcon('zap')+'Quick actions</div><div class="pbtnrow">'
    +pbtn('check','Add all','addAllRows')+pbtn('flag','Analyze','showTab("hub")')+pbtn('terminal','Console','toggleConsole')+'</div></div>';
    return; }
  const si=f.r.si, sig=SIGS[si], s=STEPS[si];
  const va=S.curA!==null? valAt(si,S.curA):null;
  const first=s.n? s.t[0]:null, last=s.n? s.t[s.n-1]:null;
  const active=s.n? ((s.n)/(Math.max(END,1))*100).toFixed(1):'0';
  P.innerHTML=
   '<div class="psec"><div class="psec-h">'+svgIcon('target')+esc(shortName(si))+'</div>'
   +'<div class="prow"><span class="pk">full name</span><span class="pv big">'+esc(sig.n)+'</span></div>'
   +'<div class="prow"><span class="pk">kind</span><span class="pv">'+sig.k+'</span></div>'
   +'<div class="prow"><span class="pk">width</span><span class="pv">'+sig.w+' bit</span></div>'
   +'<div class="prow"><span class="pk">radix</span><span class="pv">'+(f.r.radix||S.defRadix)+'</span></div></div>'
   +'<div class="psec"><div class="psec-h">'+svgIcon('activity')+'Activity</div>'
   +'<div class="prow"><span class="pk">edges</span><span class="pv">'+fmtInt(s.n)+'</span></div>'
   +'<div class="prow"><span class="pk">unique values</span><span class="pv">'+fmtInt(s.uniq)+'</span></div>'
   +'<div class="prow"><span class="pk">glitches</span><span class="pv" style="color:'+(s.glitch.length?COLORS.bad:'inherit')+'">'+s.glitch.length+'</span></div>'
   +'<div class="prow"><span class="pk">first edge</span><span class="pv">'+(first!==null?fmtT(first)+' ns':'—')+'</span></div>'
   +'<div class="prow"><span class="pk">last edge</span><span class="pv">'+(last!==null?fmtT(last)+' ns':'—')+'</span></div>'
   +'<div class="prow"><span class="pk">activity</span><span class="pv">'+active+'%</span></div></div>'
   +'<div class="psec"><div class="psec-h">'+svgIcon('cursor')+'Values</div><table class="pval-table">'
   +'<tr><td>A@'+(S.curA!==null?fmtT(S.curA):'—')+'</td><td>'+esc(va!==null?fmtRadix(va,sig.w,f.r.radix||S.defRadix):'—')+'</td></tr>'
   +'<tr><td>B@'+(S.curB!==null?fmtT(S.curB):'—')+'</td><td>'+(S.curB!==null?esc(fmtRadix(valAt(si,S.curB),sig.w,f.r.radix||S.defRadix)):'—')+'</td></tr>'
   +'</table><div class="pbtnrow">'
   +pbtn('expand',f.r.expanded?'Collapse':'Expand bus','(()=>{arguments&&0})()') 
   +pbtn('analog','Analog','(()=>{})()')
   +'</div></div>';
  // rebind buttons properly
  const btns=P.querySelectorAll('.pbtn');
  const expBtn=btns[0], anaBtn=btns[1];
  expBtn.onclick=()=>{ if(sig.w>1){f.r.expanded=!f.r.expanded; layout(); draw(); updProps();} else toast('1-bit signal','warn'); };
  anaBtn.onclick=()=>{ if(sig.w>=8){f.r.analog=!f.r.analog; layout(); draw(); updProps();} else toast('width>=8 needed','warn'); };
}
function pbtn(ic,label,code){ return '<span class="pbtn" data-code="'+esc(code)+'">'+svgIcon(ic)+label+'</span>'; }

/* ================= signal tree ================= */
function buildTree(){
  const tree=$('#tree'); tree.innerHTML='';
  const filter=$('#tf').value.trim();
  let rx2=null;
  try{ rx2=filter? new RegExp(filter,'i'):null; }catch(e){ rx2=filter? new RegExp(filter.replace(/[.*+?^${}()|[\]\\]/g,'\\$&'),'i'):null; }
  const scopes={};
  SIGS.forEach((s,i)=>{
    const sc=s.n.split('.').slice(0,-1).join('.')||'top';
    (scopes[sc]=scopes[sc]||[]).push(i);
  });
  let shown=0, total=SIGS.length;
  for(const [sc,list] of Object.entries(scopes)){
    const vis=list.filter(i=>!rx2||rx2.test(SIGS[i].n));
    if(!vis.length) continue;
    const scope=document.createElement('div'); scope.className='scope';
    scope.innerHTML='<div class="scope-h">'+svgIcon('chevD')+esc(sc)+'<span class="sc-n">'+vis.length+'</span></div><div class="scope-kids"></div>';
    scope.querySelector('.scope-h').onclick=()=>scope.classList.toggle('closed');
    for(const i of vis){
      shown++;
      const s=SIGS[i];
      const d=document.createElement('div');
      d.className='sigrow'+(findRow(i)?' inview':'');
      d.title=s.n+' · '+s.w+'b · '+s.k;
      d.innerHTML='<span class="kdot" style="background:'+kindColor(s.k)+'"></span>'
        +'<span title="'+esc(s.n)+'">'+esc(s.n.split('.').pop())+'</span>'
        +(s.k!=='rtl'? '<span class="tag" style="margin-left:4px">'+s.k+'</span>':'')
        +'<span class="sr-w">'+s.w+'b</span><span class="sr-add" title="add to view">'+svgIcon('plus')+'</span>';
      d.onclick=(ev)=>{ if(ev.target.closest('.sr-add')) return;
        const row=findRow(i); if(row){ S.sel.clear(); S.sel.add(row); draw(); updProps(); return; }
        addRow(i); toast('Added '+esc(s.n),'check'); };
      d.querySelector('.sr-add').onclick=(ev)=>{ ev.stopPropagation(); if(!findRow(i)){ addRow(i); toast('Added '+esc(s.n),'check'); } };
      d.ondblclick=()=>{ const row=findRow(i)||addRow(i); S.sel.clear(); S.sel.add(row);
        const f=LAYOUT.find(e=>e.r===row); if(f){ S.curA=f.kind==='sig'? (STEPS[i].n?STEPS[i].t[0]:0):0; }
        draw(); sbUpdate(); };
      scope.querySelector('.scope-kids').appendChild(d);
    }
    tree.appendChild(scope);
  }
  $('#treestats').textContent=filter? shown+' / '+total+' shown' : total+' signals · '+Object.keys(scopes).length+' scopes';
}
function syncTree(){ buildTree(); }
$('#tf').addEventListener('input',buildTree);
$('#tf').addEventListener('keydown',ev=>{ if(ev.key==='Enter'){ const first=$('#tree .sigrow'); if(first) first.click(); } });

/* ================= search ================= */
function focusSearch(){ $('#q').focus(); $('#q').select(); }
function cycleSearchMode(){ S.searchMode={signal:'expr',expr:'goto',goto:'signal'}[S.searchMode];
  updSearchMode(); toast('Search mode: '+S.searchMode.toUpperCase(),'search'); }
function updSearchMode(){
  $('#searchmode').textContent=S.searchMode.toUpperCase();
  $('#q').placeholder={ signal:'search signal (add: prefix adds to view)',
    expr:'boolean expr: valid && !ready || stall',
    goto:'goto time: 123 or 123ns or goto:123'}[S.searchMode];
}
function runSearch(){
  const q=$('#q').value.trim();
  if(!q){ toast('Empty query','warn'); return; }
  if(S.searchMode==='goto'||/^(goto:)?\d+(\.\d+)?\s*(ns)?$/.test(q)&&S.searchMode==='goto'){
    const m=q.match(/[\d.]+/); if(m){ setCursorA(parseFloat(m[0])); gotoT(parseFloat(m[0]),true); toast('Jumped to '+m[0]+' ns','clock'); } return;
  }
  if(S.searchMode==='signal'){
    let rx; try{ rx=new RegExp(q,'i'); }catch(e){ rx=new RegExp(q.replace(/[.*+?^${}()|[\]\\]/g,'\\$&'),'i'); }
    const hits=SIGS.map((s,i)=>({s,i})).filter(x=>rx.test(x.s.n));
    S.results=hits.map(h=>({kind:'sig',si:h.i,label:h.s.n}));
    if(/^add:\s*/i.test(q)){ // add all matches
      const base=q.replace(/^add:\s*/i,''); let rx2; try{ rx2=new RegExp(base,'i'); }catch(e){ rx2=new RegExp(base.replace(/[.*+?^${}()|[\]\\]/g,'\\$&'),'i'); }
      let n=0; SIGS.forEach((s,i)=>{ if(rx2.test(s.n)&&!findRow(i)){ addRowSilent(i); n++; } });
      syncTree(); layout(); draw(); toast('Added '+n+' signals','check'); return;
    }
    if(hits.length){ showResults(S.results, 'Signals ('+hits.length+')');
      toast(hits.length+' signal match'+(hits.length>1?'es':'')+' — Enter in list to jump','search'); }
    else toast('No signal matches','warn');
    return;
  }
  // boolean temporal expression
  const names=[];
  (function collect(toks){ for(const tk of toks){ if(/^[A-Za-z_][\w.]*$/.test(tk)) names.push(tk); } })(q.split(/\s*&&\s*|\s*\|\|\s*|\s*(!)\s*/).filter(x=>x&&x!=='!'));
  const sigIdx=names.map(n=>SIGS.findIndex(s=>s.n===n||s.n.endsWith('.'+n)));
  if(sigIdx.some(i=>i<0)){ toast('Unknown signal in expression: '+names[sigIdx.findIndex(i=>i<0)],'bad'); return; }
  // sample times: all events of involved signals + dense grid if small
  const tset=new Set();
  for(const si of sigIdx){ for(const t of STEPS[si].t) tset.add(t); }
  if(END<=20000) for(let t=0;t<=END;t++) tset.add(t);
  const times=[...tset].sort((a,b)=>a-b);
  const hits=[];
  for(const t of times){ if(evalExpr(q,t)) hits.push(t); }
  S.results=hits.map(t=>({kind:'time',t,label:'t='+fmtT(t)+' ns'}));
  if(hits.length){ showResults(S.results,'Temporal hits ('+hits.length+')');
    S.resIdx=0; jumpResult(0);
    toast(hits.length+' matching times — [ ] to walk','search');
    toggleDock('#resultsdock',true);
  } else toast('No temporal matches','warn');
}
function addRowSilent(si){ S.rows.push({type:'sig',si,radix:null,color:null,expanded:false,analog:false}); }
function evalExpr(q,t){
  const toks=q.replace(/&&/g,' && ').replace(/\|\|/g,' || ').replace(/!/g,' ! ').split(/\s+/).filter(x=>x);
  const seq=[]; let neg=0;
  for(const tk of toks){
    if(tk==='&&'){ seq.push('a'); neg=0; }
    else if(tk==='||'){ seq.push('o'); neg=0; }
    else if(tk==='!'){ neg++; }
    else{
      const idx=SIGS.findIndex(s=>s.n===tk||s.n.endsWith('.'+tk));
      if(idx<0) return false;
      let v=valAt(idx,t); v=(v==='1')||(typeof v==='number'&&v!==0);
      if(neg%2===1) v=!v; neg=0; seq.push(['v',v]);
    }
  }
  if(!seq.length||seq[0][0]!=='v') return false;
  let acc=seq[0][1], i=1;
  while(i<seq.length){
    if(seq[i]!=='a'&&seq[i]!=='o') return false;
    const nx=seq[i+1]; if(!nx||nx[0]!=='v') return false;
    acc=(seq[i]==='a')? (acc&&nx[1]): (acc||nx[1]); i+=2;
  }
  return acc;
}
function showResults(res,title){
  $('#resultsdock').classList.add('open');
  $('#restitle').textContent=title;
  const list=$('#reslist'); list.innerHTML='';
  res.slice(0,800).forEach((r,i)=>{
    const d=document.createElement('div'); d.className='resitem';
    if(r.kind==='sig'){
      d.innerHTML='<span class="rt">'+esc(shortName(r.si))+'</span><span class="rd">'+esc(SIGS[r.si].n)+'</span>';
      d.onclick=()=>{ const row=findRow(r.si)||addRow(r.si); S.sel.clear(); S.sel.add(row); scrollIntoView(row);
        const st=STEPS[r.si]; if(st.n){ S.curA=st.t[0]; }
        draw(); sbUpdate(); updProps(); };
    } else {
      d.innerHTML='<span class="rt">'+r.label+'</span><span class="rd">'+(r.desc||'')+'</span>';
      d.onclick=()=>{ S.curA=r.t; if(r.t<S.t0+S.tspan*0.05||r.t>S.t0+S.tspan*0.95) gotoT(r.t,true); draw(); sbUpdate(); };
    }
    list.appendChild(d);
  });
}
function jumpResult(d){
  if(!S.results.length){ runSearch(); return; }
  S.resIdx=(S.resIdx+d+S.results.length)%S.results.length;
  const r=S.results[S.resIdx];
  if(r.kind==='sig'){ const row=findRow(r.si)||addRow(r.si); S.sel.clear(); S.sel.add(row); scrollIntoView(row); draw(); updProps(); }
  else { S.curA=r.t; if(r.t<S.t0+S.tspan*0.05||r.t>S.t0+S.tspan*0.95) gotoT(r.t,true); draw(); sbUpdate(); }
  const el=$('#reslist').children[S.resIdx];
  if(el){ el.scrollIntoView({block:'nearest'}); el.style.background='var(--sel)';
    setTimeout(()=>el.style.background='',400); }
  toast('Result '+(S.resIdx+1)+'/'+S.results.length,'search');
}

/* ================= exports ================= */
function dl(name,data,mime){
  const blob=new Blob([data],{type:mime||'application/octet-stream'});
  const a=document.createElement('a'); a.href=URL.createObjectURL(blob); a.download=name;
  document.body.appendChild(a); a.click(); a.remove();
  setTimeout(()=>URL.revokeObjectURL(a.href),5000);
  toast('Exported '+name,'download','ok'); logC('exported '+name);
}
function stamp(){ return new Date().toISOString().replace(/[:.]/g,'-').slice(0,19); }
function exportPNG(allRows){
  const saveY=S.scrollY, saveH=H;
  if(allRows){ const ch=contentH()+AX;
    const w2=W, h2=Math.min(ch, 16000);
    cv.width=w2*dpr; cv.height=h2*dpr; cv.style.height=h2+'px';
    ctx.setTransform(dpr,0,0,dpr,0,0); H=h2; S.scrollY=0; buildLayout(); draw();
  }
  const url=cv.toDataURL('image/png');
  const a=document.createElement('a'); a.href=url; a.download='vikas-studio-'+stamp()+'.png';
  document.body.appendChild(a); a.click(); a.remove();
  if(allRows){ H=saveH; S.scrollY=saveY; cv.width=W*dpr; cv.height=H*dpr; cv.style.height=H+'px';
    ctx.setTransform(dpr,0,0,dpr,0,0); buildLayout(); draw(); }
  toast('PNG exported','download','ok');
}
function exportSVG(){
  const rows=flatRows().filter(e=>e.r.type==='sig');
  const rh=S.rowH+2, pad=4, nameW=200, valW=100;
  const w=W, h=AX+rows.length*rh+20;
  let s='<svg xmlns="http://www.w3.org/2000/svg" width="'+w+'" height="'+h+'" font-family="monospace">\n';
  s+='<rect width="'+w+'" height="'+h+'" fill="'+COLORS.bg+'"/>\n';
  s+='<text x="6" y="16" fill="'+COLORS.accent+'" font-size="12" font-weight="bold">VIKAS STUDIO — '+esc(S.fileName)+' — t=['+fmtT(S.t0)+' … '+fmtT(S.t0+S.tspan)+'] ns</text>\n';
  const x0=nameW+valW, x1=w-14;
  const tx=t=>x0+(t-S.t0)/S.tspan*(x1-x0);
  const step=niceStep(S.tspan,10);
  for(let t=Math.max(0,Math.floor(S.t0/step)*step); t<=S.t0+S.tspan; t+=step){
    s+='<line x1="'+tx(t)+'" y1="'+AX+'" x2="'+tx(t)+'" y2="'+h+'" stroke="'+COLORS.edge+'" stroke-width="1"/>\n';
    s+='<text x="'+(tx(t)+2)+'" y="'+(AX-6)+'" fill="'+COLORS.dim+'" font-size="9">'+fmtT(t)+'</text>\n';
  }
  rows.forEach((e,i)=>{
    const y=AX+i*rh; const sig=SIGS[e.r.si];
    s+='<text x="6" y="'+(y+rh/2+3)+'" fill="'+COLORS.txt+'" font-size="10">'+esc(sig.n)+'</text>\n';
    const vt=S.curA!==null? S.curA:S.t0;
    s+='<text x="'+(nameW+6)+'" y="'+(y+rh/2+3)+'" fill="'+COLORS.dim+'" font-size="10">'+esc(fmtRadix(valAt(e.r.si,vt),sig.w,e.r.radix||S.defRadix))+'</text>\n';
    const st=STEPS[e.r.si];
    if(sig.w===1&&st.n){
      const y1=y+6, y0=y+rh-4;
      let path='M '+x0+' '+y1;
      for(let k=0;k<st.n;k++){ if(st.t[k]<S.t0||st.t[k]>S.t0+S.tspan) continue;
        const v=st.v[k]; const ny=v==='1'? y1: y0;
        path+=' L '+tx(st.t[k]).toFixed(1)+' '+y1+' L '+tx(st.t[k]).toFixed(1)+' '+ny; }
      path+=' L '+x1+' '+(st.n? (st.v[st.n-1]==='1'?y1:y0):y1);
      s+='<path d="'+path+'" stroke="'+kindColor(sig.k)+'" fill="none" stroke-width="1.4"/>\n';
    } else if(st.n){
      const yt=y+5, yb=y+rh-5, ym=(yt+yb)/2;
      let first=true, lastX=x0;
      for(let k=0;k<st.n;k++){ if(st.t[k]<S.t0) continue; if(st.t[k]>S.t0+S.tspan) break;
        const xa=Math.max(x0,tx(st.t[k]));
        s+='<path d="M '+lastX+' '+yt+' L '+(xa-4)+' '+yt+' L '+xa+' '+ym+' L '+(xa-4)+' '+yb+' L '+lastX+' '+yb+' L '+(lastX+4)+' '+ym+' Z" stroke="'+kindColor(sig.k)+'" fill="none" stroke-width="1.2"/>\n';
        lastX=xa; first=false; }
      s+='<path d="M '+lastX+' '+yt+' L '+(x1-4)+' '+yt+' L '+x1+' '+ym+' L '+(x1-4)+' '+yb+' L '+lastX+' '+yb+' L '+(lastX+4)+' '+ym+' Z" stroke="'+kindColor(sig.k)+'" fill="none" stroke-width="1.2"/>\n';
      if(S.curA!==null) { const vt2=S.curA;
        s+='<text x="'+((x0+12))+'" y="'+(ym+3)+'" fill="'+COLORS.txt+'" font-size="9"></text>\n'; }
    }
  });
  s+='</svg>';
  dl('vikas-studio-'+stamp()+'.svg',s,'image/svg+xml');
}
function exportCSV(){
  let out='signal,width,radix,time_ns,value\n';
  for(const {r} of flatRows()){ if(r.type!=='sig') continue;
    const sig=SIGS[r.si], st=STEPS[r.si], rad=r.radix||S.defRadix;
    for(let i=0;i<st.n;i++) out+=sig.n+','+sig.w+','+rad+','+st.t[i]+','+fmtRadix(st.v[i],sig.w,rad)+'\n';
  }
  dl('vikas-transitions-'+stamp()+'.csv',out,'text/csv');
}
function exportJSON(){
  const obj={tool:'VIKAS Studio',version:'1.1.0',end:END,signals:SIGS,changes:{count:NEV,encoding:'delta-varint-base64',blob:CHG}};
  dl('vikas-dump-'+stamp()+'.json',JSON.stringify(obj,null,1),'application/json');
}
function exportWavedrom(){
  const from=S.curA!==null? Math.max(0,S.curA-S.tspan/4): S.t0, to=S.curA!==null? Math.min(END,S.curA+S.tspan*3/4): S.t0+S.tspan;
  const arr=[];
  for(const {r} of flatRows()){ if(r.type!=='sig') continue;
    const sig=SIGS[r.si], st=STEPS[r.si];
    const win=[]; for(let i=0;i<st.n;i++){ if(st.t[i]>=from&&st.t[i]<=to) win.push([st.t[i],st.v[i]]); }
    if(!win.length){ arr.push({name:sig.n,wave:'x'}); continue; }
    if(sig.w===1){ let wave='',last='';
      const times=[...new Set([...win.map(x=>x[0])])];
      for(const [t,v] of win){ const c=v==='1'?'1':'0'; wave+=(c===last)?'.':c; last=c; }
      arr.push({name:sig.n,wave:wave});
    } else { let data=[],wave='';
      let last=null;
      for(const [t,v] of win){ if(v!==last){ data.push(fmtRadix(v,sig.w,'hex')); wave+='='; last=v; } else wave+='.'; }
      arr.push({name:sig.n,wave:wave,data:data});
    }
  }
  dl('vikas-wavedrom-'+stamp()+'.json',JSON.stringify({signal:arr,head:{text:'VIKAS Studio window ['+fmtT(from)+' … '+fmtT(to)+'] ns'}},null,1),'application/json');
}
function saveLayout(){
  const lay={tool:'VIKAS Studio',kind:'vikas-studio-layout',theme:S.theme,t0:S.t0,tspan:S.tspan,curA:S.curA,curB:S.curB,defRadix:S.defRadix,
    rows:serializeRows(S.rows)};
  dl('vikas-layout-'+stamp()+'.json',JSON.stringify(lay,null,1),'application/json');
}
function serializeRows(rs){ return rs.map(r=> r.type==='group'? {type:'group',name:r.name,open:r.open,children:serializeRows(r.children)}
  : {type:'sig',name:SIGS[r.si].n,radix:r.radix,color:r.color,expanded:r.expanded,analog:r.analog}); }
/* ================= file open ================= */
function openDump(){
  const inp=document.createElement('input'); inp.type='file';
  inp.accept='.vwf,.vcd,.json,.vwt,.vwe,.vwp,.vwi,.vwm';
  inp.onchange=()=>{ const f=inp.files[0]; if(!f) return;
    const name=f.name.toLowerCase();
    if(name.endsWith('.json')) f.text().then(t=>{ try{ loadLayout(JSON.parse(t)); toast('Layout applied','check','ok'); }catch(e){ toast('Bad layout: '+e.message,'bad'); } });
    else if(name.endsWith('.vcd')) f.text().then(t=>{ try{ loadVCD(t,f.name); }catch(e){ toast('VCD parse error: '+e.message,'bad'); } });
    else if(name.endsWith('.vwf')) f.arrayBuffer().then(b=>{ try{ loadVWF(b,f.name); }catch(e){ toast('VWF parse error: '+e.message,'bad'); } });
    else toast('Unsupported: only .vwf .vcd .json layout','warn');
  };
  inp.click();
}
function loadVWF(buf,name){
  const u=new Uint8Array(buf); let p=0;
  const magic=String.fromCharCode(...u.subarray(0,8));
  if(magic!=='VIKASVWF') throw new Error('bad magic: '+magic);
  p=8; const crc=rdU32();
  let t=0;
  const nSigs=rdU32();
  const sigs=[];
  for(let i=0;i<nSigs;i++){ const w=rdU32(); const nl=rdU32(); const nm=new TextDecoder().decode(u.subarray(p,p+nl)); p+=nl; sigs.push({n:nm,w}); }
  const nCh=rdU64(); const tA=sigs.map(()=>[]), vA=sigs.map(()=>[]);
  for(let i=0;i<nCh;i++){ const tt=rdU64(); const si=rdU32(); const w=rdU32(); const nb=(w+3)>>2;
    const base=p; p+=nb;
    let v;
    if(w===1) v=ST_CH[u[base]&3];
    else{ let allDef=true, num=0, str='';
      for(let b=0;b<w;b++){ const d=(u[base+(b>>2)]>>((b&3)*2))&3;
        if(d>1) allDef=false; str+=ST_CH[d]; if(allDef&&b<32) num=num*2+d; }
      v=(allDef&&w<=32)? num: str;
    }
    tA[si].push(tt); vA[si].push(v); t=Math.max(t,tt);
  }
  function rdU32(){ const v=u[p]|(u[p+1]<<8)|(u[p+2]<<16)|(u[p+3]<<24); p+=4; return v>>>0; }
  function rdU64(){ let v=0n; for(let i=7;i>=0;i--){ v=(v<<8n)|BigInt(u[p+i]); } p+=8; return Number(v); }
  installData(sigs,tA,vA,Math.max(t,1),name);
}
function loadVCD(text,name){
  const sigs=[], id2i={}, scopeStack=[]; let id=0;
  const tA=[], vA=[]; let t=0;
  for(const raw of text.split('\n')){
    const line=raw.trim(); if(!line) continue;
    if(line.startsWith('$var')){ const tk=line.split(/\s+/);
      const w=parseInt(tk[2])||1; const vid=tk[3]; let nm=tk[4];
      sigs.push({n:scopeStack.length? scopeStack.join('.')+'.'+nm : nm, w});
      id2i[vid]=id++; tA.push([]); vA.push([]);
    } else if(line.startsWith('$scope')){ const tk=line.split(/\s+/); scopeStack.push(tk[2]||''); }
    else if(line.startsWith('$upscope')){ scopeStack.pop(); }
    else if(line[0]==='#'){ t=parseInt(line.slice(1))||0; }
    else if(line[0]==='b'||line[0]==='B'){ const tk=line.split(/\s+/); const si=id2i[tk[1]];
      if(si!==undefined){ let str=''; for(const c of tk[0].slice(1)){ if(c==='0'||c==='1') str+=c; else if(c==='x'||c==='X') str+='x'; else if(c==='z'||c==='Z') str+='z'; }
        const allDef=!/[xz]/.test(str); let num=0;
        if(allDef){ for(let i=0;i<str.length;i++) num=num*2+(str[i]==='1'?1:0); }
        tA[si].push(t); vA[si].push((allDef&&sigs[si].w<=32&&str.length<=32)? num: str);
      } }
    else if('01xzXZ'.includes(line[0])&&line.length>=2){ const si=id2i[line.slice(1)];
      if(si!==undefined){ tA[si].push(t); vA[si].push(line[0].toLowerCase()); } }
  }
  // strip trailing empty signals
  const keep=[];
  sigs.forEach((s,i)=>{ if(tA[i].length) keep.push(i); });
  const S2=keep.map(i=>sigs[i]), T2=keep.map(i=>tA[i]), V2=keep.map(i=>vA[i]);
  installData(S2,T2,V2,t,name);
}
function installData(sigs,tA,vA,end,name){
  // rebuild global state
  SIGS.length=0; SIGS.push(...sigs);
  STEPS=sigs.map((s,i)=>{
    const t=new Float64Array(tA[i]);
    let last=Symbol(), uniq=0;
    for(let k=0;k<vA[i].length;k++){ if(vA[i][k]!==last){ last=vA[i][k]; uniq++; } }
    return {t, v:vA[i], n:vA[i].length, glitch:[], uniq, minT:t.length?t[0]:0, maxT:t.length?t[t.length-1]:0};
  });
  END=end; NEV=STEPS.reduce((a,st)=>a+st.n,0);
  S.rows=[]; S.t0=0; S.tspan=Math.max(end,1); S.curA=S.curB=null; S.scrollY=0;
  S.fileName=name;
  buildHist(); buildTree(); addAllRows();
  const cnt=NEV;
  $('#hdrbuild').textContent=name+' · '+fmtInt(cnt)+' events';
  document.title='VIKAS Studio — '+name;
  fit(); updProps(); refreshColors();
  toast('Loaded '+name+' · '+sigs.length+' signals · '+fmtInt(cnt)+' events','check','ok');
  logC('opened '+name+': '+sigs.length+' signals, '+fmtInt(cnt)+' events','c-ok');
}
function loadLayout(lay){
  if(lay.kind!=='vikas-studio-layout') throw new Error('not a studio layout');
  setTheme(lay.theme||'dark');
  S.t0=lay.t0||0; S.tspan=lay.tspan||END; S.curA=lay.curA??null; S.curB=lay.curB??null;
  if(lay.defRadix){ S.defRadix=lay.defRadix; $('#radixsel').value=S.defRadix; }
  S.rows=restoreRows(lay.rows||[]);
  syncTree(); layout(); draw(); drawMM(); updProps();
}
function restoreRows(rs){ const out=[];
  for(const r of rs||[]){ if(r.type==='group') out.push({type:'group',name:r.name,open:r.open!==false,children:restoreRows(r.children)});
    else{ const i=SIGS.findIndex(s=>s.n===r.name); if(i>=0) out.push({type:'sig',si:i,radix:r.radix,color:r.color,expanded:!!r.expanded,analog:!!r.analog}); } }
  return out; }
function reloadSelf(){ location.reload(); }

/* ================= command palette ================= */
const PALCMDS=[
 ['View','fit','Fit timeline','F','fit()'],
 ['View','zin','Zoom in','+','zoom(0.6)'],
 ['View','zout','Zoom out','-','zoom(1.6)'],
 ['View','region','Toggle marquee zoom','Z','toggleRegion()'],
 ['View','moon','Next theme','T','cycleTheme()'],
 ['View','panelL','Toggle signal tree','Ctrl+1','toggleDock("#treedock")'],
 ['View','panelR','Toggle properties','Ctrl+2','toggleDock("#propdock")'],
 ['View','terminal','Toggle console','`','toggleConsole()'],
 ['File','open','Open dump (.vwf .vcd .json)','Ctrl+O','openDump()'],
 ['File','save','Save view layout','Ctrl+S','saveLayout()'],
 ['File','download','Export PNG (view)','Ctrl+P','exportPNG(false)'],
 ['File','download','Export PNG (all rows)','','exportPNG(true)'],
 ['File','file','Export SVG','','exportSVG()'],
 ['File','file','Export CSV transitions','','exportCSV()'],
 ['File','file','Export JSON steps','','exportJSON()'],
 ['File','file','Export WaveDrom JSON','','exportWavedrom()'],
 ['Signals','check','Add all signals','','addAllRows()'],
 ['Signals','group','Group selection','G','groupSel()'],
 ['Signals','ungroup','Ungroup selection','U','ungroupSel()'],
 ['Signals','expand','Expand bus','E','expandSel(true)'],
 ['Signals','collapse','Collapse bus','Shift+E','expandSel(false)'],
 ['Signals','analog','Toggle analog trace','A','toggleAnalog()'],
 ['Signals','radar','Radix → hex','h','setRadixAll("hex")'],
 ['Signals','radar','Radix → bin','b','setRadixAll("bin")'],
 ['Signals','radar','Radix → dec','','setRadixAll("dec")'],
 ['Signals','radar','Radix → signed','','setRadixAll("signed")'],
 ['Signals','radar','Radix → ascii','','setRadixAll("ascii")'],
 ['Analysis','search','Focus search','Ctrl+F','focusSearch()'],
 ['Analysis','play','Run search','Enter','runSearch()'],
 ['Analysis','next','Next result',']','jumpResult(1)'],
 ['Analysis','prev','Previous result','[','jumpResult(-1)'],
 ['Analysis','cursor','Cursor A at center','c','setCursorA(S.t0+S.tspan/2)'],
 ['Analysis','measure','Measure mode','m','toggleMeasure()'],
 ['Analysis','grid','Open Analysis Hub','','showTab("hub")'],
 ['Analysis','database','Open Design Intel','','showTab("intel")'],
 ['Help','help','Keyboard reference','F1','helpDialog()'],
 ['Help','info','About VIKAS Studio','','aboutDialog()'],
];
let palIdx=0, palFiltered=[];
function openPalette(){ $('#palwrap').style.display='block'; $('#palq').value=''; $('#palq').focus(); palRender(''); }
function closePalette(){ $('#palwrap').style.display='none'; }
$('#palwrap').addEventListener('mousedown',ev=>{ if(ev.target.id==='palwrap') closePalette(); });
$('#palq').addEventListener('input',ev=>palRender(ev.target.value));
$('#palq').addEventListener('keydown',ev=>{
  if(ev.key==='ArrowDown'){ ev.preventDefault(); palIdx=Math.min(palFiltered.length-1,palIdx+1); palPaint(); }
  else if(ev.key==='ArrowUp'){ ev.preventDefault(); palIdx=Math.max(0,palIdx-1); palPaint(); }
  else if(ev.key==='Enter'){ ev.preventDefault(); const it=palFiltered[palIdx]; if(it){ closePalette(); act(it[4]); } }
  else if(ev.key==='Escape'){ closePalette(); }
});
function palRender(q){
  const ql=q.toLowerCase();
  palFiltered=PALCMDS.filter(c=> !ql || (c[1]+' '+c[3]).toLowerCase().includes(ql) || c[4].toLowerCase().includes(ql));
  palIdx=0; palPaint();
}
function palPaint(){
  const list=$('#pallist'); list.innerHTML='';
  let lastGroup=null;
  palFiltered.forEach((c,i)=>{
    if(c[0]!==lastGroup){ const g=document.createElement('div'); g.className='palgroup'; g.textContent=c[0]; list.appendChild(g); lastGroup=c[0]; }
    const d=document.createElement('div'); d.className='palitem'+(i===palIdx?' on':'');
    d.innerHTML=svgIcon(c[1])+'<span>'+c[2]+'</span>'+(c[3]? '<span class="psc">'+c[3]+'</span>':'');
    d.onclick=()=>{ closePalette(); act(c[4]); };
    d.onmouseenter=()=>{ palIdx=i; palPaint(); };
    list.appendChild(d);
  });
  if(!palFiltered.length) list.innerHTML='<div class="palgroup" style="padding:14px">no matching command</div>';
}

/* ================= dialogs ================= */
function helpDialog(){
  const rows=[
   ['kgroup','Navigation'],
   ['wheel','zoom at pointer'],['drag wave','pan'],['drag axis / Z-mode drag','marquee zoom'],
   ['F','fit timeline'],['+ / −','zoom in / out'],['← →','pan 10%'],['Shift+← →','prev / next edge'],
   ['Home / End','go to t=0 / end'],['PageUp / PageDown','scroll rows'],
   ['kgroup','Cursors & measure'],
   ['click','cursor A'],['Shift+click / m','cursor B (measure)'],['C','clear cursors'],['c','cursor A at view center'],
   ['kgroup','Rows & signals'],
   ['click name','select row'],['Ctrl+click','multi-select'],['double-click name','expand bus / analog'],
   ['G / U','group / ungroup'],['E / Shift+E','expand / collapse bus'],['A','toggle analog trace'],
   ['Delete / x','remove selected rows'],['Alt+↑ ↓','move row up / down'],
   ['kgroup','Search'],
   ['Ctrl+F','focus search'],['Enter','run search'],['[ / ]','prev / next result'],
   ['kgroup','Radix'],
   ['h / b','hex / bin default radix'],['right-click row','per-row radix + colors'],
   ['kgroup','Panels & theme'],
   ['Ctrl+1 / Ctrl+2','toggle tree / properties'],['`','toggle console'],['T','cycle theme'],
   ['kgroup','Commands'],
   ['Ctrl+K','command palette'],['Ctrl+O','open dump file'],['Ctrl+S','save layout'],['Ctrl+P','export PNG'],['F1','this dialog'],
  ];
  $('#helpbody').innerHTML='<table class="keys">'+rows.map(r=> r[0]==='kgroup'? '<tr><td colspan="2" class="kgroup">'+r[1]+'</td></tr>' : '<tr><td><span class="kbd">'+r[0]+'</span></td><td>'+r[1]+'</td></tr>').join('')+'</table>';
  $('#dlg-help').showModal();
}
function aboutDialog(){
  const cnt=STEPS.reduce((a,s)=>a+s.n,0);
  $('#aboutbody').innerHTML=
   '<div id="aboutlogo">'
   +'<svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" style="color:var(--accent)"><path d="M12 2 21 7 21 15 12 22 3 15 3 7Z"/><path d="M7.5 8.5 12 15.5 16.5 8.5"/></svg>'
   +'<div id="aboutname">VIKAS <b>STUDIO</b></div></div>'
   +'<div id="aboutsub">Post-duopoly Linux-native RTL EDA platform · XEZIM event kernel 100% Rust<br>Professional single-file HTML5 workbench · fully offline · zero dependencies</div>'
   +'<div id="aboutstats"><div><div class="as-v">'+fmtInt(SIGS.length)+'</div><div class="as-k">signals</div></div>'
   +'<div><div class="as-v">'+fmtInt(cnt)+'</div><div class="as-k">events</div></div>'
   +'<div><div class="as-v">'+fmtT(END)+'</div><div class="as-k">ns end</div></div>'
   +'<div><div class="as-v">'+STEPS.filter(s=>s.glitch.length).length+'</div><div class="as-k">glitch sigs</div></div></div>'
   +'<div style="text-align:center;color:var(--faint);font-size:10.5px;margin:10px 0 2px;font-family:var(--mono)">__BUILD__</div>';
  $('#dlg-about').showModal();
}
function exportDialog(){
  $('#exportbody').innerHTML='<table class="keys">'
   +'<tr><td><span class="kbd">PNG</span> current view</td><td><span class="pbtn" onclick="exportPNG(false)">'+svgIcon('download')+'Export view</span></td></tr>'
   +'<tr><td><span class="kbd">PNG</span> all rows</td><td><span class="pbtn" onclick="exportPNG(true)">'+svgIcon('download')+'Export all</span></td></tr>'
   +'<tr><td><span class="kbd">SVG</span> vector view</td><td><span class="pbtn" onclick="exportSVG()">'+svgIcon('file')+'Export SVG</span></td></tr>'
   +'<tr><td><span class="kbd">CSV</span> transitions</td><td><span class="pbtn" onclick="exportCSV()">'+svgIcon('file')+'Export CSV</span></td></tr>'
   +'<tr><td><span class="kbd">JSON</span> full steps</td><td><span class="pbtn" onclick="exportJSON()">'+svgIcon('file')+'Export JSON</span></td></tr>'
   +'<tr><td><span class="kbd">WaveDrom</span> timing</td><td><span class="pbtn" onclick="exportWavedrom()">'+svgIcon('file')+'Export WaveDrom</span></td></tr>'
   +'<tr><td><span class="kbd">Layout</span> view state</td><td><span class="pbtn" onclick="saveLayout()">'+svgIcon('save')+'Save layout</span></td></tr>'
   +'</table>';
  $('#dlg-export').showModal();
}
function bindDialogs(){
  $$('.dlg-x').forEach(x=>x.onclick=()=>x.closest('dialog').close());
}
/* ================= tabs / hub ================= */
function showTab(which){
  $$('.vtab').forEach(t=>t.classList.toggle('on',t.dataset.tab===which));
  $('#treedock').style.display=which==='wave'? 'flex':'none';
  $('#propdock').style.display=(which==='wave'&&innerWidth>1100)? 'flex':'none';
  $('#canvaswrap').parentElement.style.display=which==='wave'? 'flex':'none';
  const hub=$('#hubscroll');
  if(which!=='wave'){ hub.style.display='block'; } else { hub.style.display='none'; }
  if(which==='hub'){ renderHub(); } else if(which==='intel'){ renderIntel(); }
  layout();
}
function renderHub(){
  const cnt=STEPS.reduce((a,s)=>a+s.n,0);
  const toggled=STEPS.filter(s=>s.n>0).length;
  const cov=SIGS.length? toggled/SIGS.length*100:0;
  const VDD=0.8, CFF=12;
  let totE=0; const perSig=[];
  STEPS.forEach((s,i)=>{ const e=0.5*CFF*1e-15*VDD*VDD*s.n*1000; totE+=e; perSig.push([i,e,s.n]); });
  perSig.sort((a,b)=>b[1]-a[1]);
  const xSigs=[];
  SIGS.forEach((s,i)=>{ const v=valAt(i,END); if(typeof v==='string'&&/[xz]/.test(v)) xSigs.push(i); });
  const glit=STEPS.filter(s=>s.glitch.length);
  const v=CTX.verdict, vk=CTX.verdictKind;
  let html='<div id="hubhead"><h2>Analysis Hub</h2><span class="hub-sub">'+esc(CTX.subtitle||'VIKAS Studio report')+'</span>'
   +(v? '<span class="badge-verdict bv-'+(vk||'ok')+'">'+esc(v)+'</span>':'')+'</div>';
  html+='<div id="hubgrid">';
  html+=card('activity','Toggle Coverage','<div class="hc-big"><span class="n">'+cov.toFixed(1)+'</span><span class="u">%</span></div>'
   +'<div class="hbar"><div style="width:'+Math.min(100,cov)+'%"></div></div>'
   +'<div class="hc-note">'+toggled+' / '+SIGS.length+' signals toggled · '+fmtInt(cnt)+' total events</div>');
  html+=card('zap','Switching Power','<div class="hc-big"><span class="n">'+totE.toFixed(1)+'</span><span class="u">fJ est. @ '+VDD+'V</span></div>'
   +'<div class="hc-note">P = Σ ½·C·V²·f from real toggle counts (C=12 fF/gate est.)</div>'
   +perSig.slice(0,5).map(p=>'<div class="hrow"><span class="hn">'+esc(shortName(p[0]))+'</span><span class="hbar"><div style="width:'+Math.min(100,p[1]/(perSig[0][1]||1)*100)+'%"></div></span><span class="hv">'+p[1].toFixed(1)+' fJ</span></div>').join(''));
  html+=card('cpu','X-Propagation Scan',xSigs.length? '<div class="hc-big"><span class="n" style="color:var(--warn)">'+xSigs.length+'</span><span class="u">signals undefined at end</span></div>'
   +xSigs.slice(0,6).map(i=>'<div class="hrow"><span class="hn">'+esc(SIGS[i].n)+'</span><span class="hv">'+fmtRadix(valAt(i,END),SIGS[i].w,'hex')+'</span></div>').join('')
   : '<div class="hc-big"><span class="n" style="color:var(--ok)">0</span><span class="u">undefined</span></div><div class="hc-note">All signals have defined values at end of dump — X-clean design.</div>',
   xSigs.length? 'h-warn':'h-ok');
  html+=card('warn','Glitch Report',glit.length? '<div class="hc-big"><span class="n" style="color:var(--bad)">'+glit.length+'</span><span class="u">signals with same-time multi-toggles</span></div>'
   +glit.slice(0,6).map(s=>'<div class="hrow"><span class="hn">'+esc(shortName(STEPS.indexOf(s)))+'</span><span class="hv">'+s.glitch.length+' events</span></div>').join('')
   : '<div class="hc-big"><span class="n" style="color:var(--ok)">0</span><span class="u">glitches</span></div><div class="hc-note">Zero-delta event collisions across the whole dump.</div>',
   glit.length? 'h-bad':'h-ok');
  const latches=(CTX.kdb&&CTX.kdb.inferred_latches)||[];
  html+=card('shield','Latch Audit',latches.length? '<div class="hc-big"><span class="n" style="color:var(--bad)">'+latches.length+'</span><span class="u">inferred latches</span></div>'
   +latches.slice(0,5).map(l=>'<div class="hrow"><span class="hn">'+esc(l.signal)+' · '+esc(l.file)+':'+l.line+'</span><span class="hv">'+esc(l.always_block_type)+'</span></div>').join('')
   : '<div class="hc-big"><span class="n" style="color:var(--ok)">0</span><span class="u">latches</span></div><div class="hc-note">RTL is latch-clean'+(CTX.kdb? ' (static L1 + live 3-layer verified)':' — generate via `vikas run --gui` for the full 3-layer audit')+'</div>',
   latches.length? 'h-bad':'h-ok');
  if(CTX.notes&&CTX.notes.length) html+=card('info','Session Notes',CTX.notes.map(n=>'<div class="hc-note">'+esc(n)+'</div>').join(''));
  html+='</div>';
  $('#hubscroll').innerHTML=html;
}
function card(ic,title,body,cls){
  return '<div class="hubcard '+(cls||'')+'"><div class="hc-h">'+svgIcon(ic)+'<span class="hc-t">'+title+'</span></div>'+body+'</div>';
}
function renderIntel(){
  const hub=$('#hubscroll');
  const kdb=CTX.kdb;
  if(!kdb){ hub.innerHTML='<div id="hub-sec-h">'+svgIcon('database')+'<h3>Design Intel</h3><div class="sec-line"></div></div>'
   +'<div class="hub-empty">No Open-KDB embedded in this file.<br>Generate with <span class="kbd">vikas run --gui</span> or <span class="kbd">vikas kdb</span> to see drivers · fan-in cones · FSM · CDC here.</div>';
   return; }
  const drv=kdb.drivers||[];
  const latchRows=(kdb.inferred_latches||[]).map(l=>'<tr><td>'+esc(l.signal)+'</td><td class="dim">'+esc(l.file)+':'+l.line+'</td><td><span class="tag warn">'+esc(l.always_block_type)+'</span></td><td class="dim">'+esc(l.enable_condition)+'</td></tr>').join('');
  const cdcRows=(kdb.cdc_paths||[]).slice(0,40).map(c=>'<tr><td>'+esc(c.src)+'</td><td>'+esc(c.dst)+'</td><td class="dim">'+esc(c.src_clk)+' → '+esc(c.dst_clk)+'</td><td>'+(c.synchronizer_depth>1? '<span class="tag ok">depth '+c.synchronizer_depth+'</span>':'<span class="tag bad">unsynchronized</span>')+'</td></tr>').join('');
  const fsmRows=(kdb.fsm_states||[]).slice(0,12).map(f=>'<tr><td>'+esc(f.state_signal)+'</td><td class="dim">'+(f.encodings||[]).map(e=>esc(e[0])).join(' · ')+'</td><td class="dim">'+(f.transitions||[]).length+' arcs</td></tr>').join('');
  const loops=kdb.scc_loops||[];
  hub.innerHTML='<div id="hub-sec-h">'+svgIcon('database')+'<h3>Design Intel — Open-KDB</h3><div class="sec-line"></div>'
   +'<span class="tag">top '+esc(kdb.top||'?')+'</span><span class="tag">'+(kdb.signals||[]).length+' signals</span><span class="tag">'+drv.length+' driver sites</span>'
   +(loops.length? '<span class="tag bad">'+loops.length+' comb loops</span>':'<span class="tag ok">loop-free</span>')+'</div>'
   +'<div class="hub-empty" style="padding:4px;text-align:left">'
   +'<table class="hubtab"><tr><th>signal</th><th>driver site</th><th>type</th><th>rhs</th></tr>'
   +drv.slice(0,150).map(d=>'<tr><td>'+esc(d.signal)+'</td><td class="dim">'+esc(d.file)+':'+d.line+'</td><td><span class="tag">'+esc(d.kind)+'</span></td><td class="dim">'+esc((d.rhs_text||'').slice(0,60))+'</td></tr>').join('')
   +'</table></div>'
   +(latchRows? '<div id="hub-sec-h">'+svgIcon('shield')+'<h3>Inferred Latches</h3><div class="sec-line"></div></div><table class="hubtab">'+latchRows+'</table>':'')
   +(cdcRows? '<div id="hub-sec-h">'+svgIcon('flow')+'<h3>CDC Paths</h3><div class="sec-line"></div></div><table class="hubtab"><tr><th>src</th><th>dst</th><th>clocks</th><th>sync</th></tr>'+cdcRows+'</table>':'')
   +(fsmRows? '<div id="hub-sec-h">'+svgIcon('target')+'<h3>FSM Extraction</h3><div class="sec-line"></div></div><table class="hubtab"><tr><th>state signal</th><th>encodings</th><th>arcs</th></tr>'+fsmRows+'</table>':'')
   +(loops.length? '<div id="hub-sec-h">'+svgIcon('refresh')+'<h3>Combinational Loops (Tarjan SCC)</h3><div class="sec-line"></div></div><table class="hubtab">'+loops.slice(0,20).map(l=>'<tr><td class="dim">'+esc(l.join(' → '))+'</td></tr>').join('')+'</table>':'');
}
/* ================= toolbar wiring ================= */
function bindToolbar(){
  $$('.tbtn[data-act]').forEach(b=>{ b.onclick=()=>{ const a=b.dataset.act;
    switch(a){
      case 'open': openDump(); return;
      case 'save-layout': saveLayout(); return;
      case 'fit': fit(); return;
      case 'zin': zoom(0.6); return;
      case 'zout': zoom(1.6); return;
      case 'region': toggleRegion(); return;
      case 'prev-edge': edgeJump(-1); return;
      case 'next-edge': edgeJump(1); return;
      case 'cursor-a': setCursorA(S.t0+S.tspan/2); return;
      case 'cursor-b': toggleMeasure(); return;
      case 'clear-cursors': clearCursors(); return;
      case 'group': groupSel(); return;
      case 'ungroup': ungroupSel(); return;
      case 'expand': expandSel(true); return;
      case 'collapse': expandSel(false); return;
      case 'analog': toggleAnalog(); return;
      case 'color': { const f=LAYOUT.find(e=>e.kind==='sig'&&S.sel.has(e.r)); if(f) colorMenuFor(f.r); else toast('Select a row first','warn'); return; }
      case 'search-go': runSearch(); return;
      case 'prev-match': jumpResult(-1); return;
      case 'next-match': jumpResult(1); return;
      case 'export': exportDialog(); return;
      case 'console': toggleConsole(); return;
      case 'theme': cycleTheme(); return;
      case 'help': helpDialog(); return;
      case 'palette-open': openPalette(); return;
      case 'add-all': addAllRows(); return;
      case 'add-top': addTopIO(); return;
      case 'clear-view': clearRows(); return;
      case 'close-results': toggleDock('#resultsdock',false); return;
      case 'row-up': moveSel(-1); return;
      case 'row-down': moveSel(1); return;
    } }; });
  $$('[data-ic]').forEach(el=>{ if(!el.innerHTML.trim()) el.innerHTML=svgIcon(el.dataset.ic); });
  $('#q').addEventListener('keydown',ev=>{ if(ev.key==='Enter'){ runSearch(); } });
  $('#searchmode').onclick=cycleSearchMode;
  $('#radixsel').onchange=ev=>setRadixAll(ev.target.value);
  $$('.vtab').forEach(t=>t.onclick=()=>showTab(t.dataset.tab));
  $$('.dt-btn[data-ic=close]').forEach(b=>{ b.onclick=()=>{ if(b.closest('#consoledock')) toggleConsole(); else toggleDock('#resultsdock',false); }; });
}
function addTopIO(){
  S.rows=[];
  SIGS.forEach((s,i)=>{
    const depth=s.n.split('.').length-1;
    const nm=s.n.split('.').pop()||'';
    if(depth<=1||/clk|clock|rst/i.test(nm)||s.n.indexOf('.')<0) S.rows.push({type:'sig',si:i,radix:null,color:null,expanded:false,analog:false});
  });
  syncTree(); layout(); draw(); toast('Added top I/O + clocks ('+S.rows.length+')','check');
}
/* ================= boot ================= */
function boot(){
  decodeBlob(); buildHist();
  try{ const th=localStorage.getItem('vikas-studio-theme'); if(th) S.theme=th; }catch(e){}
  document.documentElement.setAttribute('data-theme',S.theme);
  refreshColors(); makeHatch(); makeZPat();
  buildMenubar(); buildTree(); bindToolbar(); bindDialogs(); updSearchMode(); layout();
  addAllRows();
  const hc=$('#hdrchips');
  let chips='';
  if(CTX.verdict) chips+='<span class="badge-verdict bv-'+(CTX.verdictKind||'ok')+'">'+svgIcon('shield')+esc(CTX.verdict)+'</span>';
  for(const pair of CTX.chips||[]) chips+='<span class="hchip">'+svgIcon(pair[0]||'info')+esc(pair[1])+'</span>';
  hc.innerHTML=chips;
  document.body.classList.add('has-hub');
  if(!$('#hubscroll')){ const d=document.createElement('div'); d.id='hubscroll'; d.style.display='none'; $('#main').appendChild(d); }
  fit(); updProps();
  logC('VIKAS STUDIO ready — '+SIGS.length+' signals · '+fmtInt(NEV)+' events · '+fmtT(END)+' ns','c-ok');
  logC('shortcuts: F1 help · Ctrl+K palette · right-click menus · wheel zoom','c-dim');
  toast('VIKAS Studio ready — '+SIGS.length+' signals loaded','logo','ok');
}
window.addEventListener('resize',()=>{ layout(); draw(); drawMM(); });
boot();
</script>
</body>
</html>
"##;
