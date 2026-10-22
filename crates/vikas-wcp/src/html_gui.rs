//! VIKAS v1 — HTML5 GUI generator (feature 42 + unified GUI).
//! Self-contained single file, zero dependencies, fully offline.
//! Includes: hover-hold tooltip popups, dual cursors + Δt/frequency HUD,
//! boolean temporal search, latch-hold / physics / causality tracks,
//! keyboard shortcuts (F1 help, +/- zoom, arrows pan/edge-jump).

use vikas_wave::{WaveDump};

pub fn generate_html_gui(dump: &WaveDump, title: &str, out_path: &std::path::Path, build_stamp: &str) -> anyhow::Result<()> {
    let end = dump.end_time();
    let sigs: Vec<serde_json::Value> = dump
        .signals
        .iter()
        .map(|s| serde_json::json!({"n": s.name, "w": s.width, "k": s.kind.as_str()}))
        .collect();
    // compact arrays [dt, sigIdx, bitStr]
    let mut chg = String::from("[");
    let mut first = true;
    let mut last_t = 0u64;
    for c in &dump.changes {
        if !first {
            chg.push(',');
        }
        first = false;
        let dt = c.time - last_t;
        last_t = c.time;
        chg.push_str(&format!("[{},{},\"{}\"]", dt, c.sig, c.value.vcd_str()));
    }
    chg.push(']');

    let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<title>VIKAS v1 — {title} · {stamp}</title>
<style>
:root {{
  --bg: #0b1220; --panel: #101a2e; --edge: #1e2a44; --txt: #dbe6f5; --dim: #7a8bb0;
  --clk: #38bdf8; --rtl: #94e2a1; --phys: #f7b955; --caus: #ff6b81; --sto: #b18cff;
  --taint: #41d3a3; --latch: #ff5470; --reset: #f0a35e; --cursor: #ffffff;
}}
* {{ margin:0; padding:0; box-sizing:border-box; }}
body {{ background:var(--bg); color:var(--txt); font:13px/1.45 'Segoe UI',system-ui,sans-serif; display:flex; height:100vh; overflow:hidden; }}
#side {{ width:300px; background:var(--panel); border-right:1px solid var(--edge); display:flex; flex-direction:column; }}
#logo {{ padding:10px 14px; font-weight:700; letter-spacing:1px; border-bottom:1px solid var(--edge); color:#7dd3fc; font-size:14px; display:flex; gap:9px; align-items:center; }}
#logo span {{ display:flex; flex-direction:column; }}
#logo small {{ color:var(--dim); font-weight:400; display:block; letter-spacing:0; }}
#tree {{ flex:1; overflow:auto; padding:8px; }}
#main {{ flex:1; display:flex; flex-direction:column; min-width:0; }}
#toolbar {{ background:var(--panel); border-bottom:1px solid var(--edge); padding:8px 12px; display:flex; gap:10px; align-items:center; flex-wrap:wrap; }}
#toolbar input {{ background:#0a0f1c; color:var(--txt); border:1px solid var(--edge); border-radius:6px; padding:5px 10px; font:12px ui-monospace,monospace; width:340px; }}
#toolbar button {{ background:#12203a; color:var(--txt); border:1px solid var(--edge); border-radius:6px; padding:5px 11px; cursor:pointer; font-size:12px; }}
#toolbar button:hover {{ background:#1b3055; }}
#stat {{ color:var(--dim); font:11px ui-monospace,monospace; margin-left:auto; }}
#wavebox {{ flex:1; overflow:auto; position:relative; }}
canvas {{ display:block; background:var(--bg); }}
.sigrow {{ color:var(--txt); padding:3px 8px; cursor:pointer; font:12px ui-monospace,monospace; border-radius:5px; display:flex; gap:8px; align-items:center; white-space:nowrap; }}
.sigrow:hover {{ background:#16233f; }}
.sigrow.on {{ background:#173257; }}
.dot {{ width:8px; height:8px; border-radius:50%; flex:none; }}
#help {{ position:fixed; inset:0; background:rgba(5,10,20,.75); display:none; align-items:center; justify-content:center; z-index:99; }}
#helpbox {{ background:var(--panel); border:1px solid var(--edge); border-radius:10px; padding:20px 26px; max-width:640px; }}
#helpbox h3 {{ color:#7dd3fc; margin-bottom:10px; }}
#helpbox td {{ padding:3px 10px; color:var(--dim); }}
#helpbox td:first-child {{ color:var(--txt); font-family:ui-monospace,monospace; }}
#tip {{ position:fixed; z-index:50; background:rgba(13,26,48,.96); border:1px solid #2c4a7c; border-radius:8px; padding:8px 11px; font:11px ui-monospace,monospace; color:#dbe6f5; pointer-events:none; display:none; box-shadow:0 6px 24px rgba(0,0,0,.5); max-width:360px; }}
#tip .tn {{ color:#7dd3fc; font-weight:700; margin-bottom:3px; word-break:break-all; }}
#tip .tv {{ color:#94e2a1; }}
#tip .td {{ color:#7a8bb0; }}
.badge {{ font-size:10px; padding:1px 6px; border-radius:4px; background:#233; color:#9cc; margin-left:6px; }}
</style>
</head>
<body>
<div id="side">
  <div id="logo"><svg width="26" height="26" viewBox="0 0 512 512" style="flex:none;border-radius:7px;background:#0b1220" aria-label="VIKAS logo"><polygon points="256,68 420.5,163 420.5,351 256,446 91.5,351 91.5,163" fill="none" stroke="#3b82f6" stroke-width="22" stroke-linejoin="round"/><line x1="150" y1="150" x2="256" y2="360" stroke="#22d3ee" stroke-width="34" stroke-linecap="round"/><line x1="362" y1="150" x2="256" y2="360" stroke="#818cf8" stroke-width="34" stroke-linecap="round"/><circle cx="256" cy="360" r="66" fill="#f5a623" opacity="0.25"/><circle cx="256" cy="360" r="44" fill="#f5a623"/></svg><span>VIKAS&nbsp;v1<small>native HTML5 waveform GUI · XEZIM dump · offline single-file<br/>{stamp} — open directly; this is NOT a localhost page</small></span></div>
  <div id="tree"></div>
</div>
<div id="main">
  <div id="toolbar">
    <button onclick="fitAll()">fit</button>
    <button onclick="zoomIn()">zoom in</button>
    <button onclick="zoomOut()">zoom out</button>
    <button onclick="left()">◀ edge</button>
    <button onclick="right()">edge ▶</button>
    <input id="q" placeholder="search: fifo_full && !rd_en  |  signal name" onkeydown="if(event.key==='Enter')doSearch()"/>
    <button onclick="doSearch()">find</button>
    <button onclick="document.getElementById('help').style.display='flex'">? help</button>
    <span id="stat"></span>
  </div>
  <div id="wavebox"><canvas id="cv"></canvas></div>
</div>
<div id="help" onclick="this.style.display='none'">
  <div id="helpbox">
    <h3>VIKAS v1 GUI — keyboard &amp; mouse</h3>
    <table>
      <tr><td>wheel / drag</td><td>zoom &amp; pan the timeline</td></tr>
      <tr><td>hover-hold</td><td>popup: signal name, live value, t, next-change Δt</td></tr>
      <tr><td>click</td><td>place cursor (shows values in margins)</td></tr>
      <tr><td>Shift-click</td><td>Δt / frequency / cycle-latency HUD</td></tr>
      <tr><td>F1 / ?</td><td>this help dialog</td></tr>
      <tr><td>+ / -</td><td>zoom in / out · arrow keys pan</td></tr>
      <tr><td>edge ◀ ▶</td><td>jump to previous / next transition of selected signal</td></tr>
      <tr><td>search bar</td><td>boolean temporal query, e.g. <code>valid && !ready</code></td></tr>
      <tr><td>tracks</td><td>VIKAS_PHYSICS_* energy · VIKAS_CAUSALITY_* hazards · latch_hold_* stale-state traps</td></tr>
    </table>
  </div>
</div>
<div id="tip"></div>
<script>
const SIGS = {sigs};
const CHG  = {changes};
const END  = {end};
// ---- decode delta-encoded changes into per-signal step lists ----
const steps = SIGS.map(()=>[]);
let t=0;
for (const [dt,si,v] of CHG) {{
  t += dt;
  steps[si].push([t, v]);
}}
let t0=0, tspan=Math.max(END,1), cursor=null, cursor2=null, sel=null;
let visible = SIGS.map((_,i)=>i);
const cv = document.getElementById('cv');
const ctx = cv.getContext('2d');
let W=800,H=600;
const COL1=0.30, ROW_H=26;
function resize() {{
  const box = document.getElementById('wavebox');
  W = box.clientWidth; H = Math.max(box.clientHeight, visible.length*ROW_H + 60);
  const dpr = window.devicePixelRatio||1;
  cv.width = W*dpr; cv.height = H*dpr; cv.style.width=W+'px'; cv.style.height=H+'px';
  ctx.setTransform(dpr,0,0,dpr,0,0);
  draw();
}}
window.addEventListener('resize', resize);
function timeToX(t) {{ return (W*COL1) + (t-t0)/tspan*(W*(1-COL1)-10); }}
function xToTime(x) {{ return t0 + (x-W*COL1)/(W*(1-COL1)-10)*tspan; }}
function sigColor(k) {{ return ({{rtl:'#94e2a1',clock:'#38bdf8',physics:'#f7b955',causality:'#ff6b81',stochastic:'#b18cff',taint:'#41d3a3',latch:'#ff5470',reset:'#f0a35e'}})[k]||'#94e2a1'; }}
function valueAt(si, t) {{
  const s = steps[si]; let lo=0, hi=s.length-1, r=s.length? s[0][1] : 'x';
  while (lo<=hi) {{ const m=(lo+hi)>>1; if (s[m][0]<=t) {{ r=s[m][1]; lo=m+1; }} else hi=m-1; }}
  return r;
}}
function draw() {{
  ctx.clearRect(0,0,W,H);
  // time axis
  ctx.fillStyle='#7a8bb0'; ctx.font='10px ui-monospace';
  const ticks = 10;
  for (let i=0;i<=ticks;i++) {{
    const t = t0 + tspan*i/ticks;
    const x = timeToX(t);
    ctx.fillText(fmtT(t)+'ns', x+2, 12);
    ctx.strokeStyle='#1e2a44'; ctx.beginPath(); ctx.moveTo(x,16); ctx.lineTo(x,H); ctx.stroke();
  }}
  visible.forEach((si, row) => {{
    const s = SIGS[si];
    const y = 30 + row*ROW_H;
    const col = sigColor(s.k);
    ctx.fillStyle = col;
    ctx.font='11px ui-monospace';
    const nm = s.n.length>34 ? s.n.slice(0,33)+'…' : s.n;
    ctx.fillText(nm, 8, y+9);
    if (s.k==='latch') {{ ctx.fillStyle='#ff5470'; ctx.fillText('⚠', 150, y+9); }}
    const x0 = timeToX(Math.max(0,t0)), x1 = timeToX(t0+tspan), midY=y+9;
    if (s.w===1) {{
      ctx.strokeStyle='#3b4a6b'; ctx.beginPath(); ctx.moveTo(x0,y+ROW_H-4); ctx.lineTo(x1,y+ROW_H-4); ctx.stroke();
      ctx.strokeStyle=col; ctx.lineWidth=1.6; ctx.beginPath();
      let px=x0, py=midY, last='x';
      const inRange = steps[si].filter(([t])=> t>=t0-1 && t<=t0+tspan+1);
      if (!inRange.length || inRange[0][0]>t0) {{
        const v = valueAt(si,t0); py = v==='1'? midY-9 : (v==='0'? midY+6 : midY-2);
        ctx.moveTo(px,py);
      }}
      for (const [t,v] of inRange) {{
        const x=timeToX(t);
        const ny = v==='1'? midY-9 : (v==='0'? midY+6 : midY-2);
        ctx.lineTo(x,py); ctx.lineTo(x,ny); px=x; py=ny;
      }}
      ctx.lineTo(x1,py); ctx.stroke(); ctx.lineWidth=1;
    }} else {{
      ctx.fillStyle='#16233f'; ctx.fillRect(x0, midY-9, x1-x0, 17);
      ctx.strokeStyle='#233350'; ctx.strokeRect(x0, midY-9, x1-x0, 17);
      const inRange = steps[si].filter(([t])=> t>=t0 && t<=t0+tspan);
      let items = [];
      if (!inRange.length || inRange[0][0]>t0) items.push([t0, valueAt(si,t0)]);
      for (const [t,v] of inRange) {{ items.push([t,v]); }}
      for (let i2=0;i2<items.length;i2++) {{
        const [t,v] = items[i2];
        const te = i2+1<items.length? items[i2+1][0] : t0+tspan;
        const xa = timeToX(t), xe = timeToX(te);
        if (xe-xa > 34) {{
          ctx.fillStyle='#dbe6f5'; ctx.font='10px ui-monospace';
          ctx.fillText('x'+hex(v).toUpperCase(), xa+4, midY+3);
        }}
        ctx.strokeStyle='#2c3f63'; ctx.beginPath(); ctx.moveTo(xa,midY-9); ctx.lineTo(xa,midY+8); ctx.stroke();
      }}
    }}
  }});
  if (cursor!==null) {{
    const x=timeToX(cursor);
    ctx.strokeStyle='#fff'; ctx.setLineDash([4,3]); ctx.beginPath(); ctx.moveTo(x,16); ctx.lineTo(x,H); ctx.stroke(); ctx.setLineDash([]);
    ctx.fillStyle='#ffffff'; ctx.font='10px ui-monospace';
    visible.forEach((si,row)=>{{
      const v=valueAt(si,cursor);
      ctx.fillText(v.length>1? hex(v):v, W*COL1-64, 30+row*ROW_H+9);
    }});
  }}
  if (cursor2!==null) {{
    const x=timeToX(cursor2);
    ctx.strokeStyle='#41d3a3'; ctx.setLineDash([4,3]); ctx.beginPath(); ctx.moveTo(x,16); ctx.lineTo(x,H); ctx.stroke(); ctx.setLineDash([]);
    if (cursor!==null) {{
      const dt = Math.abs(cursor2-cursor);
      ctx.fillStyle='#41d3a3';
      ctx.fillText('Δt='+fmtT(dt)+'ns  f='+(dt? (1000/dt).toFixed(3):'∞')+'MHz', timeToX(Math.max(cursor,cursor2))+6, 24);
    }}
  }}
  // hover crosshair (faint guide line at mouse position)
  if (hoverX!==null && !drag) {{
    ctx.strokeStyle='rgba(124,180,255,.25)'; ctx.beginPath(); ctx.moveTo(hoverX,16); ctx.lineTo(hoverX,H); ctx.stroke();
    const ht=Math.max(0,Math.min(END,xToTime(hoverX)));
    ctx.fillStyle='#7a8bb0'; ctx.font='10px ui-monospace';
    ctx.fillText(fmtT(Math.round(ht*10)/10)+'ns', hoverX+3, 24);
  }}
  document.getElementById('stat').textContent =
    't=['+Math.round(t0)+', '+Math.round(t0+tspan)+']ns  signals:'+visible.length+'/'+SIGS.length+'  events:'+CHG.length;
}}
function fmtT(t) {{ t=Math.round(t*1000)/1000; return (Math.abs(t-Math.round(t))<0.001? Math.round(t): t)+''; }}
function hex(b) {{
  if (!b || b[0]!=='b') return b;
  const bits=b.slice(1); let out=''; let nib='';
  for (let i=bits.length-1;i>=0;i--) nib=bits[i]+nib;
  let v=0n; for (const ch of bits) v=(v<<1n)+BigInt(ch==='1'?1:0);
  return v.toString(16);
}}
function fitAll() {{ t0=0; tspan=Math.max(END,1); draw(); }}
function zoomIn() {{ tspan=Math.max(tspan/2,1); draw(); }}
function zoomOut() {{ tspan=Math.min(tspan*2, Math.max(END,1)*2); draw(); }}
function left() {{ if (sel===null) return; const s=steps[sel];
  let best=null; for (const [t] of s) if (t< (cursor??t0+tspan)) best=t;
  if (best!==null) {{ cursor=best; draw(); }} }}
function right() {{ if (sel===null) return; const s=steps[sel];
  let best=null; for (const [t] of s) if (t>(cursor??0) && (best===null||t<best)) best=t;
  if (best!==null) {{ cursor=best; draw(); }} }}
function evalCond(name, t) {{
  const idx = SIGS.findIndex(s=>s.n===name||s.n.endsWith('.'+name)||s.n.endsWith(name));
  if (idx<0) return false;
  const v = valueAt(idx,t);
  return v==='1';
}}
function doSearch() {{
  const q = document.getElementById('q').value.trim();
  if (!q) return;
  const m = q.match(/^\s*([A-Za-z0-9_.]+)\s*$/);
  if (m) {{
    // plain signal search: select it
    const idx = SIGS.findIndex(s=>s.n.includes(m[1]));
    if (idx>=0) {{ sel=idx; visible=[idx]; buildTree(); resize(); }}
    return;
  }}
  // boolean temporal search: sample every integer ns AND every event edge (never miss a pulse)
  const tsSet=new Set(); for (let t=0;t<=END;t+=1) tsSet.add(t);
  SIGS.forEach((s,i)=>steps[i].forEach(([t])=>tsSet.add(t)));
  const times=[...tsSet].sort((a,b)=>a-b).filter(t=>evalTokens(q,t));
  if (times.length) {{
    t0=Math.max(0,times[0]-tspan*0.2); cursor=times[0];
    alert('VIKAS find: '+times.length+' matching timestamps; first at t='+times[0]+'ns (jumped)');
    draw();
  }} else alert('VIKAS find: no matches');
}}
function evalTokens(q,t) {{
  // tokenize: names, &&, ||, ! (prefix, stackable: !!A)
  const toks=q.replace(/&&/g,' && ').replace(/\|\|/g,' || ').replace(/!/g,' ! ').split(/\s+/).filter(x=>x);
  const seq=[]; let neg=0;
  for (const tk of toks) {{
    if (tk==='&&') {{ seq.push('a'); neg=0; }}
    else if (tk==='||') {{ seq.push('o'); neg=0; }}
    else if (tk==='!') {{ neg++; }}
    else {{
      let val=evalCond(tk,t);
      if (neg%2===1) val=!val;
      neg=0; seq.push(['v',val]);
    }}
  }}
  if (!seq.length || seq[0][0]!=='v') return false;   // malformed: no leading operand
  let acc=seq[0][1];
  let i=1;
  while (i<seq.length) {{
    if (seq[i]!=='a'&&seq[i]!=='o') return false;      // malformed: operand w/o operator
    const nx=seq[i+1];
    if (!nx || nx[0]!=='v') return false;              // malformed: trailing operator
    acc = seq[i]==='a' ? (acc&&nx[1]) : (acc||nx[1]);
    i+=2;
  }}
  return acc;
}}
cv.addEventListener('wheel', e=>{{
  const f = e.deltaY>0? 1.2: 1/1.2;
  const ct = xToTime(e.offsetX);
  tspan*=f; t0 = ct-(ct-t0)*f;
  t0=Math.max(0,t0); tspan=Math.min(tspan, Math.max(END,1)*2);
  draw(); e.preventDefault();
}});
cv.addEventListener('mousedown', e=>{{ drag={{x:e.offsetX,t:t0}}; hideTip(); }});
let drag=null, hoverX=null, hoverTimer=null;
cv.addEventListener('mousemove', e=>{{
  if (drag) {{ t0=Math.max(0, drag.t-(e.offsetX-drag.x)/ (W*(1-COL1)-10)*tspan); draw(); return; }}
  hoverX=e.offsetX;
  clearTimeout(hoverTimer);
  hoverTimer=setTimeout(()=>showTip(e.offsetX,e.offsetY,e.clientX,e.clientY),230);
  draw();
}});
cv.addEventListener('mouseleave', ()=>{{ clearTimeout(hoverTimer); hoverX=null; hideTip(); draw(); }});
cv.addEventListener('mouseup', e=>{{ drag=null; }});
function showTip(ox,oy,cx,cy) {{
  const row=Math.floor((oy-30)/ROW_H);
  if (row<0||row>=visible.length) return;
  const si=visible[row], s=SIGS[si];
  const t=Math.max(0, Math.min(END, xToTime(ox)));
  const v=valueAt(si,t);
  let nxt=null; for (const [tt] of steps[si]) if (tt>t) {{ nxt=tt; break; }}
  const val = s.w===1? v : '0x'+hex(v).toUpperCase();
  const tip=document.getElementById('tip');
  tip.innerHTML='<div class="tn">'+s.n+'</div>'
    +'<div>value: <span class="tv">'+val+'</span> <span class="td">('+s.w+'b·'+s.k+')</span></div>'
    +'<div class="td">t = '+fmtT(t)+' ns</div>'
    +(nxt!==null? '<div class="td">next change t='+fmtT(nxt)+' ns (+'+fmtT(nxt-t)+'ns)</div>' : '<div class="td">stable until end of dump</div>')
    +'<div class="td">click: cursor · shift-click: Δt HUD · wheel: zoom</div>';
  tip.style.display='block';
  const r=tip.getBoundingClientRect();
  let x=cx+18, y=cy+16;
  if (x+r.width>innerWidth-10) x=cx-r.width-18;
  if (y+r.height>innerHeight-10) y=cy-r.height-16;
  tip.style.left=x+'px'; tip.style.top=y+'px';
}}
function hideTip() {{ const d=document.getElementById('tip'); if (d) d.style.display='none'; }}
cv.addEventListener('click', e=>{{
  if (e.shiftKey) {{ cursor2=xToTime(e.offsetX); draw(); return; }}
  cursor=xToTime(e.offsetX); cursor2=null;
  const row=Math.floor((e.offsetY-30)/ROW_H);
  if (row>=0&&row<visible.length) sel=visible[row];
  draw();
}});
// keyboard: F1/help, +/-, arrows, esc
window.addEventListener('keydown', e=>{{
  if (e.key==='F1') {{ e.preventDefault(); document.getElementById('help').style.display='flex'; }}
  else if (e.key==='?') {{ document.getElementById('help').style.display='flex'; }}
  else if (e.key==='Escape') {{ document.getElementById('help').style.display='none'; hideTip(); }}
  else if (e.key==='+'||e.key==='=') zoomIn();
  else if (e.key==='-') zoomOut();
  else if (e.key==='ArrowLeft') {{ t0=Math.max(0,t0-tspan*0.1); draw(); }}
  else if (e.key==='ArrowRight') {{ t0=t0+tspan*0.1; draw(); }}
  else if (e.key==='ArrowUp') {{ if (sel===null) sel=0; else sel=Math.max(0,sel-1); draw(); }}
  else if (e.key==='ArrowDown') {{ if (sel===null) sel=0; else sel=Math.min(SIGS.length-1,sel+1); draw(); }}
  else if (e.key==='f'||e.key==='F') fitAll();
}});
function buildTree() {{
  const tree=document.getElementById('tree');
  tree.innerHTML='';
  const scopes={{}};
  SIGS.forEach((s,i)=>{{
    const sc=s.n.split('.').slice(0,-1).join('.')||'top';
    (scopes[sc]=scopes[sc]||[]).push(i);
  }});
  for (const [sc, list] of Object.entries(scopes)) {{
    const h=document.createElement('div');
    h.style.cssText='color:#7dd3fc;font-weight:600;padding:6px 8px 2px;';
    h.textContent=sc; tree.appendChild(h);
    for (const i of list) {{
      const s=SIGS[i];
      const d=document.createElement('div');
      d.className='sigrow'+(visible.length===1&&visible[0]===i?' on':'');
      const c=sigColor(s.k);
      d.innerHTML='<span class="dot" style="background:'+c+'"></span>'+s.n.split('.').pop()+'<span style="color:#5a6b8f">'+s.w+'b</span>'+(s.k!=='rtl'?'<span class="badge">'+s.k+'</span>':'');
      d.onclick=()=>{{ sel=i; visible=[i]; buildTree(); resize(); }};
      tree.appendChild(d);
    }}
  }}
}}
buildTree(); resize();
</script>
</body>
</html>
"##,
        title = title,
        stamp = build_stamp,
        sigs = serde_json::to_string(&sigs)?,
        changes = chg,
        end = end,
    );
    std::fs::write(out_path, html)?;
    Ok(())
}
