// ThinkSpace Navigator — v1.107 (MINOR_NO=107)
// © 2026 Robin L. M. Cheung. All Rights Reserved.
// Extracted from the single-file HTML preview into a standalone React/TSX module.
//   • hooks destructure (const {..}=React) -> real import
//   • <script>/CDN scaffolding + ReactDOM mount removed (mount snippet at bottom)
//   • everything between is verbatim source; body was already typescript,react (Babel data-presets)
import React, { useRef, useState, useEffect, useCallback } from "react";

// ThinkSpace — 6DoF semantic concept navigator (mockup)
// Controls: drag empty space = pivot/orbit about a vertical hinge out front.
//   scroll = fall forward (banks toward focused node). grab bauble = haul mass
//   (Alt = pluck one). grab edge = haul by strut. double-click = dive to wiki.
//   E = pysanky: tap semi-auto, hold ≥1s = aimable slingshot (overcharge quivers
//   then glitches then misfires). 4 = journey map. ✶ sky = orientation stars.

const ONTO = {
  entity:{c:"#7ee8fa",label:"Entity"}, process:{c:"#b388ff",label:"Process"},
  abstract:{c:"#ff9ecd",label:"Abstract"}, place:{c:"#7cffc4",label:"Place"},
  artifact:{c:"#ffd479",label:"Artifact"}, field:{c:"#9db4ff",label:"Field"},
};
const REL = {
  is_a:{c:"#7ee8fa",label:"is-a"}, part_of:{c:"#7cffc4",label:"part-of"},
  causes:{c:"#ff9ecd",label:"causes"}, related:{c:"#b388ff",label:"related"}, uses:{c:"#ffd479",label:"uses"},
};
// ── Startup seed pool ──────────────────────────────────────────────
// Real, linked Wikipedia topic clusters (NOT placeholders). Each node
// carries its actual en.wikipedia.org URL so the link-retrieval reader +
// PDF/flipbook fetch have real content to pull. pickSeed() returns one
// random cluster per startup, so the opening constellation varies but is
// always a coherent, genuinely-linked graph.
const W = (slug)=>"https://en.wikipedia.org/wiki/"+slug;
const SEED_POOL = [
  { topic:"Thermodynamics & Information",
    nodes:[
      {id:"entropy",title:"Entropy",onto:"abstract",url:W("Entropy")},
      {id:"thermo",title:"Thermodynamics",onto:"field",url:W("Thermodynamics")},
      {id:"information",title:"Information Theory",onto:"field",url:W("Information_theory")},
      {id:"shannon",title:"Claude Shannon",onto:"entity",url:W("Claude_Shannon")},
      {id:"heat",title:"Heat Death of the Universe",onto:"process",url:W("Heat_death_of_the_universe")},
      {id:"boltzmann",title:"Ludwig Boltzmann",onto:"entity",url:W("Ludwig_Boltzmann")},
      {id:"engine",title:"Heat Engine",onto:"artifact",url:W("Heat_engine")},
      {id:"cosmos",title:"Observable Universe",onto:"place",url:W("Observable_universe")},
      {id:"compression",title:"Data Compression",onto:"process",url:W("Data_compression")},
      {id:"order",title:"Second Law of Thermodynamics",onto:"abstract",url:W("Second_law_of_thermodynamics")},
    ],
    edges:[
      {from:"entropy",to:"thermo",rel:"part_of",conf:0.92},
      {from:"entropy",to:"information",rel:"related",conf:0.81},
      {from:"shannon",to:"information",rel:"causes",conf:0.88},
      {from:"heat",to:"thermo",rel:"is_a",conf:0.74},
      {from:"boltzmann",to:"entropy",rel:"causes",conf:0.69},
      {from:"engine",to:"thermo",rel:"uses",conf:0.66},
      {from:"heat",to:"cosmos",rel:"related",conf:0.58},
      {from:"compression",to:"information",rel:"uses",conf:0.83},
      {from:"order",to:"entropy",rel:"related",conf:0.77},
      {from:"engine",to:"entropy",rel:"uses",conf:0.6},
    ] },
  { topic:"Music Theory & Pedagogy",
    nodes:[
      {id:"harmony",title:"Harmony",onto:"abstract",url:W("Harmony")},
      {id:"counterpoint",title:"Counterpoint",onto:"abstract",url:W("Counterpoint")},
      {id:"bach",title:"Johann Sebastian Bach",onto:"entity",url:W("Johann_Sebastian_Bach")},
      {id:"scale",title:"Scale (music)",onto:"abstract",url:W("Scale_(music)")},
      {id:"solfege",title:"Solfège",onto:"process",url:W("Solf%C3%A8ge")},
      {id:"kodaly",title:"Zoltán Kodály",onto:"entity",url:W("Zolt%C3%A1n_Kod%C3%A1ly")},
      {id:"tonality",title:"Tonality",onto:"abstract",url:W("Tonality")},
      {id:"fugue",title:"Fugue",onto:"artifact",url:W("Fugue")},
      {id:"eartrain",title:"Ear Training",onto:"process",url:W("Ear_training")},
      {id:"interval",title:"Interval (music)",onto:"abstract",url:W("Interval_(music)")},
    ],
    edges:[
      {from:"counterpoint",to:"harmony",rel:"related",conf:0.8},
      {from:"fugue",to:"counterpoint",rel:"is_a",conf:0.86},
      {from:"bach",to:"fugue",rel:"causes",conf:0.78},
      {from:"interval",to:"scale",rel:"part_of",conf:0.82},
      {from:"scale",to:"tonality",rel:"part_of",conf:0.75},
      {from:"harmony",to:"tonality",rel:"related",conf:0.79},
      {from:"solfege",to:"eartrain",rel:"uses",conf:0.84},
      {from:"kodaly",to:"solfege",rel:"causes",conf:0.71},
      {from:"eartrain",to:"interval",rel:"uses",conf:0.77},
      {from:"bach",to:"counterpoint",rel:"uses",conf:0.7},
    ] },
  { topic:"Cosmology",
    nodes:[
      {id:"bigbang",title:"Big Bang",onto:"process",url:W("Big_Bang")},
      {id:"galaxy",title:"Galaxy",onto:"place",url:W("Galaxy")},
      {id:"blackhole",title:"Black Hole",onto:"entity",url:W("Black_hole")},
      {id:"relativity",title:"General Relativity",onto:"field",url:W("General_relativity")},
      {id:"einstein",title:"Albert Einstein",onto:"entity",url:W("Albert_Einstein")},
      {id:"darkmatter",title:"Dark Matter",onto:"abstract",url:W("Dark_matter")},
      {id:"redshift",title:"Redshift",onto:"process",url:W("Redshift")},
      {id:"hubble",title:"Hubble's Law",onto:"abstract",url:W("Hubble%27s_law")},
      {id:"cmb",title:"Cosmic Microwave Background",onto:"place",url:W("Cosmic_microwave_background")},
      {id:"spacetime",title:"Spacetime",onto:"abstract",url:W("Spacetime")},
    ],
    edges:[
      {from:"einstein",to:"relativity",rel:"causes",conf:0.9},
      {from:"relativity",to:"spacetime",rel:"uses",conf:0.85},
      {from:"blackhole",to:"relativity",rel:"related",conf:0.8},
      {from:"bigbang",to:"cmb",rel:"causes",conf:0.83},
      {from:"redshift",to:"hubble",rel:"causes",conf:0.79},
      {from:"hubble",to:"bigbang",rel:"related",conf:0.74},
      {from:"galaxy",to:"darkmatter",rel:"related",conf:0.72},
      {from:"blackhole",to:"galaxy",rel:"part_of",conf:0.68},
      {from:"spacetime",to:"bigbang",rel:"part_of",conf:0.6},
      {from:"cmb",to:"redshift",rel:"uses",conf:0.64},
    ] },
  { topic:"Neuroscience & Cognition",
    nodes:[
      {id:"neuron",title:"Neuron",onto:"entity",url:W("Neuron")},
      {id:"brain",title:"Human Brain",onto:"place",url:W("Human_brain")},
      {id:"memory",title:"Memory",onto:"process",url:W("Memory")},
      {id:"synapse",title:"Synapse",onto:"artifact",url:W("Synapse")},
      {id:"consciousness",title:"Consciousness",onto:"abstract",url:W("Consciousness")},
      {id:"plasticity",title:"Neuroplasticity",onto:"process",url:W("Neuroplasticity")},
      {id:"cortex",title:"Cerebral Cortex",onto:"place",url:W("Cerebral_cortex")},
      {id:"dopamine",title:"Dopamine",onto:"entity",url:W("Dopamine")},
      {id:"learning",title:"Learning",onto:"process",url:W("Learning")},
      {id:"perception",title:"Perception",onto:"abstract",url:W("Perception")},
    ],
    edges:[
      {from:"neuron",to:"brain",rel:"part_of",conf:0.9},
      {from:"synapse",to:"neuron",rel:"part_of",conf:0.88},
      {from:"cortex",to:"brain",rel:"part_of",conf:0.85},
      {from:"plasticity",to:"learning",rel:"causes",conf:0.82},
      {from:"learning",to:"memory",rel:"related",conf:0.8},
      {from:"memory",to:"synapse",rel:"uses",conf:0.76},
      {from:"dopamine",to:"learning",rel:"causes",conf:0.73},
      {from:"perception",to:"cortex",rel:"uses",conf:0.7},
      {from:"consciousness",to:"brain",rel:"related",conf:0.66},
      {from:"plasticity",to:"neuron",rel:"uses",conf:0.69},
    ] },
  { topic:"Computing & AI",
    nodes:[
      {id:"turing",title:"Alan Turing",onto:"entity",url:W("Alan_Turing")},
      {id:"algorithm",title:"Algorithm",onto:"abstract",url:W("Algorithm")},
      {id:"ml",title:"Machine Learning",onto:"field",url:W("Machine_learning")},
      {id:"neuralnet",title:"Artificial Neural Network",onto:"artifact",url:W("Artificial_neural_network")},
      {id:"computation",title:"Theory of Computation",onto:"field",url:W("Theory_of_computation")},
      {id:"complexity",title:"Computational Complexity Theory",onto:"field",url:W("Computational_complexity_theory")},
      {id:"transformer",title:"Transformer (deep learning)",onto:"artifact",url:W("Transformer_(deep_learning_architecture)")},
      {id:"vonneumann",title:"John von Neumann",onto:"entity",url:W("John_von_Neumann")},
      {id:"llm",title:"Large Language Model",onto:"artifact",url:W("Large_language_model")},
      {id:"backprop",title:"Backpropagation",onto:"process",url:W("Backpropagation")},
    ],
    edges:[
      {from:"turing",to:"computation",rel:"causes",conf:0.88},
      {from:"algorithm",to:"computation",rel:"part_of",conf:0.84},
      {from:"complexity",to:"computation",rel:"part_of",conf:0.82},
      {from:"ml",to:"neuralnet",rel:"uses",conf:0.85},
      {from:"neuralnet",to:"backprop",rel:"uses",conf:0.83},
      {from:"transformer",to:"neuralnet",rel:"is_a",conf:0.8},
      {from:"llm",to:"transformer",rel:"uses",conf:0.86},
      {from:"backprop",to:"ml",rel:"causes",conf:0.74},
      {from:"vonneumann",to:"algorithm",rel:"related",conf:0.66},
      {from:"llm",to:"ml",rel:"part_of",conf:0.78},
    ] },
  { topic:"Philosophy & Logic",
    nodes:[
      {id:"logic",title:"Logic",onto:"field",url:W("Logic")},
      {id:"godel",title:"Kurt Gödel",onto:"entity",url:W("Kurt_G%C3%B6del")},
      {id:"incompleteness",title:"Gödel's Incompleteness Theorems",onto:"abstract",url:W("G%C3%B6del%27s_incompleteness_theorems")},
      {id:"epistemology",title:"Epistemology",onto:"field",url:W("Epistemology")},
      {id:"kant",title:"Immanuel Kant",onto:"entity",url:W("Immanuel_Kant")},
      {id:"ethics",title:"Ethics",onto:"field",url:W("Ethics")},
      {id:"metaphysics",title:"Metaphysics",onto:"field",url:W("Metaphysics")},
      {id:"syllogism",title:"Syllogism",onto:"abstract",url:W("Syllogism")},
      {id:"aristotle",title:"Aristotle",onto:"entity",url:W("Aristotle")},
      {id:"truth",title:"Truth",onto:"abstract",url:W("Truth")},
    ],
    edges:[
      {from:"godel",to:"incompleteness",rel:"causes",conf:0.9},
      {from:"incompleteness",to:"logic",rel:"part_of",conf:0.83},
      {from:"aristotle",to:"syllogism",rel:"causes",conf:0.85},
      {from:"syllogism",to:"logic",rel:"part_of",conf:0.82},
      {from:"kant",to:"metaphysics",rel:"related",conf:0.76},
      {from:"kant",to:"epistemology",rel:"related",conf:0.78},
      {from:"epistemology",to:"truth",rel:"uses",conf:0.74},
      {from:"logic",to:"truth",rel:"related",conf:0.72},
      {from:"aristotle",to:"ethics",rel:"causes",conf:0.68},
      {from:"metaphysics",to:"epistemology",rel:"related",conf:0.65},
    ] },
];
// pick one random cluster per page load (module-eval = once per startup)
function pickSeed(){ return SEED_POOL[Math.floor(Math.random()*SEED_POOL.length)]; }
const SEED = pickSeed();
// when this session opened — fallback "created" stamp for legacy/seed nodes that predate provenance
const SESSION_START = Date.now();
// human date+time for the card's citation footer
const fmtStamp = (ts)=>{ if(!ts) return "—"; try{ const d=new Date(ts); return d.toLocaleDateString(undefined,{year:"numeric",month:"short",day:"numeric"})+" · "+d.toLocaleTimeString(undefined,{hour:"2-digit",minute:"2-digit"}); }catch(e){ return "—"; } };
// derive a citation source label from a node's url/source — the host, prettified
const srcLabel = (n)=>{ if(n&&n.source) return n.source; const u=n&&n.url; if(!u) return ""; try{ const h=new URL(u).hostname.replace(/^www\./,""); return h==="en.wikipedia.org"?"Wikipedia":h; }catch(e){ return u.startsWith("file:")?"local file":""; } };

const sub=(a,b)=>[a[0]-b[0],a[1]-b[1],a[2]-b[2]];
const add=(a,b)=>[a[0]+b[0],a[1]+b[1],a[2]+b[2]];
const scale=(a,s)=>[a[0]*s,a[1]*s,a[2]*s];
const len=(a)=>Math.hypot(a[0],a[1],a[2])||1e-6;
const dist=(a,b)=>len(sub(a,b));
const segDist=(px,py,x1,y1,x2,y2)=>{const dx=x2-x1,dy=y2-y1,l2=dx*dx+dy*dy||1e-6;let t=((px-x1)*dx+(py-y1)*dy)/l2;t=Math.max(0,Math.min(1,t));return Math.hypot(px-(x1+t*dx),py-(y1+t*dy));};
function inferEdges(id,g){return new Promise(r=>setTimeout(()=>r(g.edges.filter(e=>e.from===id||e.to===id)),380));}

// ── ingest helpers: parse .md → node, embed, build similarity graph, project to 3D ──
function parseMd(name,text){
  // strip YAML frontmatter, pull title + tags if present
  let title=name.replace(/\.md$/i,""), tags=[], body=text;
  const fm=text.match(/^---\n([\s\S]*?)\n---\n?/);
  if(fm){ body=text.slice(fm[0].length);
    const tm=fm[1].match(/title:\s*"?([^"\n]+)"?/); if(tm) title=tm[1].trim();
  }
  // text used for embedding = title + body, trimmed
  return {title, tags, text:(title+"\n"+body).slice(0,8000), raw:body.slice(0,1200)};
}
// ── document ingestion: PDF / HTML / plain text → full text, then passage chunks ──
// reuses the same pdf.js (3.11.174) the flipbook lazy-loads, so no second copy.
let _pdfjsP=null;
async function ensurePdfjs(){
  if(typeof window==="undefined") return null;
  if(window.pdfjsLib) return window.pdfjsLib;
  if(!_pdfjsP) _pdfjsP=new Promise((res,rej)=>{
    const s=document.createElement("script");
    s.src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js";
    s.onload=()=>{ try{ window.pdfjsLib.GlobalWorkerOptions.workerSrc="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js"; }catch(e){} res(window.pdfjsLib); };
    s.onerror=()=>rej(new Error("pdf.js failed to load (offline?)")); document.head.appendChild(s);
  });
  return _pdfjsP;
}
async function pdfToText(file){
  const lib=await ensurePdfjs(); if(!lib) throw new Error("pdf.js unavailable");
  const buf=await file.arrayBuffer();
  const pdf=await lib.getDocument({data:buf}).promise;
  const out=[];
  for(let p=1;p<=pdf.numPages && p<=120;p++){   // 120-page safety cap for huge papers
    const page=await pdf.getPage(p);
    const tc=await page.getTextContent();
    out.push(tc.items.map(it=>it.str).join(" "));
  }
  return out.join("\n\n");
}
function htmlToText(t){
  try{ const d=new DOMParser().parseFromString(t,"text/html");
    d.querySelectorAll("script,style,nav,header,footer,aside,noscript").forEach(e=>e.remove());
    return (d.body&&(d.body.innerText||d.body.textContent))||t;
  }catch(e){ return String(t).replace(/<[^>]+>/g," "); }
}
async function extractFileText(f){
  const n=(f.name||"").toLowerCase();
  if(/\.pdf$/.test(n)) return await pdfToText(f);
  const t=await f.text();
  if(/\.html?$/.test(n)) return htmlToText(t);
  return t;   // md, txt, csv, json, tex, etc.
}
// split a long document into ~900-char passages on paragraph boundaries (RAG chunking)
function chunkDoc(text,maxChunks){
  const cap=maxChunks||12;
  const paras=String(text).split(/\n\s*\n+/).map(s=>s.replace(/[ \t]+/g," ").trim()).filter(p=>p.length>40);
  const chunks=[]; let cur="";
  for(const p of paras){
    if((cur+" "+p).length>900){ if(cur)chunks.push(cur.trim()); cur=p; }
    else cur=cur?cur+" "+p:p;
    if(chunks.length>=cap) break;
  }
  if(cur.trim()&&chunks.length<cap) chunks.push(cur.trim());
  return chunks;
}
const conceptTitle=(s)=>{ const c=String(s).replace(/^#+\s*/,"").trim(); const w=c.split(/\s+/).slice(0,8).join(" "); return c.length>w.length?w+"…":w; };
// Embedding goes through embedUnified() only (Ollama / OpenAI-compatible).
// There is deliberately NO hash-to-vector fallback anywhere: a missing host
// fails loudly rather than fabricating meaningless vectors.
const cosine=(a,b)=>{let d=0,na=0,nb=0;for(let i=0;i<a.length;i++){d+=a[i]*b[i];na+=a[i]*a[i];nb+=b[i]*b[i];}return d/((Math.sqrt(na)*Math.sqrt(nb))||1);};
const hashStr=(s)=>{let h=0;for(let i=0;i<s.length;i++){h=((h<<5)-h+s.charCodeAt(i))|0;}return h;};
// project N-d embeddings → 3D via top-3 principal-ish axes (cheap power-iteration-free:
// use 3 fixed random orthogonal-ish projections, good enough for a layout seed)
function projectTo3D(vecs){
  const D=vecs[0].length;
  const axes=[0,1,2].map(s=>{const a=new Array(D).fill(0).map((_,i)=>Math.sin(i*1.7+s*2.1)+Math.cos(i*0.9+s*3.3));const n=Math.hypot(...a);return a.map(x=>x/n);});
  return vecs.map(v=>axes.map(ax=>{let d=0;for(let i=0;i<D;i++)d+=v[i]*ax[i];return d*260;}));
}

// ── connection helpers: Ollama + OpenAI-compatible ────────────────────────
// provider: "ollama" | "openai". normBase strips whatever the user pasted
// (a full /v1/chat/completions, a trailing /v1, query/hash, slashes) back to
// the ROOT, then each call re-appends the right endpoint. So OpenAI-compatible
// gateways — OpenAI, OpenRouter, z.ai/GLM, vLLM, LM Studio — all work from one
// field whether you paste "…/api/v1", "…/v1/chat/completions" or "…/api".
function normBase(host){
  if(!host) return "";
  let h=String(host).trim(); if(!h) return "";
  h=/^https?:\/\//.test(h)?h:("http://"+h);
  h=h.replace(/[?#].*$/,"");                 // drop query / hash
  h=h.replace(/\/+$/,"");                     // trailing slashes
  // strip a pasted endpoint path (with or without the /v1 prefix)…
  h=h.replace(/\/(?:v1\/)?(?:chat\/completions|completions|embeddings|messages|models|api\/(?:chat|tags|embeddings))$/i,"");
  h=h.replace(/\/+$/,"").replace(/\/v1$/i,"");// …then a bare trailing /v1; we add it back per provider
  return h.replace(/\/+$/,"");
}
// One Bearer header covers everything: OpenAI-compatible gateways (incl. z.ai/GLM
// and OpenRouter, which all expose /v1/chat/completions) and Ollama-cloud
// (ollama.com / turbo). Blank key = unauthenticated LAN Ollama.
function authH(provider,key){ return key?{"Authorization":"Bearer "+key}:{}; }
const hint401=(s)=>s===401?" — missing/invalid API key":s===403?" — key rejected (403)":"";
// returns {list, error} — never swallows the reason the dropdown is empty
async function listModels(host,key,provider){
  const base=normBase(host); if(!base) return {list:[],error:"no host set"};
  try{
    if(provider==="openai"){
      const url=base+"/v1/models"; const r=await fetch(url,{headers:authH(provider,key)});
      if(!r.ok) return {list:[],error:"HTTP "+r.status+hint401(r.status)+"  @ "+url};
      const j=await r.json(); return {list:(j.data||[]).map(m=>m.id).filter(Boolean),error:""};
    }
    const url=base+"/api/tags"; const r=await fetch(url,{headers:authH("ollama",key)});
    if(!r.ok) return {list:[],error:"HTTP "+r.status+hint401(r.status)+"  @ "+url};
    const j=await r.json(); return {list:(j.models||[]).map(m=>m.name).filter(Boolean),error:""};
  }catch(e){ return {list:[],error:(e&&e.message||"fetch failed")+" — likely CORS (a browser fetch the host won't allow this origin) or host unreachable"}; }
}
// Matryoshka-style resize: qwen3-embedding is trained so a leading slice is still
// a valid embedding — truncate to `dim`, then renormalize so cosine stays sane.
// dim falsy / >= native length → returned unchanged (native dimensionality).
function fitDim(vec,dim){
  if(!dim||!Array.isArray(vec)||vec.length<=dim) return vec;
  const v=vec.slice(0,dim); const n=Math.hypot(...v)||1; return v.map(x=>x/n);
}
async function embedUnified(text,host,model,key,provider,dim){
  const base=normBase(host);
  // NO MOCK: a blank embedding host is a hard error, never a faked vector.
  // (A hashed pseudo-embedding would look like it works while producing
  //  meaningless clusters — silently wrong. Set a real Ollama/OpenAI host.)
  if(!base) throw new Error("no embedding host set — open ⊞ data → embeddings and enter your Ollama/OpenAI host");
  if(!model) throw new Error("no embedding model set — pick one in ⊞ data → embeddings (e.g. qwen3-embedding:8b)");
  if(provider==="openai"){ const r=await fetch(base+"/v1/embeddings",{method:"POST",headers:{"Content-Type":"application/json",...authH("openai",key)},body:JSON.stringify({model,input:text,...(dim?{dimensions:dim}:{})})}); const j=await r.json(); return fitDim((j.data&&j.data[0]&&j.data[0].embedding)||[],dim); }
  // Ollama: /api/embeddings ignores a dimensions arg, so we resize client-side.
  const r=await fetch(base+"/api/embeddings",{method:"POST",headers:{"Content-Type":"application/json",...authH("ollama",key)},body:JSON.stringify({model,prompt:text})}); const j=await r.json(); return fitDim(j.embedding||[],dim);
}
// streaming chat — Ollama (/api/chat NDJSON), OpenAI-compatible
// (/v1/chat/completions SSE — covers OpenAI, OpenRouter, z.ai/GLM, vLLM…).
async function chatStream({host,model,key,provider,system,messages,context,onToken}){
  const base=normBase(host); if(!base) throw new Error("no chat host");
  const sys=(system||"")+(context&&context.length?("\n\nCONTEXT (cite titles in [brackets]):\n"+context.join("\n\n---\n\n")):"");
  let url,headers,body;
  if(provider==="openai"){
    url=base+"/v1/chat/completions";
    headers={"Content-Type":"application/json",...authH("openai",key)};
    body={model,stream:true,messages:[{role:"system",content:sys},...messages]};
  } else {
    url=base+"/api/chat";
    headers={"Content-Type":"application/json",...authH("ollama",key)};
    body={model,stream:true,messages:[{role:"system",content:sys},...messages]};
  }
  const r=await fetch(url,{method:"POST",headers,body:JSON.stringify(body)});
  if(!r.ok||!r.body){ let detail=""; try{ detail=" — "+(await r.text()).slice(0,200); }catch{} throw new Error("HTTP "+r.status+hint401(r.status)+detail); }
  const rd=r.body.getReader(),dec=new TextDecoder(); let buf="",out="";
  for(;;){ const {value,done}=await rd.read(); if(done) break; buf+=dec.decode(value,{stream:true});
    const lines=buf.split("\n"); buf=lines.pop();
    for(const ln of lines){ const s=ln.trim(); if(!s) continue;
      if(provider==="openai"){ if(!s.startsWith("data:")) continue; const d=s.slice(5).trim(); if(d==="[DONE]") continue; try{ const j=JSON.parse(d); const tok=j.choices?.[0]?.delta?.content||""; if(tok){out+=tok;onToken&&onToken(tok);} }catch{} }
      else { try{ const j=JSON.parse(s); const tok=j.message?.content||""; if(tok){out+=tok;onToken&&onToken(tok);} }catch{} }
    }
  }
  return out;
}
const KB_SYSTEM="You are the agent inside ThinkSpace — the user's live 3-D knowledge graph (the \"canvas\"). You can SEE the whole canvas: the LIVE GRAPH block lists every node, its type, its links, and which nodes are ISOLATED (no links). Treat it as the source of truth for STRUCTURE: answer structural questions directly from it — e.g. a node is unlinked because it has zero edges; what connects to what; where the gaps are. Never say you lack information about the graph's shape. The node-body CONTEXT is extra detail for meaning.\n\nYou can also CHANGE the canvas. When the user wants you to fix / join / link / add / remove / rename / re-type nodes, say briefly what you'll do, then END your reply with ONE fenced block, exactly:\n```thinkspace\n{\"actions\":[{\"op\":\"link\",\"a\":\"Tonic sol-fa\",\"b\":\"Solfège\",\"rel\":\"related\"}]}\n```\nOps: link {a,b,rel?} · unlink {a,b} · addNode {title,type?,body?,near?} · removeNode {name} · relabel {name,title} · setType {name,type} · joinIsolated {name} (auto-links it to its most similar node). Refer to nodes by their exact title. type ∈ {entity, process, abstract, place, artifact, field}; rel ∈ {is_a, part_of, causes, related, uses}. Emit ONLY the changes the user actually wants; omit the block entirely if none are needed. Cite node titles inline like [Title]. Be concise and concrete.";

// ── tiny, escape-first markdown → html for the reader pane ────────────────
function escapeHtml(s){ return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;"); }
// Trust-your-own-archives mode: this reader renders YOUR exported notes (Evernote,
// UpNote, Obsidian, ChatGPT) — not hostile web pages. So we DON'T strip links,
// inline styles, images, classes, or handlers; every href (incl. relative, file://,
// and javascript: bookmarklets) is preserved so nothing in your notes goes missing.
// The only thing dropped is <script> blocks — React's dangerouslySetInnerHTML can't
// execute them anyway, so keeping them would just dump raw code into the page.
function scrubHtml(h){ return String(h).replace(/<script[\s\S]*?<\/script>/gi,""); }
function mdToHtml(src){
  if(!src) return "";
  const fences=[]; let s=String(src).replace(/```([\s\S]*?)```/g,(_,c)=>{fences.push(c);return `\u0000${fences.length-1}\u0000`;});
  s=escapeHtml(s);
  s=s.replace(/^###### (.*)$/gm,"<h6>$1</h6>").replace(/^##### (.*)$/gm,"<h5>$1</h5>").replace(/^#### (.*)$/gm,"<h4>$1</h4>").replace(/^### (.*)$/gm,"<h3>$1</h3>").replace(/^## (.*)$/gm,"<h2>$1</h2>").replace(/^# (.*)$/gm,"<h1>$1</h1>");
  s=s.replace(/!\[([^\]]*)\]\(([^)]+)\)/g,'<img alt="$1" src="$2" style="max-width:100%;border-radius:8px"/>');
  s=s.replace(/\[([^\]]+)\]\(([^)]+)\)/g,'<a href="$2">$1</a>');
  s=s.replace(/\*\*([^*]+)\*\*/g,"<b>$1</b>").replace(/(^|[^*])\*([^*\n]+)\*/g,"$1<i>$2</i>");
  s=s.replace(/`([^`]+)`/g,'<code style="background:#11183a;padding:1px 5px;border-radius:4px">$1</code>');
  s=s.replace(/(?:^|\n)((?:[ \t]*[-*+] .*(?:\n|$))+)/g,(m,blk)=>{const items=blk.trim().split(/\n/).map(l=>`<li>${l.replace(/^[ \t]*[-*+] /,"")}</li>`).join("");return `\n<ul>${items}</ul>\n`;});
  s=s.replace(/(?:^|\n)((?:[ \t]*\d+\. .*(?:\n|$))+)/g,(m,blk)=>{const items=blk.trim().split(/\n/).map(l=>`<li>${l.replace(/^[ \t]*\d+\. /,"")}</li>`).join("");return `\n<ol>${items}</ol>\n`;});
  s=s.split(/\n{2,}/).map(p=>/^\s*<(h\d|ul|ol|pre|img|blockquote)/.test(p.trim())?p:(p.trim()?`<p>${p.trim().replace(/\n/g,"<br/>")}</p>`:"")).join("\n");
  s=s.replace(/\u0000(\d+)\u0000/g,(_,i)=>`<pre style="background:#0a0e22;border:1px solid #1f2a55;border-radius:8px;padding:10px;overflow:auto"><code>${escapeHtml(fences[+i])}</code></pre>`);
  return s;
}
// ── screenshot the live viewport (svg + html overlays) via foreignObject ──
function fmtTs(){ const d=new Date(),p=n=>String(n).padStart(2,"0"); return `${d.getFullYear()}${p(d.getMonth()+1)}${p(d.getDate())}-${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`; }
async function snapshotEl(node){
  if(!node) return;
  const rect=node.getBoundingClientRect(),w=Math.max(1,Math.ceil(rect.width)),h=Math.max(1,Math.ceil(rect.height));
  const clone=node.cloneNode(true);
  clone.querySelectorAll&&clone.querySelectorAll("svg").forEach(s=>s.setAttribute("xmlns","http://www.w3.org/2000/svg"));
  const xml=new XMLSerializer().serializeToString(clone);
  const svg=`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}"><foreignObject x="0" y="0" width="${w}" height="${h}"><div xmlns="http://www.w3.org/1999/xhtml" style="width:${w}px;height:${h}px;background:#05060f">${xml}</div></foreignObject></svg>`;
  const url="data:image/svg+xml;charset=utf-8,"+encodeURIComponent(svg);
  try{
    const img=new Image(); await new Promise((res,rej)=>{img.onload=res;img.onerror=rej;img.src=url;});
    const scale=2,cv=document.createElement("canvas"); cv.width=w*scale; cv.height=h*scale;
    const ctx=cv.getContext("2d"); ctx.scale(scale,scale); ctx.fillStyle="#05060f"; ctx.fillRect(0,0,w,h); ctx.drawImage(img,0,0);
    const png=cv.toDataURL("image/png");
    const a=document.createElement("a"); a.href=png; a.download=`thinkspace-${fmtTs()}.png`; a.click();
    try{ const blob=await(await fetch(png)).blob(); if(navigator.clipboard&&window.ClipboardItem) await navigator.clipboard.write([new window.ClipboardItem({"image/png":blob})]); }catch{}
  }catch(err){ console.warn("snapshot failed",err); }
}
// ── rasterize an <svg> DIRECTLY (foreignObject can't draw a nested SVG → blank PNG).
//    The star-map is pure SVG shapes + text, so serialize→Image→canvas works cleanly. ──
async function snapshotSvg(svg){
  if(!svg) return;
  try{
    const rect=svg.getBoundingClientRect();
    const vb=svg.viewBox&&svg.viewBox.baseVal;
    const w=Math.max(1,Math.ceil(vb&&vb.width?vb.width:rect.width)), h=Math.max(1,Math.ceil(vb&&vb.height?vb.height:rect.height));
    const clone=svg.cloneNode(true);
    clone.setAttribute("xmlns","http://www.w3.org/2000/svg");
    clone.setAttribute("width",w); clone.setAttribute("height",h);
    clone.setAttribute("viewBox",`0 0 ${w} ${h}`);
    clone.setAttribute("preserveAspectRatio","xMidYMid meet");
    const xml=new XMLSerializer().serializeToString(clone);
    const url="data:image/svg+xml;charset=utf-8,"+encodeURIComponent(xml);
    const img=new Image(); await new Promise((res,rej)=>{img.onload=res;img.onerror=rej;img.src=url;});
    const scale=2,cv=document.createElement("canvas"); cv.width=w*scale; cv.height=h*scale;
    const ctx=cv.getContext("2d"); ctx.scale(scale,scale); ctx.fillStyle="#05060f"; ctx.fillRect(0,0,w,h); ctx.drawImage(img,0,0,w,h);
    const png=cv.toDataURL("image/png");
    const a=document.createElement("a"); a.href=png; a.download=`thinkspace-${fmtTs()}.png`; a.click();
    try{ const blob=await(await fetch(png)).blob(); if(navigator.clipboard&&window.ClipboardItem) await navigator.clipboard.write([new window.ClipboardItem({"image/png":blob})]); }catch{}
  }catch(err){ console.warn("svg snapshot failed",err); }
}

// ── persistence: plain browser localStorage ────────────────────────────────
// Settings (hosts, keys, models, dim, store config, annotations) are written to
// localStorage on every change and read back on load — set them once, they stick.
// In-memory is only a last-resort fallback if a host truly walls off storage; the
// Export/Import-settings buttons in the tray cover moving settings between machines.
const PERSIST=(()=>{
  const mem={}; let ls=null;
  try{ const k="__ts_probe"; window.localStorage.setItem(k,"1"); window.localStorage.removeItem(k); ls=window.localStorage; }catch(e){ ls=null; }
  return {
    mode: ls? "browser storage" : "session only",
    get(k){ try{ if(ls){ const v=ls.getItem(k); if(v!=null) return v; } }catch(e){} return (k in mem)? mem[k] : null; },
    set(k,v){ mem[k]=v; try{ if(ls) ls.setItem(k,v); }catch(e){} }
  };
})();

// ── global version line: minor is a plain release constant (bump by hand when cutting a
//    release); build is a 5-digit epoch-MINUTE modulo stamp — ~one tick per minute, wraps
//    every ~69 days. e.g. v.1.107.77726. Computed at load, not stored. ──
const BUILD_NO=String(Math.floor(Date.now()/60000)%100000).padStart(5,"0");
const MINOR_NO=107;
const APP_VERSION=`v.1.${MINOR_NO}.${BUILD_NO}`;
const APP_COPYRIGHT=`© ${new Date().getFullYear()} Robin L. M. Cheung. All Rights Reserved`;
// one HUD accent shared by the version line, copyright, the spread toggle, and node factoids
const HUD_ACCENT="#7ee8fa";

function ThinkSpace(){
  const W=760,H=520,FOV=620;
  const [,force]=useState(0);
  const tick=useCallback(()=>force(t=>(t+1)%1e9),[]);

  const [cfg,setCfg]=useState({damp:0.78,lookDamp:0.86,spring:0.018,charge:9000,breathe:0.5,glow:1,vignette:true,recenterIdle:true,spinDecay:0.94,przewalski:false});
  const cfgRef=useRef(cfg); cfgRef.current=cfg;
  const [trayOpen,setTrayOpen]=useState(false);

  // live graph: starts as SEED, replaced when a folder is ingested
  const graph=useRef(SEED);
  const [graphVer,setGraphVer]=useState(0);
  const [ingest,setIngest]=useState({open:false,busy:false,count:0,total:0,endpoint:"",model:"qwen3-embedding:8b",thresh:0.55,urls:"",phase:""});
  const [load,setLoad]=useState({maxNodes:600,minConf:0,total:0,shown:0,err:null});
  // ── unified search: RAG hits glow gold, Wikipedia gaps appear as dim "ghost" baubles you can add ──
  const [search,setSearch]=useState({q:"",hits:[],ghosts:[],adding:[],err:""});
  const [ctx,setCtx]=useState(null);   // node context menu {x,y,id,ghost}

  // ── tabs · connections · chat · reader (new) ──
  const [tab,setTab]=useState("spatial");          // "spatial" | "reader"
  const tabRef=useRef(tab); tabRef.current=tab;
  const [conn,setConn]=useState(()=>{
    const def={ embedHost:"http://192.168.0.41:11434", embedModel:"qwen3-embedding:8b", embedKey:"", embedProvider:"ollama", embedDim:4096,
                chatHost:"http://192.168.0.41:11434",  chatModel:"gemma4:e2b",          chatKey:"",  chatProvider:"ollama" };
    try{ const s=JSON.parse(PERSIST.get("ts.conn")||"null"); if(s){ const m={...def,...s};
      if(m.embedProvider==="anthropic") m.embedProvider="openai";   // retired provider → openai-compat
      if(m.chatProvider==="anthropic")  m.chatProvider="openai";
      return m; } }catch(e){}
    return def;
  });
  const connRef=useRef(conn); connRef.current=conn;
  useEffect(()=>{ PERSIST.set("ts.conn",JSON.stringify(conn)); },[conn]); // persist hosts/keys/models/dim so you never re-enter them
  // external graph/vector stores — Qdrant + Neo4j (several-GB graphs live here, not in memory)
  const [stores,setStores]=useState(()=>{
    const def={ qdrantUrl:"http://192.168.0.252:6333", qdrantColl:"thinkspace", qdrantKey:"",
                neoUrl:"http://192.168.0.46:7474", neoDb:"neo4j", neoUser:"neo4j", neoPass:"" };
    try{ const s=JSON.parse(PERSIST.get("ts.stores")||"null"); if(s) return {...def,...s}; }catch(e){}
    return def;
  });
  useEffect(()=>{ PERSIST.set("ts.stores",JSON.stringify(stores)); },[stores]);
  const [storeMsg,setStoreMsg]=useState("");
  const ingestCancel=useRef(false);                 // long ingest loops check this to bail out
  const [storeBusy,setStoreBusy]=useState(false);   // a Qdrant/Neo4j push or pull is in flight
  const storeAbort=useRef(null);                     // AbortController for the in-flight store fetch
  const beginStore=()=>{ try{storeAbort.current&&storeAbort.current.abort();}catch(e){} storeAbort.current=new AbortController(); setStoreBusy(true); return storeAbort.current.signal; };
  const endStore=()=>{ setStoreBusy(false); storeAbort.current=null; };
  const cancelStore=()=>{ try{storeAbort.current&&storeAbort.current.abort();}catch(e){} setStoreBusy(false); setStoreMsg("✕ cancelled"); };
  const isAbort=(e)=>e&&(e.name==="AbortError"||/abort/i.test(String(e.message||e)));
  const [big,setBig]=useState(false);              // maximize → fill the viewport
  const [notes,setNotes]=useState(false);          // collated notebook panel
  const [models,setModels]=useState({embed:[],chat:[]});
  const [modelsBusy,setModelsBusy]=useState({embed:false,chat:false});
  const [modelsErr,setModelsErr]=useState({embed:"",chat:""});
  const [chat,setChat]=useState({open:true,busy:false,q:"",msgs:[]});
  const [readerId,setReaderId]=useState("entropy");
  const nodeVecs=useRef({});                        // id -> Float-ish[] for RAG retrieval

  const sim=useRef(null);
  if(!sim.current){
    sim.current={nodes:SEED.nodes.map((n,i)=>{const a=(i/SEED.nodes.length)*Math.PI*2;const p=[Math.cos(a)*220,Math.sin(a*1.3)*120,Math.sin(a)*180];return{...n,p,v:[0,0,0],home:[...p]};})};
  }

  // ── load ANY conformant graph JSON (getrecall, claude-export, client-files,
  //    whatever) — keyed on SHAPE not filename. scale filters for big graphs. ──
  const loadGraphJson=async(file)=>{
    try{
      const j=JSON.parse(await file.text());
      // accept a few shapes: {nodes,edges} | {graph:{...}} | bare array of nodes
      const g = j.nodes ? j : (j.graph||j.data||{nodes:Array.isArray(j)?j:[],edges:[]});
      let nodes=(g.nodes||[]).map((n,i)=>({
        id:n.id!=null?String(n.id):("n"+i),
        title:n.title||n.name||n.label||("node "+i),
        onto:n.onto||n.type||n.category||"field",
        raw:n.raw||n.text||n.body||n.snippet||"",
        html:n.html||n.content_html||"",
        url:n.url||n.link||n.source||n.href||n.uri||(n.meta&&(n.meta.url||n.meta.link))||"",
        media:n.media||[],
        source:n.source||n.provenance||"",
        created:n.created||n.ts||n.added||Date.now(),   // preserve original stamp, else stamp on import
        vec:(Array.isArray(n.vec)?n.vec:(Array.isArray(n.embedding)?n.embedding:null)),
        _pos:n.pos||n.position||null,
      }));
      // normalize unknown onto values into our palette buckets
      const okOnto=Object.keys(ONTO);
      nodes=nodes.map(n=>({...n,onto:okOnto.includes(n.onto)?n.onto:okOnto[Math.abs(hashStr(String(n.onto)))%okOnto.length]}));
      let edges=(g.edges||g.links||[]).map(e=>({
        from:String(e.from!=null?e.from:e.source),to:String(e.to!=null?e.to:e.target),
        rel:e.rel||e.type||"related",conf:e.conf!=null?e.conf:(e.weight!=null?e.weight:0.7),
      }));
      // ── scale guard: huge graphs (e.g. 4198 nodes) won't render in SVG. cap
      //    to the most-connected slice + a confidence floor, keep it flyable. ──
      const CAP=load.maxNodes||300;
      if(nodes.length>CAP){
        const deg={}; edges.forEach(e=>{deg[e.from]=(deg[e.from]||0)+e.conf;deg[e.to]=(deg[e.to]||0)+e.conf;});
        nodes=[...nodes].sort((a,b)=>(deg[b.id]||0)-(deg[a.id]||0)).slice(0,CAP);
        const keep=new Set(nodes.map(n=>n.id));
        edges=edges.filter(e=>keep.has(e.from)&&keep.has(e.to)&&e.conf>=(load.minConf||0));
      } else {
        edges=edges.filter(e=>e.conf>=(load.minConf||0));
      }
      const idset=new Set(nodes.map(n=>n.id));
      edges=edges.filter(e=>idset.has(e.from)&&idset.has(e.to));
      const placed=nodes.map((n,i)=>{
        const p=n._pos?[...n._pos]:[(Math.random()-0.5)*340,(Math.random()-0.5)*340,(Math.random()-0.5)*340];
        return {id:n.id,title:n.title,onto:n.onto,raw:n.raw,html:n.html,url:n.url,media:n.media,source:n.source,created:n.created,vec:n.vec,p};
      });
      graph.current={nodes:placed.map(n=>({id:n.id,title:n.title,onto:n.onto,raw:n.raw,html:n.html,url:n.url,media:n.media,source:n.source,created:n.created})),edges};
      sim.current={nodes:placed.map(n=>({...n,v:[0,0,0],home:[...n.p]}))};
      const nv={}; placed.forEach(n=>{ if(n.vec) nv[n.id]=n.vec; }); nodeVecs.current=nv;
      setSel(placed[0]?.id||"n0"); setReaderId(placed[0]?.id||"n0"); setGraphVer(v=>v+1);
      setLoad(s=>({...s,total:(g.nodes||[]).length,shown:placed.length,err:null}));
      setIngest(s=>({...s,open:false}));
    }catch(err){ setLoad(s=>({...s,err:"bad JSON: "+err.message})); }
  };
  // feed a plain {nodes,edges} object through the same scaling/sim pipeline as a file load
  const applyGraphObj=(obj)=>loadGraphJson(new File([JSON.stringify(obj)],"pull.json",{type:"application/json"}));

  // ── exports: the graph is stored as JSON, so it round-trips to disk / Qdrant / Neo4j ──
  const exportGraphJson=()=>{
    const g=graph.current;
    const real=g.nodes.filter(n=>!n.ghost);   // un-added wiki ghosts are not part of the saved graph
    const out={nodes:real.map(n=>({id:n.id,title:n.title,onto:n.onto,raw:n.raw,html:n.html,url:n.url||undefined,source:n.source||undefined,created:n.created||undefined,vec:nodeVecs.current[n.id]||undefined})),edges:g.edges};
    download("thinkspace-graph.json",JSON.stringify(out),"application/json");
    setStoreMsg("✓ exported "+real.length+" nodes · "+g.edges.length+" edges → thinkspace-graph.json");
  };
  // ── Qdrant connector (vectors live here for several-GB corpora, not in memory) ──
  const qurl=()=>String(stores.qdrantUrl||"").replace(/\/+$/,"");
  const qH=()=>({"Content-Type":"application/json",...(stores.qdrantKey?{"api-key":stores.qdrantKey}:{})});
  const qdrantPush=async()=>{
    if(!qurl()||!stores.qdrantColl){setStoreMsg("set Qdrant url + collection");return;}
    const pts=graph.current.nodes.map((n,i)=>{const v=nodeVecs.current[n.id]; return v?{id:i+1,vector:Array.from(v),payload:{id:n.id,title:n.title,onto:n.onto,raw:(n.raw||"").slice(0,4000)}}:null;}).filter(Boolean);
    if(!pts.length){setStoreMsg("no vectors yet — ingest a folder first (Qdrant needs embeddings)");return;}
    const sig=beginStore();
    try{
      setStoreMsg("Qdrant: ensuring collection…");
      await fetch(qurl()+"/collections/"+encodeURIComponent(stores.qdrantColl),{method:"PUT",headers:qH(),body:JSON.stringify({vectors:{size:pts[0].vector.length,distance:"Cosine"}}),signal:sig});
      let done=0;
      for(let i=0;i<pts.length;i+=256){ const batch=pts.slice(i,i+256);
        const r=await fetch(qurl()+"/collections/"+encodeURIComponent(stores.qdrantColl)+"/points?wait=true",{method:"PUT",headers:qH(),body:JSON.stringify({points:batch}),signal:sig});
        if(!r.ok){setStoreMsg("Qdrant HTTP "+r.status+" — "+(await r.text()).slice(0,120));return;}
        done+=batch.length; setStoreMsg("Qdrant: upserted "+done+"/"+pts.length); }
      setStoreMsg("✓ Qdrant: "+pts.length+" points → "+stores.qdrantColl);
    }catch(e){ if(!isAbort(e)) setStoreMsg("Qdrant failed: "+(e.message||e)+" — likely CORS (works from the native shell)"); }
    finally{ endStore(); }
  };
  const qdrantPull=async()=>{
    if(!qurl()||!stores.qdrantColl){setStoreMsg("set Qdrant url + collection");return;}
    const sig=beginStore();
    try{ setStoreMsg("Qdrant: scrolling points…"); let next=null,all=[];
      for(let pg=0;pg<60;pg++){ const r=await fetch(qurl()+"/collections/"+encodeURIComponent(stores.qdrantColl)+"/points/scroll",{method:"POST",headers:qH(),body:JSON.stringify({limit:512,with_payload:true,with_vector:false,...(next?{offset:next}:{})}),signal:sig});
        if(!r.ok){setStoreMsg("Qdrant HTTP "+r.status);return;} const j=await r.json();
        const pts=(j.result&&j.result.points)||[]; all=all.concat(pts); next=j.result&&j.result.next_page_offset; if(!next||!pts.length)break; }
      const nodes=all.map(p=>({id:(p.payload&&p.payload.id)||String(p.id),title:(p.payload&&p.payload.title)||String(p.id),onto:(p.payload&&p.payload.onto)||"entity",raw:(p.payload&&p.payload.raw)||""}));
      if(!nodes.length){setStoreMsg("Qdrant: collection empty");return;}
      await applyGraphObj({nodes,edges:[]}); setStoreMsg("✓ Qdrant → "+nodes.length+" nodes (re-ingest to rebuild similarity edges)");
    }catch(e){ if(!isAbort(e)) setStoreMsg("Qdrant failed: "+(e.message||e)+" — likely CORS"); }
    finally{ endStore(); }
  };
  // ── Neo4j connector (HTTP transaction endpoint; bolt isn't reachable from a browser) ──
  const neoEp=()=>String(stores.neoUrl||"").replace(/\/+$/,"")+"/db/"+encodeURIComponent(stores.neoDb||"neo4j")+"/tx/commit";
  const neoH=()=>({"Content-Type":"application/json","Accept":"application/json",...(stores.neoUser?{"Authorization":"Basic "+btoa(stores.neoUser+":"+(stores.neoPass||""))}:{})});
  const neo4jPush=async()=>{
    if(!stores.neoUrl){setStoreMsg("set Neo4j url");return;}
    const g=graph.current;
    const nodes=g.nodes.map(n=>({id:String(n.id),title:n.title||"",onto:n.onto||"",raw:(n.raw||"").slice(0,4000)}));
    const rels=g.edges.map(e=>({from:String(e.from),to:String(e.to),rel:e.rel||"related"}));
    const statements=[
      {statement:"UNWIND $rows AS r MERGE (n:Concept {id:r.id}) SET n.title=r.title, n.onto=r.onto, n.raw=r.raw",parameters:{rows:nodes}},
      {statement:"UNWIND $rows AS r MATCH (a:Concept {id:r.from}),(b:Concept {id:r.to}) MERGE (a)-[x:REL {rel:r.rel}]->(b)",parameters:{rows:rels}},
    ];
    const sig=beginStore();
    try{ setStoreMsg("Neo4j: writing "+nodes.length+" nodes / "+rels.length+" rels…");
      const r=await fetch(neoEp(),{method:"POST",headers:neoH(),body:JSON.stringify({statements}),signal:sig});
      const j=await r.json().catch(()=>({}));
      if(!r.ok||(j.errors&&j.errors.length)){ setStoreMsg("Neo4j error: "+(j.errors&&j.errors[0]?(j.errors[0].code+" — "+j.errors[0].message):("HTTP "+r.status))); return; }
      setStoreMsg("✓ Neo4j: "+nodes.length+" nodes · "+rels.length+" rels → "+(stores.neoDb||"neo4j"));
    }catch(e){ if(!isAbort(e)) setStoreMsg("Neo4j failed: "+(e.message||e)+" — likely CORS (works from the native shell)"); }
    finally{ endStore(); }
  };
  const neo4jPull=async()=>{
    if(!stores.neoUrl){setStoreMsg("set Neo4j url");return;}
    const statements=[
      {statement:"MATCH (n:Concept) RETURN n.id, n.title, n.onto, n.raw LIMIT 5000"},
      {statement:"MATCH (a:Concept)-[x:REL]->(b:Concept) RETURN a.id, b.id, x.rel LIMIT 20000"},
    ];
    const sig=beginStore();
    try{ setStoreMsg("Neo4j: reading…");
      const r=await fetch(neoEp(),{method:"POST",headers:neoH(),body:JSON.stringify({statements}),signal:sig});
      const j=await r.json(); if(j.errors&&j.errors.length){setStoreMsg("Neo4j error: "+j.errors[0].message);return;}
      const rn=j.results[0].data.map(d=>d.row), re=j.results[1].data.map(d=>d.row);
      const nodes=rn.map(([id,title,onto,raw])=>({id,title:title||id,onto:onto||"entity",raw:raw||""}));
      const edges=re.map(([from,to,rel])=>({from,to,rel:rel||"related"}));
      if(!nodes.length){setStoreMsg("Neo4j: no :Concept nodes found");return;}
      await applyGraphObj({nodes,edges}); setStoreMsg("✓ Neo4j → "+nodes.length+" nodes · "+edges.length+" rels");
    }catch(e){ if(!isAbort(e)) setStoreMsg("Neo4j failed: "+(e.message||e)+" — likely CORS"); }
    finally{ endStore(); }
  };
  const ONTO_KEYS=Object.keys(ONTO);
  const ingestFiles=async(fileList)=>{
    const files=Array.from(fileList).filter(f=>/\.md$/i.test(f.name));
    if(!files.length) return;
    const C=connRef.current;
    // NO MOCK: refuse to ingest without a real embedder rather than faking vectors.
    if(!normBase(C.embedHost)||!C.embedModel){ setIngest(s=>({...s,err:"Set an embedding host + model first (⊞ data → embeddings). Blank no longer fabricates vectors."})); return; }
    ingestCancel.current=false;
    setIngest(s=>({...s,busy:true,err:"",count:0,total:files.length}));
    const docs=[];
    for(let i=0;i<files.length;i++){
      if(ingestCancel.current){ setIngest(s=>({...s,busy:false})); return; }   // cancelled mid-run — keep nothing partial
      const f=files[i];
      const text=await f.text();
      const parsed=parseMd(f.name,text);
      // folder name (first path segment) → color hint only; drives nothing else
      const path=(f.webkitRelativePath||"").split("/");
      const folder=path.length>1?path[path.length-2]:"";
      const onto=ONTO_KEYS[Math.abs(hashStr(folder))%ONTO_KEYS.length];
      let vec;
      try{ vec=await embedUnified(parsed.text, C.embedHost, C.embedModel, C.embedKey, C.embedProvider, C.embedDim); }
      catch(e){ setIngest(s=>({...s,busy:false,err:"Embedding failed at “"+f.name+"”: "+(e&&e.message||e)+"  (nothing saved — no fake fallback)"})); return; }
      docs.push({id:"n"+i, title:parsed.title, onto, vec, raw:parsed.raw, folder, source:f.name, url:(f.webkitRelativePath?"file:///"+f.webkitRelativePath:""), created:Date.now()});
      setIngest(s=>({...s,count:i+1}));
    }
    if(ingestCancel.current){ setIngest(s=>({...s,busy:false})); return; }
    // edges: cosine similarity over the threshold
    const edges=[];
    for(let i=0;i<docs.length;i++)for(let j=i+1;j<docs.length;j++){
      const c=cosine(docs[i].vec,docs[j].vec);
      if(c>=ingest.thresh) edges.push({from:docs[i].id,to:docs[j].id,rel:"related",conf:Math.min(0.99,c)});
    }
    // 3D positions from embedding projection (the "shadow"); sim adds life on top
    const pos=projectTo3D(docs.map(d=>d.vec));
    graph.current={nodes:docs.map((d,i)=>({id:d.id,title:d.title,onto:d.onto,raw:d.raw,url:d.url,source:d.source,created:d.created})),edges};
    sim.current={nodes:docs.map((d,i)=>({id:d.id,title:d.title,onto:d.onto,raw:d.raw,url:d.url,source:d.source,created:d.created,p:[...pos[i]],v:[0,0,0],home:[...pos[i]]}))};
    const nv={}; docs.forEach(d=>{ nv[d.id]=d.vec; }); nodeVecs.current=nv;
    setSel(docs[0]?.id||"n0"); setReaderId(docs[0]?.id||"n0"); setGraphVer(v=>v+1);
    setIngest(s=>({...s,busy:false,open:false}));
  };

  // ── append nodes/edges to the LIVE graph (bulk loading accumulates, never wipes) ──
  const mergeIntoGraph=(newNodes,newEdges,vecMap)=>{
    if(!newNodes.length) return;
    const wv=newNodes.filter(n=>vecMap[n.id]);
    const pos={};
    try{ if(wv.length>=3){ const p=projectTo3D(wv.map(n=>vecMap[n.id])); wv.forEach((n,i)=>pos[n.id]=p[i]); } }catch(e){}
    newNodes.forEach(n=>{ if(!pos[n.id]) pos[n.id]=[(Math.random()-0.5)*360,(Math.random()-0.5)*360,(Math.random()-0.5)*360]; });
    const bare=n=>({id:n.id,title:n.title,onto:n.onto,raw:n.raw,url:n.url,source:n.source,created:n.created});
    const g=graph.current;
    g.nodes=g.nodes.concat(newNodes.map(bare));
    g.edges=g.edges.concat(newEdges);
    sim.current.nodes=sim.current.nodes.concat(newNodes.map(n=>({...bare(n),p:[...pos[n.id]],v:[0,0,0],home:[...pos[n.id]]})));
    Object.assign(nodeVecs.current,vecMap);
    setGraphVer(v=>v+1);
  };

  // ── rich ingest: docs (PDF/HTML/text) or URLs → a document node + embedded passage
  //    nodes (concepts within), linked part-of, then cross-similarity edges. Accumulates. ──
  const ingestSources=async(items)=>{
    // items: [{kind:'file',file} | {kind:'url',url}]
    if(!items.length) return;
    const C=connRef.current;
    if(!normBase(C.embedHost)||!C.embedModel){ setIngest(s=>({...s,err:"Set an embedding host + model first (⊞ data → embeddings). No fake vectors — ingest needs a real embedder."})); return; }
    ingestCancel.current=false;
    setIngest(s=>({...s,busy:true,err:"",count:0,total:items.length,phase:"reading"}));
    const ts=Date.now().toString(36);
    const newNodes=[], newEdges=[], vecMap={};
    const OK=Object.keys(ONTO);
    const embed=(txt)=>embedUnified(txt,C.embedHost,C.embedModel,C.embedKey,C.embedProvider,C.embedDim);
    let made=0;
    for(let i=0;i<items.length;i++){
      if(ingestCancel.current){ setIngest(s=>({...s,busy:false,phase:""})); return; }
      const it=items[i];
      const label=it.kind==="url"?it.url:it.file.name;
      setIngest(s=>({...s,count:i,phase:"reading "+label}));
      let text="", srcUrl="", srcName=label, title=label;
      try{
        if(it.kind==="url"){
          srcUrl=it.url; srcName=it.url;
          const r=await fetch(it.url); if(!r.ok) throw new Error("HTTP "+r.status);
          const ct=r.headers.get("content-type")||"";
          if(/pdf/.test(ct)||/\.pdf(\?|$)/i.test(it.url)){ const blob=await r.blob(); text=await pdfToText(new File([blob],"u.pdf")); }
          else { const raw=await r.text(); text=/html/.test(ct)||/^\s*</.test(raw)?htmlToText(raw):raw; }
          try{ title=decodeURIComponent(it.url.split("/").filter(Boolean).pop()||it.url).replace(/[-_]+/g," ").replace(/\.[a-z0-9]+$/i,"")||it.url; }catch(e){ title=it.url; }
        } else {
          srcName=it.file.name; srcUrl=it.file.webkitRelativePath?("file:///"+it.file.webkitRelativePath):"";
          text=await extractFileText(it.file);
          title=(it.file.name||"document").replace(/\.[a-z0-9]+$/i,"");
        }
      }catch(e){ setIngest(s=>({...s,err:"Couldn't read "+label+": "+(e&&e.message||e)+(it.kind==="url"?" (URL fetch is CORS-gated in preview; works in the native shell)":"")})); continue; }
      if(!text||text.trim().length<40){ setIngest(s=>({...s,err:"No extractable text in "+label+" (scanned/image PDF?)"})); continue; }
      // document node
      const docId="doc-"+ts+"-"+made;
      let dv; try{ dv=await embed((title+"\n"+text).slice(0,8000)); }
      catch(e){ setIngest(s=>({...s,busy:false,phase:"",err:"Embedding failed: "+(e&&e.message||e)+" (nothing saved — no fake fallback)"})); return; }
      vecMap[docId]=dv;
      newNodes.push({id:docId,title,onto:"artifact",raw:text.slice(0,1500),source:srcName,url:srcUrl,created:Date.now()});
      // passage nodes (concepts within)
      const chunks=chunkDoc(text,12);
      setIngest(s=>({...s,phase:"embedding "+chunks.length+" passages · "+title}));
      for(let ci=0;ci<chunks.length;ci++){
        if(ingestCancel.current){ setIngest(s=>({...s,busy:false,phase:""})); return; }
        const cId=docId+"-c"+ci;
        let cv; try{ cv=await embed(chunks[ci]); }
        catch(e){ setIngest(s=>({...s,busy:false,phase:"",err:"Embedding failed: "+(e&&e.message||e)})); return; }
        vecMap[cId]=cv;
        newNodes.push({id:cId,title:conceptTitle(chunks[ci]),onto:OK[Math.abs(hashStr(chunks[ci]))%OK.length],raw:chunks[ci],source:srcName,url:srcUrl,created:Date.now()});
        newEdges.push({from:docId,to:cId,rel:"part_of",conf:0.9});
      }
      made++;
      setIngest(s=>({...s,count:i+1}));
    }
    if(!newNodes.length){ setIngest(s=>({...s,busy:false,phase:""})); return; }
    // similarity edges among the new vectors…
    setIngest(s=>({...s,phase:"linking "+newNodes.length+" nodes"}));
    const ids=Object.keys(vecMap), th=ingest.thresh;
    for(let i=0;i<ids.length;i++)for(let j=i+1;j<ids.length;j++){
      if(ids[j]===ids[i]+"") continue;
      const c=cosine(vecMap[ids[i]],vecMap[ids[j]]);
      if(c>=th) newEdges.push({from:ids[i],to:ids[j],rel:"related",conf:Math.min(0.98,c)});
    }
    // …and a best-match edge from each new node into the pre-existing graph
    const existing=Object.keys(nodeVecs.current).filter(id=>!vecMap[id]);
    if(existing.length) for(const nid of ids){ let best=null,bc=th;
      for(const eid of existing){ const c=cosine(vecMap[nid],nodeVecs.current[eid]); if(c>bc){bc=c;best=eid;} }
      if(best) newEdges.push({from:nid,to:best,rel:"related",conf:Math.min(0.97,bc)});
    }
    mergeIntoGraph(newNodes,newEdges,vecMap);
    setSel(newNodes[0].id); setReaderId(newNodes[0].id);
    setIngest(s=>({...s,busy:false,phase:"",err:"",done:newNodes.length}));
  };
  const ingestDocs=(fileList)=>ingestSources(Array.from(fileList||[]).map(file=>({kind:"file",file})));
  const ingestUrls=(text)=>{
    const urls=String(text||"").split(/[\s,]+/).map(u=>u.trim()).filter(u=>/^https?:\/\//i.test(u));
    if(!urls.length){ setIngest(s=>({...s,err:"Paste one or more http(s) URLs (newline or space separated)."})); return; }
    ingestSources(urls.map(url=>({kind:"url",url})));
  };

  const cam=useRef({pos:[0,0,-560],yaw:0,pitch:0,vYaw:0,vPitch:0,vFwd:0});
  const lookDrag=useRef(null), nodeDrag=useRef(null);
  const [sel,setSel]=useState("entropy");
  const selRef=useRef(sel); selRef.current=sel;
  const [hover,setHover]=useState(null);
  const [grabbed,setGrabbed]=useState(null);
  const [reResolving,setReResolving]=useState(false);
  const [edgeMode,setEdgeMode]=useState("vectors");
  const [showHud,setShowHud]=useState(true);
  const [diveInto,setDiveInto]=useState(null);
  const [cardPos,setCardPos]=useState(null);   // {x,y} once the dive card has been dragged; null = default anchor
  const cardDrag=useRef(null);
  const [bodyFetch,setBodyFetch]=useState({});  // id → "loading" | "done"; clears the eternal "Fetching…" spinner
  const seenGhosts=useRef(new Set());          // titles already suggested, so search doesn't loop in circles
  const [selMenu,setSelMenu]=useState(null);   // {x,y,text} text-selection context menu in wiki
  const [selAct,setSelAct]=useState(null);     // null=textual menu shown; "hl"=colour picker stage
  const [selVoice,setSelVoice]=useState(null); // {rec,chunks,sec,err} live voice-note recording
  const voiceRec=useRef(null);
  const [selDraft,setSelDraft]=useState(null); // {color,note} inline note composer for the dive-in panel
  const [annos,setAnnos]=useState(()=>{ try{return JSON.parse(PERSIST.get("ts.annos")||"[]");}catch(e){return [];} }); // highlights + notes, persisted across reloads
  useEffect(()=>{ PERSIST.set("ts.annos",JSON.stringify(annos)); },[annos]);
  // dive-card highlighter: the card body is innerHTML'd into cardBodyRef, so we re-wrap the
  // saved selections in <mark> after each render — same pattern the reader uses. Without this
  // the card only *listed* marks; it never painted the neon background on the text itself.
  const cardBodyRef=useRef(null);
  useEffect(()=>{
    const root=cardBodyRef.current; if(!root) return;
    root.querySelectorAll("mark.ts-hl").forEach(m=>{ const p=m.parentNode; while(m.firstChild) p.insertBefore(m.firstChild,m); p.removeChild(m); p.normalize&&p.normalize(); });
    (annos||[]).filter(a=>a.nodeId===diveInto&&(a.kind==="highlight"||a.kind==="annotate")&&a.text).forEach(a=>paintMark(root,a.text,a.color,a.note));
  },[diveInto,annos,bodyFetch]);
  const [journey,setJourney]=useState(false);
  const [trail,setTrail]=useState(["entropy"]);
  const lastMouse=useRef({x:380,y:260});
  const interact=useRef(performance.now());   // last manipulation time (for idle self-right)
  const svgRef=useRef(null);
  const stageRef=useRef(null);                // viewport wrapper, used for screenshots

  // egg / slingshot state
  const eggs=useRef([]), shards=useRef([]), shatter=useRef({});
  const eggCooldown=useRef(0);
  const charge=useRef({active:false,since:0,armed:false});
  const repelPulse=useRef(1);                 // multitouch spread → transient node-repulsion boost
  const touchState=useRef(null);              // active touch gesture (pinch / spread)
  const [reticle,setReticle]=useState(null);

  // sound (defaults ON — actual audio graph arms on first user gesture, browser policy)
  const [sound,setSound]=useState(true);
  const soundRef=useRef(sound); soundRef.current=sound;
  const audio=useRef({ctx:null,pad:null});

  // sky (device orientation) — defaults ON
  const [sky,setSky]=useState({on:true,lat:null,lon:null});
  const skyHeading=useRef(0);

  // focus gate (so E doesn't leak into page inputs)
  const [focused,setFocused]=useState(false);
  const focusRef=useRef(false); focusRef.current=focused;
  const rootRef=useRef(null);

  const byId=()=>Object.fromEntries(sim.current.nodes.map(n=>[n.id,n]));

  useEffect(()=>{ setTrail(tr=>tr[tr.length-1]===sel?tr:[...tr,sel]); },[sel]);

  // ── audio (procedural, no files) ──
  const initAudio=()=>{
    if(audio.current.ctx) return;
    const ctx=new (window.AudioContext||window.webkitAudioContext)();
    audio.current.ctx=ctx;
    const g=ctx.createGain(); g.gain.value=0; g.connect(ctx.destination);
    // brighter than before so the chord is actually audible, but still soft-edged
    const lp=ctx.createBiquadFilter(); lp.type="lowpass"; lp.frequency.value=1500; lp.Q.value=0.4; lp.connect(g);
    // A-minor with an added major-9th up top → melancholic root, hopeful shimmer.
    // [root drone, fifth, octave, minor-3rd (the ache), high fifth, the hopeful 9th]
    const voices=[
      {f:55,    g:0.20, type:"sine",     lfo:0.05},
      {f:82.4,  g:0.13, type:"sine",     lfo:0.07},
      {f:110,   g:0.11, type:"sine",     lfo:0.06},
      {f:130.8, g:0.07, type:"triangle", lfo:0.09},  // C  — minor third
      {f:164.8, g:0.06, type:"triangle", lfo:0.08},  // E
      {f:246.9, g:0.045,type:"triangle", lfo:0.11},  // B  — the rising, hopeful tone
    ];
    voices.forEach((v)=>{
      const o=ctx.createOscillator(); o.type=v.type; o.frequency.value=v.f;
      const og=ctx.createGain(); og.gain.value=v.g;
      const lfo=ctx.createOscillator(); lfo.frequency.value=v.lfo;
      const lg=ctx.createGain(); lg.gain.value=4; lfo.connect(lg); lg.connect(o.detune);
      // slow tremolo so the higher, hopeful voices breathe in and out
      const trem=ctx.createOscillator(); trem.frequency.value=0.08+v.lfo*0.3;
      const tg=ctx.createGain(); tg.gain.value=v.g*0.5; trem.connect(tg); tg.connect(og.gain);
      o.connect(og); og.connect(lp); o.start(); lfo.start(); trem.start();
    });
    g.gain.linearRampToValueAtTime(0.85,ctx.currentTime+4);
    audio.current.pad=g;
  };
  const playShatter=()=>{
    const a=audio.current; if(!a.ctx) return;
    const ctx=a.ctx, t0=ctx.currentTime;
    // ASMR glass tinkle: a handful of inharmonic crystal partials, each a tiny ping that
    // decays fast and is staggered a few ms apart so it reads as glass scattering — not a
    // wooden thock, not a hiss. Master low-pass keeps it gentle on laptop speakers.
    const out=ctx.createGain(); out.gain.value=0.6;                   // audible glass — louder than the swoosh
    const lp=ctx.createBiquadFilter(); lp.type="lowpass"; lp.frequency.value=5200; lp.Q.value=0.5;
    out.connect(lp); lp.connect(ctx.destination);
    const parts=[1860,2490,3110,4200,2740];                           // glassy inharmonic ratios
    parts.forEach((f,i)=>{
      const dt=t0+i*0.012+Math.random()*0.01;                        // scatter the taps
      const o=ctx.createOscillator(); o.type="sine"; o.frequency.setValueAtTime(f*(0.99+Math.random()*0.04),dt);
      const g=ctx.createGain(); g.gain.setValueAtTime(0.0001,dt);
      g.gain.exponentialRampToValueAtTime(0.16/(1+i*0.4),dt+0.004);   // sharp attack
      g.gain.exponentialRampToValueAtTime(0.0001,dt+0.18+Math.random()*0.12);
      o.connect(g); g.connect(out); o.start(dt); o.stop(dt+0.34);
    });
    // soft shell "tss": short noise through a band-pass, rolled well down so it shimmers, not hisses
    const dur=0.22, buf=ctx.createBuffer(1,ctx.sampleRate*dur,ctx.sampleRate), d=buf.getChannelData(0);
    for(let i=0;i<d.length;i++){ const env=Math.pow(1-i/d.length,3.5); d[i]=(Math.random()*2-1)*env; }
    const src=ctx.createBufferSource(); src.buffer=buf;
    const bp=ctx.createBiquadFilter(); bp.type="bandpass"; bp.frequency.value=2600; bp.Q.value=1.1;
    const ng=ctx.createGain(); ng.gain.value=0.12;
    src.connect(bp); bp.connect(out); src.start(t0);
  };
  // positive "launch" whoosh — a rising pitch sweep (ascending = hopeful/affirmative) plus a soft
  // air swell. Master low-pass deliberately low so there's body, not the thin treath that laptop
  // speakers turn into harsh fizz.
  const playLaunch=()=>{
    const a=audio.current; if(!a.ctx) return;
    const ctx=a.ctx, t0=ctx.currentTime;
    const lp=ctx.createBiquadFilter(); lp.type="lowpass"; lp.frequency.setValueAtTime(900,t0);
    lp.frequency.exponentialRampToValueAtTime(3200,t0+0.16); lp.Q.value=0.7;      // opens as it rises
    const out=ctx.createGain(); out.gain.value=0.4; out.connect(lp); lp.connect(ctx.destination);
    // tone: triangle sweeping UP — the affirmative arc
    const o=ctx.createOscillator(); o.type="triangle";
    o.frequency.setValueAtTime(150,t0); o.frequency.exponentialRampToValueAtTime(520,t0+0.19);
    const og=ctx.createGain(); og.gain.setValueAtTime(0.0001,t0); og.gain.exponentialRampToValueAtTime(0.2,t0+0.03);
    og.gain.exponentialRampToValueAtTime(0.0001,t0+0.32); o.connect(og); og.connect(out); o.start(t0); o.stop(t0+0.36);
    // air: filtered noise swell for the "thrust", low-passed so there's no laptop fizz
    const dur=0.34, buf=ctx.createBuffer(1,ctx.sampleRate*dur,ctx.sampleRate), d=buf.getChannelData(0);
    for(let i=0;i<d.length;i++){ const x=i/d.length; d[i]=(Math.random()*2-1)*Math.sin(Math.PI*x)*0.7; }
    const src=ctx.createBufferSource(); src.buffer=buf;
    const nf=ctx.createBiquadFilter(); nf.type="lowpass"; nf.frequency.setValueAtTime(500,t0); nf.frequency.exponentialRampToValueAtTime(2200,t0+0.18);
    const ng=ctx.createGain(); ng.gain.value=0.14; src.connect(nf); nf.connect(out); src.start(t0);
  };
  const toggleSound=()=>{
    setSound(s=>{
      if(!s){initAudio(); audio.current.ctx?.resume();}
      else if(audio.current.pad) audio.current.pad.gain.linearRampToValueAtTime(0,audio.current.ctx.currentTime+0.6);
      return !s;
    });
  };

  // ── sky enable (real orientation hooks + graceful fallback) ──
  const enableSky=async()=>{
    try{
      if(typeof DeviceOrientationEvent!=="undefined" && DeviceOrientationEvent.requestPermission){
        const p=await DeviceOrientationEvent.requestPermission();
        if(p!=="granted"){ setSky(s=>({...s,on:true})); return; }
      }
      window.addEventListener("deviceorientation",(e)=>{
        const h=e.webkitCompassHeading!=null?e.webkitCompassHeading:(e.alpha!=null?360-e.alpha:null);
        if(h!=null) skyHeading.current=h;
      },true);
      navigator.geolocation?.getCurrentPosition(
        pos=>setSky(s=>({...s,lat:pos.coords.latitude,lon:pos.coords.longitude,on:true})),
        ()=>setSky(s=>({...s,on:true})),{timeout:4000}
      );
      setSky(s=>({...s,on:true}));
    }catch{ setSky(s=>({...s,on:true})); }
  };
  const toggleSky=()=>{ if(sky.on) setSky(s=>({...s,on:false})); else enableSky(); };

  // ── default-on bootstrapping: heading listener + arm ambient audio on first gesture ──
  useEffect(()=>{
    const onOri=(e)=>{ const h=e.webkitCompassHeading!=null?e.webkitCompassHeading:(e.alpha!=null?360-e.alpha:null); if(h!=null) skyHeading.current=h; };
    window.addEventListener("deviceorientation",onOri,true);
    const arm=()=>{ if(soundRef.current){ initAudio(); audio.current.ctx?.resume(); } window.removeEventListener("pointerdown",arm); window.removeEventListener("keydown",arm); };
    window.addEventListener("pointerdown",arm); window.addEventListener("keydown",arm);
    return ()=>{ window.removeEventListener("deviceorientation",onOri,true); window.removeEventListener("pointerdown",arm); window.removeEventListener("keydown",arm); };
  },[]);

  // ── projectile launchers ──
  const fireSlingshot=(aimX,aimY,power)=>{
    const ang=Math.atan2(aimY-(H-30),aimX-W/2);
    eggs.current.push({mode:"ballistic",x:W/2,y:H-30,vx:Math.cos(ang)*power*5.7,vy:Math.sin(ang)*power*5.7,
      hue:Math.random()*360,life:9,seed:Math.floor(Math.random()*99999),
      spin:Math.random()<0.5?"tumble":"spiral",spinV:0.5+Math.random()*0.7,faberge:Math.random()<0.16});
    if(soundRef.current) playLaunch();
  };
  const fireEgg=(targetId,velScale=1)=>{
    const ns=sim.current.nodes;
    const tgt=targetId?byId()[targetId]:ns[Math.floor(Math.random()*ns.length)];
    const sp=projectRef.current(tgt.p); if(!sp) return;
    eggs.current.push({mode:"homing",x:W/2,y:H-30,tx:sp.x,ty:sp.y,t:0,tStep:0.105*velScale,target:tgt.id,hue:Math.random()*360,
      seed:Math.floor(Math.random()*99999),spin:Math.random()<0.5?"tumble":"spiral",spinV:0.45+Math.random()*0.6,faberge:Math.random()<0.16});
    if(soundRef.current) playLaunch();
  };

  const basis=()=>{
    const c=cam.current;
    const cy=Math.cos(c.yaw),sy=Math.sin(c.yaw),cp=Math.cos(c.pitch),sp=Math.sin(c.pitch);
    const fwd=[sy*cp,-sp,cy*cp];
    const right=[cy,0,-sy];
    const up=[right[1]*fwd[2]-right[2]*fwd[1],right[2]*fwd[0]-right[0]*fwd[2],right[0]*fwd[1]-right[1]*fwd[0]];
    return {fwd,right,up};
  };
  const project=(world)=>{
    const c=cam.current; const {fwd,right,up}=basis();
    const d=sub(world,c.pos);
    const x=d[0]*right[0]+d[1]*right[1]+d[2]*right[2];
    const y=d[0]*up[0]+d[1]*up[1]+d[2]*up[2];
    const z=d[0]*fwd[0]+d[1]*fwd[1]+d[2]*fwd[2];
    if(z<=1) return null;
    const s=FOV/z;
    return {x:W/2+x*s,y:H/2-y*s,z,s};
  };
  const projectRef=useRef(project); projectRef.current=project;
  const screenToWorldAtDepth=(sx,sy,depth)=>{
    const c=cam.current; const {fwd,right,up}=basis();
    const k=depth/FOV;
    const ndcx=(sx-W/2)*k, ndcy=-(sy-H/2)*k;
    return add(c.pos,add(scale(fwd,depth),add(scale(right,ndcx),scale(up,ndcy))));
  };

  // ── main loop ──
  useEffect(()=>{
    let raf;
    const step=()=>{
      const C=cfgRef.current;
      const ns=sim.current.nodes;
      const map={}; ns.forEach(n=>map[n.id]=n);
      const t=performance.now()*0.001;
      const grabId=nodeDrag.current?.id;
      const feral=C.przewalski;

      for(let i=0;i<ns.length;i++){
        const n=ns[i];
        if(n.id===grabId) continue;
        let f=[0,0,0];
        for(let j=0;j<ns.length;j++){
          if(i===j) continue;
          const d=sub(n.p,ns[j].p); const r=len(d);
          f=add(f,scale(d,(C.charge*repelPulse.current)/(r*r*r)));
        }
        f=add(f,scale(n.p,feral?-0.003:-0.012));
        // home-tether: untouched node springboks to its slot; a wrangled/re-placed
        // node's `home` has MOVED, so this tethers it to the NEW equilibrium.
        f=add(f,scale(sub(n.home,n.p),feral?0.004:0.02));
        const br=Math.sin(t*0.7+i*1.3)*C.breathe*(feral?2.2:1);
        f=add(f,[br*0.4,Math.cos(t*0.6+i)*C.breathe*(feral?2.2:1),br*0.3]);
        n._f=f;
      }
      graph.current.edges.forEach(e=>{
        const a=map[e.from],b=map[e.to]; if(!a||!b) return;
        const d=sub(b.p,a.p); const r=len(d);
        const rest=120+(1-e.conf)*160;
        const sF=(r-rest)*C.spring*(0.4+e.conf);
        const dir=scale(d,sF/r);
        if(a.id!==grabId) a._f=add(a._f,dir);
        if(b.id!==grabId) b._f=add(b._f,scale(dir,-1));
      });
      ns.forEach(n=>{ if(n.id===grabId) return; n.v=scale(add(n.v,scale(n._f,0.18)),C.damp); n.p=add(n.p,n.v); });

      // camera look (pivot about offset vertical hinge handled in drag); fall fwd
      const c=cam.current;
      c.pitch=Math.max(-1.1,Math.min(1.1,c.pitch));
      const {fwd}=basis();
      // ── zoom = a straight-line dolly toward the structure (its centroid, or the
      //    selected node if you've picked one). No camera-forward curve — wheel/pinch
      //    in moves you in a dead-straight line toward the cluster, out moves away. ──
      if(Math.abs(c.vFwd)>0.01){
        let aim;
        if(map[selRef.current]) aim=map[selRef.current].p;
        else { let cen=[0,0,0]; for(let i=0;i<ns.length;i++) cen=add(cen,ns[i].p); aim=scale(cen,1/(ns.length||1)); }
        const toAim=sub(aim,c.pos), dd=len(toAim)||1, dir=scale(toAim,1/dd);
        const stepLen=c.vFwd>0?Math.min(c.vFwd,Math.max(0,dd-90)):c.vFwd; // don't punch through the far side
        c.pos=add(c.pos,scale(dir,stepLen));
        const wantYaw=Math.atan2(toAim[0],toAim[2]);
        let dY=((wantYaw-c.yaw+Math.PI)%(2*Math.PI))-Math.PI;
        c.yaw+=dY*0.04; // keep the structure gently centred as you dolly in
      }
      c.vFwd*=0.84;

      // ── flick-to-spin coast: once you let go of a Shift/right-drag, the leftover
      //    angular velocity keeps orbiting the structure and eases off like a top.
      //    Skipped while actively orbit-dragging (the drag owns the motion then).
      //    Decay per frame is the "spin coast" slider — 0.94 default. ──
      if(!(lookDrag.current && lookDrag.current.orbit) && (Math.abs(c.vYaw)>1e-4 || Math.abs(c.vPitch)>1e-4)){
        let scen=[0,0,0]; for(let i=0;i<ns.length;i++) scen=add(scen,ns[i].p); scen=scale(scen,1/(ns.length||1));
        const sh=add(c.pos,scale(sub(scen,c.pos),0.55));
        const sr=sub(c.pos,sh); const scs=Math.cos(c.vYaw),ssn=Math.sin(c.vYaw);
        c.pos=[sh[0]+(sr[0]*scs-sr[2]*ssn),c.pos[1],sh[2]+(sr[0]*ssn+sr[2]*scs)];
        c.yaw+=c.vYaw;
        c.pitch=Math.max(-1.1,Math.min(1.1,c.pitch+c.vPitch));
        c.vYaw*=C.spinDecay; c.vPitch*=C.spinDecay;
        if(Math.abs(c.vYaw)<1e-4) c.vYaw=0;
        if(Math.abs(c.vPitch)<1e-4) c.vPitch=0;
      }
      if(repelPulse.current>1.001) repelPulse.current=1+(repelPulse.current-1)*0.9; // spread pulse eases back

      // ── gentle self-right: after a beat of no manipulation, if the structure has
      //    drifted off the central zone, ease yaw/pitch (and, if it slipped behind,
      //    position) back so you can't lose it off-screen. toggle in CONTROLS. ──
      if(C.recenterIdle && !nodeDrag.current && !lookDrag.current && !charge.current.active && Math.abs(c.vFwd)<0.02 && (performance.now()-interact.current)>900){
        let cen=[0,0,0]; for(let i=0;i<ns.length;i++) cen=add(cen,ns[i].p); cen=scale(cen,1/(ns.length||1));
        const sp=project(cen);
        const offscreen = !sp || sp.x<W*0.2 || sp.x>W*0.8 || sp.y<H*0.2 || sp.y>H*0.8;
        if(offscreen){
          const toCen=sub(cen,c.pos), dd=len(toCen);
          const wantYaw=Math.atan2(toCen[0],toCen[2]);
          let dY=((wantYaw-c.yaw+Math.PI)%(2*Math.PI))-Math.PI; c.yaw+=dY*0.045;
          const wantPitch=Math.max(-1.1,Math.min(1.1,-Math.asin(Math.max(-1,Math.min(1,toCen[1]/dd)))));
          c.pitch+=(wantPitch-c.pitch)*0.045;
          if(!sp || sp.z<1){ const {fwd}=basis(); const want=sub(cen,scale(fwd,440)); c.pos=add(c.pos,scale(sub(want,c.pos),0.03)); }
        }
      }

      // charge battery
      if(charge.current.active){
        const held=performance.now()-charge.current.since;
        const power=held<1000?0:Math.min(1,(held-1000)/1400);
        const over=held>2400?(held-2400)/800:0;
        if(held>=1000) charge.current.armed=true;
        setReticle({x:lastMouse.current.x,y:lastMouse.current.y,power,over:Math.min(1.4,over)});
      }

      // eggs
      for(let i=eggs.current.length-1;i>=0;i--){
        const egg=eggs.current[i];
        if(egg.mode==="ballistic"){
          egg.x+=egg.vx; egg.y+=egg.vy; egg.vy+=0.05; egg.life-=0.0065;
          let struck=null;
          for(const n of ns){ const sp=project(n.p); if(!sp) continue; const r=Math.max(7,22*(FOV/sp.z)*0.5); if(Math.hypot(sp.x-egg.x,sp.y-egg.y)<r+5){struck=n;break;} }
          if(struck||egg.life<=0||egg.y>H+40){
            if(struck){ shatter.current[struck.id]=1; const sp=project(struck.p); if(sp) for(let s=0;s<14;s++){const a=Math.random()*Math.PI*2,sd=1+Math.random()*3;shards.current.push({x:sp.x,y:sp.y,vx:Math.cos(a)*sd,vy:Math.sin(a)*sd-1,life:1,hue:ONTO[struck.onto].c,sz:2+Math.random()*4});} if(soundRef.current) playShatter(); }
            eggs.current.splice(i,1);
          }
          continue;
        }
        egg.t+=(egg.tStep||0.15);
        if(egg.t>=1){
          shatter.current[egg.target]=1;
          const tn=map[egg.target]; const sp=project(tn.p);
          if(sp) for(let s=0;s<14;s++){const a=Math.random()*Math.PI*2,sd=1+Math.random()*3;shards.current.push({x:sp.x,y:sp.y,vx:Math.cos(a)*sd,vy:Math.sin(a)*sd-1,life:1,hue:ONTO[tn.onto].c,sz:2+Math.random()*4});}
          if(soundRef.current) playShatter();
          eggs.current.splice(i,1);
        }
      }
      for(let i=shards.current.length-1;i>=0;i--){ const sh=shards.current[i]; sh.x+=sh.vx; sh.y+=sh.vy; sh.vy+=0.18; sh.life-=0.025; if(sh.life<=0) shards.current.splice(i,1); }
      for(const id in shatter.current){ shatter.current[id]-=0.012; if(shatter.current[id]<=0) delete shatter.current[id]; }

      tick();
      raf=requestAnimationFrame(step);
    };
    raf=requestAnimationFrame(step);

    const GAME=["e","E","4","Escape"];
    const inField=(t)=>t&&(/(input|textarea|select)/i.test(t.tagName)||t.isContentEditable);
    const onKey=(e)=>{
      if(!focusRef.current) return;
      if(inField(e.target)) return;            // don't hijack typing in chat / host / model
      if(tabRef.current!=="spatial") return;   // pysanky / 411 are spatial-only
      interact.current=performance.now();
      if(GAME.includes(e.key)) e.preventDefault();
      if(e.key==="4") setJourney(j=>!j);
      if(e.key==="Escape"){ setJourney(false); setDiveInto(null); setCardPos(null); charge.current={active:false,since:0,armed:false}; setReticle(null); }
      if((e.key==="e"||e.key==="E") && !e.repeat){
        if(!lastMouse.current||(lastMouse.current.x===380&&lastMouse.current.y===260)) lastMouse.current={x:W/2,y:H/2};
        charge.current={active:true,since:performance.now(),armed:false};
        setReticle({x:lastMouse.current.x,y:lastMouse.current.y,power:0,over:0});
      }
      // ── laptop-friendly spin: arrow keys feed the same fling velocity the drag does,
      //    so the loop's coast keeps it turning and eases off. tap = a nudge, hold to
      //    wind up a steady spin, let go and it coasts down. no mouse button needed. ──
      if(e.key==="ArrowLeft"||e.key==="ArrowRight"||e.key==="ArrowUp"||e.key==="ArrowDown"){
        e.preventDefault();
        const c=cam.current, step=0.016;
        if(e.key==="ArrowRight") c.vYaw=Math.min( 0.08,c.vYaw+step);
        if(e.key==="ArrowLeft")  c.vYaw=Math.max(-0.08,c.vYaw-step);
        if(e.key==="ArrowDown")  c.vPitch=Math.min( 0.05,c.vPitch+step*0.6);
        if(e.key==="ArrowUp")    c.vPitch=Math.max(-0.05,c.vPitch-step*0.6);
      }
    };
    const onKeyUp=(e)=>{
      if(!focusRef.current) return;
      if(inField(e.target)) return;
      if(tabRef.current!=="spatial") return;
      if(e.key==="e"||e.key==="E"){
        e.preventDefault();
        const ch=charge.current; if(!ch.active) return;
        const held=performance.now()-ch.since;
        if(ch.armed){
          const power=Math.min(1,(held-1000)/1400);
          const over=held>2400?(held-2400)/800:0;
          if(over>1){ const n=2+Math.floor(Math.random()*3); for(let k=0;k<n;k++) fireSlingshot(lastMouse.current.x+(Math.random()-0.5)*180,lastMouse.current.y+(Math.random()-0.5)*180,0.6+Math.random()*0.8); }
          else fireSlingshot(lastMouse.current.x,lastMouse.current.y,0.35+power*0.9);
        } else {
          const now=performance.now();
          if(now-(eggCooldown.current||0)>=90){ eggCooldown.current=now; fireEgg(); }
        }
        charge.current={active:false,since:0,armed:false};
        setReticle(null);
      }
    };
    const onWinMove=(ev)=>{ const r=svgRef.current?.getBoundingClientRect(); if(!r) return; lastMouse.current={x:(ev.clientX-r.left)*(W/r.width),y:(ev.clientY-r.top)*(H/r.height)}; };
    window.addEventListener("keydown",onKey);
    window.addEventListener("keyup",onKeyUp);
    window.addEventListener("pointermove",onWinMove);
    window.addEventListener("mousemove",onWinMove);
    return ()=>{ cancelAnimationFrame(raf); window.removeEventListener("keydown",onKey); window.removeEventListener("keyup",onKeyUp); window.removeEventListener("pointermove",onWinMove); window.removeEventListener("mousemove",onWinMove); };
  },[tick]);

  const onWheel=useCallback((e)=>{ e.preventDefault(); cam.current.vFwd+=-e.deltaY*0.05; interact.current=performance.now(); },[]);
  // ── touch: 2 fingers pinch = straight-line zoom; 3+ fingers spreading = push the
  //    nodes apart (the "spread them out to read between" gesture). ──
  const touchSpread=(tl)=>{ let cx=0,cy=0; for(let i=0;i<tl.length;i++){cx+=tl[i].clientX;cy+=tl[i].clientY;} cx/=tl.length;cy/=tl.length; let s=0; for(let i=0;i<tl.length;i++) s+=Math.hypot(tl[i].clientX-cx,tl[i].clientY-cy); return s/tl.length; };
  const onTouchStart=(e)=>{ if(e.touches.length>=2){ touchState.current={n:e.touches.length,spread:touchSpread(e.touches)}; interact.current=performance.now(); } };
  const onTouchMove=(e)=>{
    const tl=e.touches; if(tl.length<2){ touchState.current=null; return; }
    const sp=touchSpread(tl); const prev=touchState.current?.spread??sp; const dS=sp-prev;
    if(tl.length>=3){ if(dS>0.4) repelPulse.current=Math.min(7,repelPulse.current+dS*0.05); e.preventDefault(); }
    else { cam.current.vFwd+=dS*0.05; e.preventDefault(); } // pinch out → toward the structure
    touchState.current={n:tl.length,spread:sp}; interact.current=performance.now();
  };
  const onTouchEnd=(e)=>{ touchState.current=e.touches.length>=2?{n:e.touches.length,spread:touchSpread(e.touches)}:null; };
  const evtXY=(e)=>{ const r=svgRef.current.getBoundingClientRect(); return [(e.clientX-r.left)*(W/r.width),(e.clientY-r.top)*(H/r.height)]; };
  const onDown=(e)=>{
    interact.current=performance.now();
    const [mx,my]=evtXY(e); lastMouse.current={x:mx,y:my}; const map=byId();
    let hit=null,hitD=1e9;
    for(const n of sim.current.nodes){ const sp=project(n.p); if(!sp) continue; const r=Math.max(7,22*(FOV/sp.z)*0.5); const dd=Math.hypot(sp.x-mx,sp.y-my); if(dd<r+4&&sp.z<hitD){hit=n;hitD=sp.z;} }
    if(hit){ nodeDrag.current={id:hit.id,depth:project(hit.p).z,moved:0,origin:[...hit.p],lastX:null,lastY:null,wrangle:!e.altKey}; setGrabbed(hit.id); return; }
    let eHit=null,eBest=14;
    for(const ed of graph.current.edges){ const a=project(map[ed.from]?.p),b=project(map[ed.to]?.p); if(!a||!b) continue; const d=segDist(mx,my,a.x,a.y,b.x,b.y); const aura=(0.8+ed.conf*3.4)+8; if(d<aura&&d<eBest){eBest=d;eHit=ed;} }
    if(eHit){ const depth=(project(map[eHit.from].p).z+project(map[eHit.to].p).z)/2; nodeDrag.current={id:eHit.from,depth,moved:0,origin:[...map[eHit.from].p],lastX:null,lastY:null,wrangle:true}; setGrabbed(eHit.from); return; }
    // empty space: a plain press charges a pysanky — the reticle aims at the cursor and
    // follows it freely (movement re-aims, it never cancels the shot). Pivoting the camera
    // is an explicit Shift-drag or right-drag, so aiming can't get hijacked into an orbit.
    const orbitMod = e.shiftKey || e.button===2 || (e.buttons&2)===2;
    lookDrag.current={x:e.clientX,y:e.clientY,x0:e.clientX,y0:e.clientY,moved:0,mouse:(e.pointerType?e.pointerType==="mouse":true),orbit:orbitMod};
    if(tabRef.current==="spatial" && !orbitMod){
      charge.current={active:true,since:performance.now(),armed:false,mouse:true};
      setReticle({x:mx,y:my,power:0,over:0});
    }
  };
  const onMove=(e)=>{
    const [mx,my]=evtXY(e); lastMouse.current={x:mx,y:my};
    if(charge.current.active){
      // aiming: the cursor freely re-aims the reticle (the charge loop repaints it at
      // lastMouse, updated just above). Movement no longer cancels the shot or orbits —
      // pivot is the separate Shift/right-drag path below.
      return;
    }
    if(nodeDrag.current||lookDrag.current) interact.current=performance.now();
    if(nodeDrag.current){
      const nd=nodeDrag.current; const node=byId()[nd.id];
      const lastX=nd.lastX??mx,lastY=nd.lastY??my;
      const before=screenToWorldAtDepth(lastX,lastY,nd.depth);
      let after;
      if(e.shiftKey){ nd.depth=Math.max(60,Math.min(1400,nd.depth-(my-lastY)*(nd.depth/FOV)*3)); after=screenToWorldAtDepth(lastX,my,nd.depth); }
      else after=screenToWorldAtDepth(mx,my,nd.depth);
      const wd=sub(after,before);
      if(nd.wrangle){ sim.current.nodes.forEach(p=>{p.p=add(p.p,wd);p.home=add(p.home,wd);p.v=[0,0,0];}); }
      else { node.p=add(node.p,wd); node.v=[0,0,0]; }
      nd.moved+=len(wd); nd.lastX=mx; nd.lastY=my; return;
    }
    if(lookDrag.current && lookDrag.current.orbit){
      const dx=e.clientX-lookDrag.current.x,dy=e.clientY-lookDrag.current.y;
      lookDrag.current.moved+=Math.abs(dx)+Math.abs(dy);
      lookDrag.current.x=e.clientX; lookDrag.current.y=e.clientY;
      const c=cam.current;
      const ns=sim.current.nodes; let cen=[0,0,0]; ns.forEach(n=>cen=add(cen,n.p)); cen=scale(cen,1/ns.length);
      const hinge=add(c.pos,scale(sub(cen,c.pos),0.55));
      const dYaw=dx*0.0042;
      const rel=sub(c.pos,hinge); const cs=Math.cos(dYaw),sn=Math.sin(dYaw);
      c.pos=[hinge[0]+(rel[0]*cs-rel[2]*sn),c.pos[1],hinge[2]+(rel[0]*sn+rel[2]*cs)];
      c.yaw+=dYaw;
      c.pitch=Math.max(-1.1,Math.min(1.1,c.pitch+dy*0.0016));
      // remember the fling: a smoothed trailing velocity so letting go keeps it spinning.
      // a fast drag leaves a big vYaw; a slow, deliberate turn leaves almost none.
      c.vYaw=c.vYaw*0.45+dYaw*0.55;
      c.vPitch=c.vPitch*0.45+(dy*0.0016)*0.55;
    }
  };
  const onUp=()=>{
    const nd=nodeDrag.current;
    if(nd){
      if(!nd.wrangle){
        const node=byId()[nd.id]; const disp=dist(node.p,nd.origin);
        if(disp<60) node.home=nd.origin;
        else { node.home=[...node.p]; setSel(nd.id); setReResolving(true); inferEdges(nd.id,graph.current).then(()=>setReResolving(false)); }
      }
      nodeDrag.current=null; setGrabbed(null);
    } else if(charge.current.active && charge.current.mouse){
      // release the held mouse charge → fire a pysanky aimed at the cursor.
      const held=performance.now()-charge.current.since;
      if(charge.current.armed){
        const power=Math.min(1,(held-1000)/1400);
        const over=held>2400?(held-2400)/800:0;
        if(over>1){ const n=2+Math.floor(Math.random()*3); for(let k=0;k<n;k++) fireSlingshot(lastMouse.current.x+(Math.random()-0.5)*180,lastMouse.current.y+(Math.random()-0.5)*180,0.6+Math.random()*0.8); }
        else fireSlingshot(lastMouse.current.x,lastMouse.current.y,0.35+power*0.9);
      } else {
        const now=performance.now();
        if(now-(eggCooldown.current||0)>=70){ eggCooldown.current=now; fireSlingshot(lastMouse.current.x,lastMouse.current.y,1.0); }
      }
      charge.current={active:false,since:0,armed:false}; setReticle(null);
    }
    lookDrag.current=null;
  };
  const onDoubleClick=(e)=>{
    const [mx,my]=evtXY(e);
    let hit=null,hitD=1e9;
    for(const n of sim.current.nodes){ const sp=project(n.p); if(!sp) continue; const r=Math.max(7,22*(FOV/sp.z)*0.5); if(Math.hypot(sp.x-mx,sp.y-my)<r+4&&sp.z<hitD){hit=n;hitD=sp.z;} }
    if(hit){ setSel(hit.id); setDiveInto(hit.id); fireEgg(hit.id,0.8); }
    else { fireEgg(undefined,0.8); }  // reticle-less pysanky launch on double-click, 20% slower for a readable tumble
  };
  const onClickNode=(id)=>{ if((nodeDrag.current?.moved||0)<4) setSel(id); };
  const recenter=()=>{ cam.current={pos:[0,0,-560],yaw:0,pitch:0,vYaw:0,vPitch:0,vFwd:0}; sim.current.nodes.forEach((n,i)=>{const a=(i/sim.current.nodes.length)*Math.PI*2;n.home=[Math.cos(a)*220,Math.sin(a*1.3)*120,Math.sin(a)*180];}); };
  const set=(k)=>(e)=>setCfg(c=>({...c,[k]:parseFloat(e.target.value)}));

  // ── pull model list from a host into the combo dropdown ──
  const fetchModels=async(kind)=>{
    const host=kind==="embed"?conn.embedHost:conn.chatHost;
    const key =kind==="embed"?conn.embedKey :conn.chatKey;
    const provider=kind==="embed"?conn.embedProvider:conn.chatProvider;
    setModelsBusy(s=>({...s,[kind]:true}));
    const {list,error}=await listModels(host,key,provider);
    setModels(s=>({...s,[kind]:list}));
    setModelsErr(s=>({...s,[kind]:error}));
    setModelsBusy(s=>({...s,[kind]:false}));
  };

  // ── RAG retrieval over the loaded nodes (vectors if dims match query, else lexical) ──
  const retrieveKB=async(query,k=6)=>{
    const ns=graph.current.nodes, vmap=nodeVecs.current;
    const sampleVec=ns.map(n=>vmap[n.id]).find(Boolean);
    if(sampleVec){
      try{
        const qv=await embedUnified(query,conn.embedHost,conn.embedModel,conn.embedKey,conn.embedProvider,conn.embedDim);
        if(qv.length===sampleVec.length){
          const scored=ns.filter(n=>vmap[n.id]).map(n=>({n,score:cosine(qv,vmap[n.id])})).sort((a,b)=>b.score-a.score).slice(0,k);
          if(scored.length) return scored.map(s=>({id:s.n.id,title:s.n.title,onto:s.n.onto,raw:s.n.raw,score:s.score}));
        }
      }catch{}
    }
    const toks=query.toLowerCase().split(/\W+/).filter(w=>w.length>2);
    const scored=ns.map(n=>{const hay=((n.title||"")+" "+(n.raw||"")).toLowerCase();let sc=0;toks.forEach(tk=>{if(hay.includes(tk))sc++;});return {n,score:sc};}).filter(x=>x.score>0).sort((a,b)=>b.score-a.score).slice(0,k);
    const pick=scored.length?scored:ns.slice(0,Math.min(k,ns.length)).map(n=>({n,score:0}));
    return pick.map(s=>({id:s.n.id,title:s.n.title,onto:s.n.onto,raw:s.n.raw,score:s.score}));
  };

  // ── Wikipedia (CORS-open via origin=*) — real titles/extracts, never fabricated ──
  const wikiSearch=async(q)=>{
    const u="https://en.wikipedia.org/w/api.php?action=query&list=search&format=json&origin=*&srlimit=6&srsearch="+encodeURIComponent(q);
    const r=await fetch(u); if(!r.ok) throw new Error("HTTP "+r.status);
    const j=await r.json();
    return (j.query?.search||[]).map(s=>({title:s.title,snippet:deEntity((s.snippet||"").replace(/<[^>]+>/g,"")),
      url:"https://en.wikipedia.org/wiki/"+encodeURIComponent(s.title.replace(/ /g,"_"))}));
  };
  // Wikipedia search snippets are HTML fragments full of &quot; &amp; &#160; etc. — decode them so
  // a ghost preview reads as prose, not a broken half-sentence with raw entities.
  const deEntity=(s)=>{ try{ const d=document.createElement("textarea"); d.innerHTML=s; return d.value; }catch(e){ return s; } };
  const wikiExtract=async(title,intro)=>{
    // intro=true → exintro returns ONLY the lead section (small, fast); without it the API
    // streams the entire article plaintext, which is what made the startup cache crawl.
    const u="https://en.wikipedia.org/w/api.php?action=query&prop=extracts&"+(intro?"exintro=1&":"")+"explaintext=1&redirects=1&format=json&origin=*&titles="+encodeURIComponent(title);
    const r=await fetch(u); if(!r.ok) return ""; const j=await r.json();
    const pages=j.query?.pages||{}; const first=Object.values(pages)[0]; return (first&&first.extract)||"";
  };
  // ── lead-paragraph cache: fetch a node's real opening passage (intro only → small + fast).
  //    Marks bodyFetch done on SUCCESS *or* FAILURE so the "Fetching…" spinner can never hang
  //    forever on a CORS/offline block — it falls back to the factoid instead.
  const ensureBody=useCallback(async(node)=>{
    if(!node||!node.url||node.bodied) return;
    if(bodyFetch[node.id]) return;                          // already loading or done
    setBodyFetch(s=>({...s,[node.id]:"loading"}));
    try{
      const slug=decodeURIComponent(String(node.url).split("/wiki/")[1]||node.title).replace(/_/g," ");
      const ext=await wikiExtract(slug,true);
      if(ext){
        const lead=ext.split("\n").map(s=>s.trim()).filter(Boolean).slice(0,2).join("\n\n").slice(0,900);
        const gn=graph.current.nodes.find(x=>x.id===node.id); if(gn){gn.raw=lead;gn.bodied=true;}
        const sn=sim.current&&sim.current.nodes.find(x=>x.id===node.id); if(sn){sn.raw=lead;sn.bodied=true;}
        setGraphVer(v=>v+1);
        // best-effort: embed the REAL lead so the seed graph supports semantic RAG (not just keyword).
        // gated on the embed host; silently skipped if unreachable — never fabricates a vector.
        if(normBase(conn.embedHost)&&conn.embedModel&&!nodeVecs.current[node.id]){
          try{ nodeVecs.current[node.id]=await embedUnified(lead||node.title,conn.embedHost,conn.embedModel,conn.embedKey,conn.embedProvider,conn.embedDim); }catch(e){/* host down → stays keyword-searchable */}
        }
      }
    }catch(e){ /* CORS / offline → keep factoid fallback, never fabricated */ }
    finally{ setBodyFetch(s=>({...s,[node.id]:"done"})); }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  },[bodyFetch]);
  // startup: warm the lead paragraph of each seed topic concurrently (only the random seed graph)
  const seedCached=useRef(false);
  useEffect(()=>{
    if(seedCached.current) return; seedCached.current=true;
    if(graph.current!==SEED) return;
    SEED.nodes.forEach(n=>ensureBody(n));
  // eslint-disable-next-line react-hooks/exhaustive-deps
  },[]);
  // diving into any linked node (seed or ghost) lazily fetches its real intro — also the retry path
  // if the startup batch was blocked, and what replaces a ghost's broken search-snippet with prose.
  useEffect(()=>{
    if(!diveInto) return;
    const dn=sim.current.nodes.find(n=>n.id===diveInto)||graph.current.nodes.find(n=>n.id===diveInto);
    if(dn) ensureBody(dn);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  },[diveInto]);

  // remove any not-yet-added ghost baubles from the field
  const clearGhosts=(ids)=>{
    const gset=new Set(ids); if(!gset.size) return;
    graph.current.nodes=graph.current.nodes.filter(n=>!gset.has(n.id));
    sim.current.nodes=sim.current.nodes.filter(n=>!gset.has(n.id));
    graph.current.edges=graph.current.edges.filter(e=>!gset.has(e.from)&&!gset.has(e.to));
    gset.forEach(id=>delete nodeVecs.current[id]);
  };

  // ── voice notes: record real mic audio, store as a data-URL on the annotation
  const startVoice=async(nodeId,text)=>{
    if(!navigator.mediaDevices||!window.MediaRecorder){ setSelVoice({err:"Mic recording isn't available here — try the native shell.",nodeId,text}); return; }
    try{
      const stream=await navigator.mediaDevices.getUserMedia({audio:true});
      const rec=new MediaRecorder(stream); const chunks=[]; voiceRec.current=rec;
      rec.ondataavailable=e=>{ if(e.data&&e.data.size) chunks.push(e.data); };
      rec.onstop=()=>{
        stream.getTracks().forEach(t=>t.stop());
        const blob=new Blob(chunks,{type:rec.mimeType||"audio/webm"});
        const fr=new FileReader();
        fr.onload=()=>{ setAnnos(a=>[...a,{ts:Date.now()+Math.random(),kind:"voice",text,audio:fr.result,nodeId}]); };
        fr.readAsDataURL(blob);
        setSelVoice(null); setSelMenu(null); setSelAct(null);
      };
      rec.start(); setSelVoice({nodeId,text,t0:performance.now()});
    }catch(e){ setSelVoice({err:"Mic permission denied or blocked in this preview.",nodeId,text}); }
  };
  const stopVoice=()=>{ try{ voiceRec.current&&voiceRec.current.state!=="inactive"&&voiceRec.current.stop(); }catch(e){} };
  const cancelVoice=()=>{ try{ if(voiceRec.current){ voiceRec.current.onstop=null; voiceRec.current.state!=="inactive"&&voiceRec.current.stop(); } }catch(e){} setSelVoice(null); setSelMenu(null); setSelAct(null); };

  // the search strategist: RAG first, Wikipedia for the gaps, LLM narrates what's where
  const SEARCH_SYSTEM="You are the search strategist inside ThinkSpace, the user's personal knowledge graph. They ran a search. You are given (A) nodes ALREADY in their graph that matched, and (B) candidate topics from Wikipedia NOT yet in their graph. Narrate briefly and concretely: 1) what was found in THEIR graph — cite node titles in [brackets]; 2) what's available to ADD from Wikipedia — name them, and note they appear as dim 'ghost' baubles they can right-click → add; 3) suggest one or two alternate search angles. Be concise. Never invent facts.";

  const runSearch=async(q0)=>{
    const q=(q0!=null?q0:chat.q).trim(); if(!q||chat.busy) return;
    clearGhosts(search.ghosts);
    setChat(c=>({...c,q:"",busy:true,msgs:[...c.msgs,{role:"user",content:"🔍 "+q},{role:"assistant",content:""}]}));
    // 1 — RAG over the graph
    let hits=[]; try{ hits=await retrieveKB(q,6); }catch{}
    const strong=hits.filter(h=>h.score>0.12||hits.length<=3).slice(0,6);
    // 2 — Wikipedia for what's missing
    let wiki=[],wikiErr="";
    try{ wiki=await wikiSearch(q); }catch(e){ wikiErr=e&&e.message||String(e); }
    const have=new Set(graph.current.nodes.map(n=>(n.title||"").toLowerCase()));
    const novel=w=>!have.has(w.title.toLowerCase())&&!seenGhosts.current.has(w.title.toLowerCase());
    let fresh=wiki.filter(novel);
    // if the user has their OWN graph, don't keep circling the same query — pick the least-connected
    // (frontier) real node and extend a few suggestions outward from there.
    if(graph.current!==SEED){
      const real=graph.current.nodes.filter(n=>!n.ghost);
      if(real.length){
        const deg={}; graph.current.edges.forEach(e=>{deg[e.from]=(deg[e.from]||0)+1;deg[e.to]=(deg[e.to]||0)+1;});
        const frontier=real.slice().sort((a,b)=>(deg[a.id]||0)-(deg[b.id]||0))[0];
        if(frontier){ try{ const more=await wikiSearch(frontier.title); fresh=fresh.concat(more.filter(novel)); }catch(e){} }
      }
    }
    // de-dup by title and cap; remember what we've shown so the next search advances instead of repeating
    const seenThis=new Set(); fresh=fresh.filter(w=>{const k=w.title.toLowerCase();if(seenThis.has(k))return false;seenThis.add(k);return true;}).slice(0,5);
    fresh.forEach(w=>seenGhosts.current.add(w.title.toLowerCase()));
    // 3 — inject ghosts into the field
    const ghostIds=[];
    fresh.forEach((w,i)=>{
      const id="ghost-"+i+"-"+Math.abs(hashStr(w.title));
      const a=(i/Math.max(1,fresh.length))*Math.PI*2, R=300+i*14;
      const p=[Math.cos(a)*R,(Math.sin(a*1.7))*120,Math.sin(a)*R*0.8];
      const onto=ONTO_KEYS[Math.abs(hashStr(w.title))%ONTO_KEYS.length];
      graph.current.nodes.push({id,title:w.title,onto,raw:w.snippet,url:w.url,source:"Wikipedia",created:Date.now(),ghost:true});
      sim.current.nodes.push({id,title:w.title,onto,raw:w.snippet,url:w.url,source:"Wikipedia",created:Date.now(),ghost:true,p:[...p],v:[0,0,0],home:[...p]});
      ghostIds.push(id);
    });
    setSearch({q,hits:strong.map(h=>h.id),ghosts:ghostIds,adding:[],err:wikiErr});
    setGraphVer(v=>v+1);
    // 4 — narrate (LLM plans + reports); falls back to a plain summary with no chat host
    const ghostList=fresh.map(w=>`• ${w.title} — ${w.snippet}`).join("\n");
    if(!normBase(conn.chatHost)){
      const lines=[];
      lines.push(strong.length?`In your graph: ${strong.map(h=>"["+h.title+"]").join(", ")} lit up gold.`:"Nothing in your graph matched.");
      lines.push(fresh.length?`From Wikipedia (dim ghost baubles — right-click → add): ${fresh.map(w=>w.title).join(", ")}.`:(wikiErr?`Wikipedia lookup failed: ${wikiErr}`:"No new Wikipedia topics."));
      lines.push("Set a Chat host in ⊞ data for a narrated strategy.");
      setChat(c=>{const m=[...c.msgs];m[m.length-1]={role:"assistant",content:lines.join("\n\n"),sources:strong,ghosts:fresh.map((w,i)=>({...w,id:ghostIds[i]}))};return {...c,busy:false,msgs:m};});
      return;
    }
    try{
      const context=["LIVE GRAPH (what's already on the canvas):\n"+graphDigest(), ...strong.map(n=>`[${n.title}]\n${(n.raw||"").slice(0,900)}`)];
      if(ghostList) context.push("WIKIPEDIA CANDIDATES (not yet in graph):\n"+ghostList);
      await chatStream({host:conn.chatHost,model:conn.chatModel,key:conn.chatKey,provider:conn.chatProvider,system:SEARCH_SYSTEM,context,
        messages:[{role:"user",content:"I searched: "+q}],
        onToken:(tok)=>setChat(c=>{const m=[...c.msgs];const last=m[m.length-1];m[m.length-1]={...last,content:(last.content||"")+tok};return {...c,msgs:m};})});
      setChat(c=>{const m=[...c.msgs];m[m.length-1]={...m[m.length-1],sources:strong,ghosts:fresh.map((w,i)=>({...w,id:ghostIds[i]}))};return {...c,busy:false,msgs:m};});
    }catch(err){
      setChat(c=>{const m=[...c.msgs];m[m.length-1]={role:"assistant",content:"⚠ search narration failed: "+err.message+" (the gold/ghost baubles are still live in the map).",sources:strong,ghosts:fresh.map((w,i)=>({...w,id:ghostIds[i]}))};return {...c,busy:false,msgs:m};});
    }
  };

  // materialize a ghost: fetch the real extract, embed it, wire similarity edges, flip ghost→real
  const addGhost=async(id)=>{
    const node=byId()[id]; if(!node||!node.ghost) return;
    if(!normBase(conn.embedHost)||!conn.embedModel){ setSearch(s=>({...s,err:"Set an embedding host + model (⊞ data) before adding to your RAG."})); return; }
    setSearch(s=>({...s,adding:[...s.adding,id],err:""}));
    try{
      let raw=node.raw||"";
      try{ const ex=await wikiExtract(node.title); if(ex) raw=ex; }catch{}
      const vec=await embedUnified(raw||node.title,conn.embedHost,conn.embedModel,conn.embedKey,conn.embedProvider,conn.embedDim);
      nodeVecs.current[id]=vec;
      graph.current.nodes.forEach(o=>{ if(o.id===id||o.ghost) return; const v=nodeVecs.current[o.id]; if(!v||v.length!==vec.length) return;
        const c=cosine(vec,v); if(c>=ingest.thresh) graph.current.edges.push({from:id,to:o.id,rel:"related",conf:Math.min(0.99,c)}); });
      const gn=graph.current.nodes.find(n=>n.id===id); if(gn){gn.ghost=false;gn.raw=raw;}
      const sn=sim.current.nodes.find(n=>n.id===id); if(sn){sn.ghost=false;sn.raw=raw;}
      setSearch(s=>({...s,ghosts:s.ghosts.filter(g=>g!==id),hits:[...s.hits.filter(h=>h!==id),id],adding:s.adding.filter(a=>a!==id)}));
      setGraphVer(v=>v+1);
    }catch(e){ setSearch(s=>({...s,adding:s.adding.filter(a=>a!==id),err:"add failed: "+(e&&e.message||e)})); }
  };

  const sendChat=async(preset)=>{
    const q=(preset!=null?preset:chat.q).trim(); if(!q||chat.busy) return;
    const history=chat.msgs.filter(m=>m.content).map(m=>({role:m.role,content:m.content}));
    setChat(c=>({...c,q:"",busy:true,msgs:[...c.msgs,{role:"user",content:q},{role:"assistant",content:""}]}));
    if(!normBase(conn.chatHost)){
      setChat(c=>{const m=[...c.msgs];m[m.length-1]={role:"assistant",content:"No chat host set. Open ⊞ data → CHAT, pick a provider (Ollama / OpenAI-compat), and enter the host — an Ollama ip:port (e.g. 192.168.0.41:11434), or an OpenAI-compatible base URL + key (OpenAI, OpenRouter, z.ai/GLM…) — then ↻ models to pick one."};return {...c,busy:false,msgs:m};});
      return;
    }
    try{
      const top=await retrieveKB(q);
      const context=["LIVE GRAPH (authoritative for structure — every node, its links, and what's isolated):\n"+graphDigest(), ...top.map(n=>`[${n.title}]\n${(n.raw||"").slice(0,1200)}`)];
      const full=await chatStream({host:conn.chatHost,model:conn.chatModel,key:conn.chatKey,provider:conn.chatProvider,system:KB_SYSTEM,context,
        messages:[...history,{role:"user",content:q}],
        onToken:(tok)=>setChat(c=>{const m=[...c.msgs];const last=m[m.length-1];m[m.length-1]={...last,content:(last.content||"")+tok};return {...c,msgs:m};})});
      const applied=applyAgentActions(full);
      setChat(c=>{const m=[...c.msgs];const last=m[m.length-1];
        let content=(last.content||"").replace(/```(?:thinkspace|json)?\s*[\s\S]*?```/gi,"").trim();  // hide the raw action JSON from the transcript
        if(applied.length) content+=(content?"\n\n":"")+"✓ canvas updated — "+applied.join(" · ");
        m[m.length-1]={...last,content,sources:top};
        return {...c,busy:false,msgs:m};});
    }catch(err){
      setChat(c=>{const m=[...c.msgs];m[m.length-1]={role:"assistant",content:"⚠ chat failed: "+err.message+" — check the Chat host/model in ⊞ data (and that the server permits this origin; native Tauri bypasses CORS)."};return {...c,busy:false,msgs:m};});
    }
  };

  // ── AGENT SIGHT: a compact, current snapshot of the whole canvas, fed to the model
  //    so it can reason about structure (what's linked, what's isolated) — not just text. ──
  const graphDigest=()=>{
    const ns=graph.current.nodes.filter(n=>!n.ghost), es=graph.current.edges;
    const deg={},nbr={}; es.forEach(e=>{deg[e.from]=(deg[e.from]||0)+1;deg[e.to]=(deg[e.to]||0)+1;(nbr[e.from]=nbr[e.from]||[]).push(e.to);(nbr[e.to]=nbr[e.to]||[]).push(e.from);});
    const byId={}; ns.forEach(n=>byId[n.id]=n);
    const iso=ns.filter(n=>!deg[n.id]);
    const hist={}; ns.forEach(n=>hist[n.onto]=(hist[n.onto]||0)+1);
    const L=[`${ns.length} nodes, ${es.length} links · types: ${Object.entries(hist).map(([k,v])=>k+"×"+v).join(", ")||"—"}`];
    L.push(iso.length?`ISOLATED (no links): ${iso.map(n=>'"'+n.title+'"').join(", ")}`:"ISOLATED: none — every node is connected.");
    L.push("NODES (title · type · #links → neighbors):");
    [...ns].sort((a,b)=>(deg[b.id]||0)-(deg[a.id]||0)).slice(0,70).forEach(n=>{
      const near=(nbr[n.id]||[]).slice(0,6).map(id=>byId[id]&&byId[id].title).filter(Boolean);
      L.push(`• "${n.title}" · ${n.onto} · ${deg[n.id]||0}${near.length?" → "+near.map(t=>'"'+t+'"').join(", "):" → (none)"}`);
    });
    if(ns.length>70) L.push(`…and ${ns.length-70} more nodes (showing the most-connected).`);
    return L.join("\n");
  };

  // ── AGENT HANDS: resolve a node by id or title, place/wire nodes, and apply a parsed action list. ──
  const findNode=(ref)=>{ if(ref==null) return null; const s=String(ref).trim(),low=s.toLowerCase();
    return graph.current.nodes.find(n=>n.id===s)||graph.current.nodes.find(n=>(n.title||"").toLowerCase()===low)||graph.current.nodes.find(n=>low.length>2&&(n.title||"").toLowerCase().includes(low))||null; };
  const ringPos=()=>{ const a=Math.random()*Math.PI*2,r=150+Math.random()*130; return [Math.cos(a)*r,(Math.random()-0.5)*180,Math.sin(a)*r]; };
  const placeNode=(node,nearId)=>{
    let p=ringPos(); const s=nearId&&sim.current.nodes.find(n=>n.id===nearId);
    if(s&&s.p) p=[s.p[0]+(Math.random()-0.5)*70,s.p[1]+(Math.random()-0.5)*70,s.p[2]+(Math.random()-0.5)*70];
    const base={id:node.id,title:node.title,onto:node.onto,raw:node.raw||"",html:"",url:"",media:[],source:"agent",created:Date.now()};
    graph.current.nodes.push({...base});
    sim.current.nodes.push({...base,vec:null,p,v:[0,0,0],home:[...p]});
    if(normBase(conn.embedHost)&&conn.embedModel) embedUnified((node.title||"")+"\n"+(node.raw||""),conn.embedHost,conn.embedModel,conn.embedKey,conn.embedProvider,conn.embedDim).then(v=>{if(v&&v.length)nodeVecs.current[node.id]=v;}).catch(()=>{});
  };
  const addEdgeSafe=(a,b,rel)=>{ if(!a||!b||a===b) return false;
    if(graph.current.edges.some(e=>(e.from===a&&e.to===b)||(e.from===b&&e.to===a))) return false;
    const okRel=Object.keys(REL); graph.current.edges.push({from:a,to:b,rel:okRel.includes(rel)?rel:"related",conf:0.82}); return true; };
  const nearestNode=(id)=>{ const vmap=nodeVecs.current,v=vmap[id]; const ns=graph.current.nodes.filter(n=>n.id!==id&&!n.ghost);
    if(v){ let best=null,bs=-1; ns.forEach(n=>{const w=vmap[n.id]; if(w&&w.length===v.length){const sc=cosine(v,w); if(sc>bs){bs=sc;best=n;}}}); if(best) return best; }
    const me=graph.current.nodes.find(n=>n.id===id); const toks=((me&&me.title||"")+" "+(me&&me.raw||"")).toLowerCase().split(/\W+/).filter(w=>w.length>3);
    let best=null,bs=0; ns.forEach(n=>{const hay=((n.title||"")+" "+(n.raw||"")).toLowerCase(); let sc=0; toks.forEach(t=>{if(hay.includes(t))sc++;}); if(sc>bs){bs=sc;best=n;}}); return best; };
  const applyAgentActions=(text)=>{
    const blocks=[...String(text).matchAll(/```(?:thinkspace|json)?\s*([\s\S]*?)```/gi)].map(m=>m[1]);
    let acts=null; for(let i=blocks.length-1;i>=0&&!acts;i--){ try{ const o=JSON.parse(blocks[i]); if(Array.isArray(o)) acts=o; else if(o&&Array.isArray(o.actions)) acts=o.actions; }catch{} }
    if(!acts||!acts.length) return [];
    const okOnto=Object.keys(ONTO); const done=[]; let changed=false;
    for(const a of acts){ const op=String(a.op||a.action||a.type||"").toLowerCase().replace(/[_\s-]/g,"");
      try{
        if(op==="link"||op==="addedge"||op==="connect"||op==="join"){ const A=findNode(a.a!=null?a.a:a.from!=null?a.from:a.source),B=findNode(a.b!=null?a.b:a.to!=null?a.to:a.target);
          if(A&&B&&addEdgeSafe(A.id,B.id,a.rel||a.relation)){ done.push(`linked “${A.title}” → “${B.title}”`); changed=true; } }
        else if(op==="unlink"||op==="removeedge"||op==="disconnect"){ const A=findNode(a.a!=null?a.a:a.from),B=findNode(a.b!=null?a.b:a.to);
          if(A&&B){ const n0=graph.current.edges.length; graph.current.edges=graph.current.edges.filter(e=>!((e.from===A.id&&e.to===B.id)||(e.from===B.id&&e.to===A.id))); if(graph.current.edges.length<n0){ done.push(`unlinked “${A.title}” ✕ “${B.title}”`); changed=true; } } }
        else if(op==="addnode"||op==="create"||op==="add"||op==="newnode"){ const title=String(a.title||a.name||"").trim(); if(!title){} else if(findNode(title)){ const N=findNode(title),near=findNode(a.near||a.linkto||a.parent); if(near&&addEdgeSafe(N.id,near.id,a.rel)){ done.push(`linked “${N.title}” → “${near.title}”`); changed=true; } }
          else { const onto=okOnto.includes(a.type||a.onto)?(a.type||a.onto):okOnto[Math.abs(hashStr(title))%okOnto.length]; const id="a"+Date.now().toString(36)+Math.floor(Math.random()*1e3); const near=findNode(a.near||a.linkto||a.parent); placeNode({id,title,onto,raw:a.body||a.raw||a.text||""},near&&near.id); if(near) addEdgeSafe(id,near.id,a.rel); done.push(`added “${title}”${near?` → “${near.title}”`:""}`); changed=true; } }
        else if(op==="removenode"||op==="delete"||op==="remove"||op==="deletenode"){ const N=findNode(a.name!=null?a.name:a.id!=null?a.id:a.title!=null?a.title:a.node);
          if(N){ graph.current.nodes=graph.current.nodes.filter(n=>n.id!==N.id); sim.current.nodes=sim.current.nodes.filter(n=>n.id!==N.id); graph.current.edges=graph.current.edges.filter(e=>e.from!==N.id&&e.to!==N.id); delete nodeVecs.current[N.id]; done.push(`removed “${N.title}”`); changed=true; } }
        else if(op==="relabel"||op==="rename"){ const N=findNode(a.name!=null?a.name:a.id!=null?a.id:a.node); const t=String(a.title||a.to||a.newtitle||"").trim();
          if(N&&t){ const old=N.title; N.title=t; const s=sim.current.nodes.find(n=>n.id===N.id); if(s)s.title=t; done.push(`renamed “${old}” → “${t}”`); changed=true; } }
        else if(op==="settype"||op==="retype"){ const N=findNode(a.name!=null?a.name:a.id!=null?a.id:a.node); const ty=a.to||a.type||a.onto;
          if(N&&okOnto.includes(ty)){ N.onto=ty; const s=sim.current.nodes.find(n=>n.id===N.id); if(s)s.onto=ty; done.push(`set “${N.title}” type → ${ty}`); changed=true; } }
        else if(op==="joinisolated"||op==="autojoin"){ const N=findNode(a.name!=null?a.name:a.id!=null?a.id:a.node); const best=N&&nearestNode(N.id);
          if(best&&addEdgeSafe(N.id,best.id,"related")){ done.push(`auto-joined “${N.title}” → “${best.title}”`); changed=true; } }
      }catch{}
    }
    if(changed) setGraphVer(v=>v+1);
    return done;
  };

  // citation chip → fly-to in space, or open in reader
  const onSource=(id)=>{
    if(!graph.current.nodes.find(n=>n.id===id)) return;
    if(tabRef.current==="reader"){ setReaderId(id); }
    else { setSel(id); cam.current.vFwd+=6; interact.current=performance.now(); }
  };
  const onShot=()=>{ if(tabRef.current==="spatial"&&svgRef.current) snapshotSvg(svgRef.current); else snapshotEl(stageRef.current); };

  const map=byId();
  const projected=sim.current.nodes.map(n=>({...n,sp:project(n.p)})).filter(n=>n.sp);
  const selEdges=graph.current.edges.filter(e=>e.from===sel||e.to===sel);
  const selNode=map[sel];
  const t=performance.now()*0.001;
  const G=cfg.glow;
  const searchHits=new Set(search.hits);

  return (
    <div ref={rootRef} tabIndex={0}
      onMouseEnter={()=>{rootRef.current?.focus();setFocused(true);}}
      onFocus={()=>setFocused(true)} onBlur={()=>setFocused(false)}
      style={big
        ? {fontFamily:"ui-sans-serif, system-ui",color:"#dfe7ff",background:"#05060f",border:"none",outline:"none",position:"fixed",inset:0,zIndex:9000,borderRadius:0,overflow:"hidden",display:"flex",flexDirection:"column",boxShadow:"none"}
        : {fontFamily:"ui-sans-serif, system-ui",color:"#dfe7ff",background:"#05060f",borderRadius:16,overflow:"hidden",border:`1px solid ${focused?"#2a4a7a":"#1b2140"}`,width:"100%",maxWidth:"100%",margin:"0 auto",outline:"none"}}>
      <div style={{display:"flex",alignItems:"center",gap:8,rowGap:8,flexWrap:"wrap",padding:"10px 14px",borderBottom:"1px solid #161b38",background:"linear-gradient(90deg,#0a0d22,#070818)"}}>
        <div style={{width:9,height:9,borderRadius:9,background:"#7ee8fa",boxShadow:"0 0 10px #7ee8fa"}}/>
        <strong style={{letterSpacing:1,fontSize:13}}>THINKSPACE</strong>
        <span style={{fontSize:11,color:"#7a86b8"}}>6DoF semantic navigator</span>
        <div style={{display:"flex",gap:2,marginLeft:6,background:"#0a0e22",border:"1px solid #1f2a55",borderRadius:9,padding:2}}>
          {[["spatial","✦ space"],["reader","▤ reader"]].map(([k,lab])=>(
            <button key={k} onClick={()=>setTab(k)} style={{border:"none",borderRadius:7,padding:"4px 11px",fontSize:11,cursor:"pointer",fontWeight:tab===k?700:400,color:tab===k?"#05060f":"#9ad8ff",background:tab===k?"#7ee8fa":"transparent",transition:"all .15s"}}>{lab}</button>
          ))}
        </div>
        <div style={{flex:1}}/>
        <button onClick={()=>setIngest(s=>({...s,open:!s.open}))} style={{...btn,color:graphVer>0?"#05060f":"#9ad8ff",background:graphVer>0?"#ffd479":"rgba(126,232,250,0.08)"}}>⊞ data</button>
        <button onClick={()=>setChat(c=>({...c,open:!c.open}))} style={{...btn,color:chat.open?"#05060f":"#9ad8ff",background:chat.open?"#7cffc4":"rgba(126,232,250,0.08)"}} title="chat with your knowledge (RAG)">✦ chat</button>
        <button onClick={()=>setNotes(v=>!v)} style={{...btn,color:notes?"#05060f":"#9ad8ff",background:notes?"#ffd479":"rgba(126,232,250,0.08)"}} title="notebook — every highlight & note, collated">✑ notes{annos.length?" · "+annos.filter(a=>a.kind==="highlight"||a.kind==="annotate").length:""}</button>
        <button onClick={onShot} style={btn} title="screenshot the viewport">📷</button>
        <button onClick={()=>setBig(v=>!v)} style={{...btn,color:big?"#05060f":"#9ad8ff",background:big?"#9db4ff":"rgba(126,232,250,0.08)"}} title={big?"restore size":"maximize"}>{big?"⤡ restore":"⤢"}</button>
        {tab==="spatial"&&<button onClick={recenter} style={btn}>⟲ recenter</button>}
        {tab==="spatial"&&<button onClick={toggleSky} style={{...btn,color:sky.on?"#05060f":"#9ad8ff",background:sky.on?"#9db4ff":"rgba(126,232,250,0.08)"}} title="real constellations for your heading — squint">✶ sky</button>}
        <button onClick={toggleSound} style={{...btn,color:sound?"#05060f":"#9ad8ff",background:sound?"#7cffc4":"rgba(126,232,250,0.08)"}}>{sound?"♪ on":"♪ off"}</button>
        {tab==="spatial"&&<button onClick={()=>setJourney(j=>!j)} style={{...btn,color:journey?"#05060f":"#9ad8ff",background:journey?"#7ee8fa":"rgba(126,232,250,0.08)"}}>411</button>}
        {tab==="spatial"&&<button onClick={()=>setEdgeMode(m=>m==="vectors"?"flow":"vectors")} style={btn}>edge: {edgeMode}</button>}
      </div>

      <div ref={stageRef} style={big
        ? {position:"relative",flex:1,minHeight:0,width:"100%"}
        : {position:"relative",aspectRatio:"760 / 520"}}>
        {ingest.open&&(
          <div style={{position:"absolute",top:0,right:0,height:"100%",width:360,maxWidth:"94%",zIndex:15,overflowY:"auto",padding:16,background:"rgba(8,11,28,0.97)",backdropFilter:"blur(18px)",borderLeft:"1px solid #1f2a55",boxShadow:"-14px 0 50px #000a",animation:"tsTray .22s cubic-bezier(.2,.9,.3,1)"}}>
            <style>{`@keyframes tsTray{from{transform:translateX(102%)}to{transform:translateX(0)}}`}</style>
            <div style={{display:"flex",alignItems:"center",marginBottom:10}}>
              <span style={{fontSize:11,letterSpacing:1,color:"#ffd479",textShadow:"0 0 8px #ffd479"}}>⊞ DATA · CONNECTIONS · INGEST</span>
              <div style={{flex:1}}/>
              <span onClick={()=>setIngest(s=>({...s,open:false}))} style={{cursor:"pointer",color:"#7a86b8",fontSize:14}}>✕</span>
            </div>
            <p style={{fontSize:11,color:"#9aa6d8",lineHeight:1.6,margin:"0 0 12px"}}>
              Point <b style={{color:"#7cffc4"}}>embeddings</b> + <b style={{color:"#7cffc4"}}>chat</b> at an Ollama <i>ip:port</i> (key optional — Ollama cloud/turbo) or any <b>OpenAI-compatible</b> base URL — OpenAI, OpenRouter, <b>z.ai/GLM</b>, vLLM, LM Studio (these all expose <code style={{color:"#7ee8fa"}}>/v1/chat/completions</code>). Paste the URL however the docs give it — a trailing <code style={{color:"#7ee8fa"}}>/v1</code> or <code style={{color:"#7ee8fa"}}>/v1/chat/completions</code> is stripped for you. Use a <b>LAN IP</b>, not <code style={{color:"#7ee8fa"}}>localhost</code> (that points at this machine, where Ollama isn't running). A real embed host is <b>required</b> — nothing is faked. Then ↻ to pull the model list.
            </p>
            <div style={{fontSize:10,letterSpacing:1,color:"#7ee8fa",textShadow:"0 0 8px #7ee8fa",margin:"0 0 8px"}}>CONNECTIONS</div>
            <ConnFields kind="embed" conn={conn} setConn={setConn} models={models.embed} busy={modelsBusy.embed} err={modelsErr.embed} onList={()=>fetchModels("embed")} title="embeddings" accent="#7cffc4" phHost="192.168.0.41:11434 (required — no blank/demo)" phModel="qwen3-embedding:8b"/>
            <ConnFields kind="chat" conn={conn} setConn={setConn} models={models.chat} busy={modelsBusy.chat} err={modelsErr.chat} onList={()=>fetchModels("chat")} title="chat / inference" accent="#7ee8fa" phHost="192.168.0.41:11434 · OpenAI / OpenRouter / z.ai URL" phModel="gemma4:e2b"/>
            <div style={{padding:"8px 10px",borderRadius:10,background:"rgba(124,255,196,0.03)",border:"1px solid #182146",marginBottom:12}}>
              <div style={{display:"flex",alignItems:"center",gap:8,marginBottom:6}}>
                <span style={{fontSize:10,letterSpacing:1,color:"#7cffc4",textTransform:"uppercase"}}>embedding dimensions</span>
                <div style={{flex:1}}/>
                <input type="number" min="0" max="4096" step="64" value={conn.embedDim||0}
                  onChange={e=>setConn(c=>({...c,embedDim:Math.max(0,Math.min(4096,parseInt(e.target.value)||0))}))}
                  style={{...inS,width:72,textAlign:"right",padding:"2px 6px",fontSize:11}}/>
                <span style={{fontSize:10,color:"#7a86b8",minWidth:46,textAlign:"right"}}>{conn.embedDim?conn.embedDim+"-d":"native"}</span>
              </div>
              <input type="range" min="0" max="4096" step="64" value={conn.embedDim||0}
                onChange={e=>setConn(c=>({...c,embedDim:parseInt(e.target.value)}))}
                style={{width:"100%",accentColor:"#7cffc4"}}/>
              <div style={{fontSize:9,color:"#5d6796",marginTop:4,lineHeight:1.4}}>0 = model native (qwen3-embedding:8b → 4096). Lower truncates + renormalizes (Matryoshka). Re-ingest after changing.</div>
            </div>

            <div style={{fontSize:10,letterSpacing:1,color:"#7ee8fa",textShadow:"0 0 8px #7ee8fa",margin:"2px 0 8px"}}>SETTINGS</div>
            <div style={{padding:"8px 10px",borderRadius:10,background:"rgba(126,232,250,0.03)",border:"1px solid #182146",marginBottom:10}}>
              <div style={{fontSize:9,color:"#7a86b8",lineHeight:1.5,marginBottom:8}}>
                Storage: <b style={{color:PERSIST.mode==="browser storage"?"#7cffc4":"#ff9b8a"}}>{PERSIST.mode}</b>.
                {PERSIST.mode==="browser storage"
                  ? " Hosts, keys & models are saved here automatically — enter them once."
                  : " This host walls off browser storage, so settings only last the session — use Export to keep them and Import to restore."}
              </div>
              <div style={{display:"flex",gap:6}}>
                <button onClick={()=>download("thinkspace-settings.json",JSON.stringify({v:1,conn,stores},null,2),"application/json")} style={{...btn,flex:1}}>⬇ export settings</button>
                <label style={{...btn,flex:1,textAlign:"center",cursor:"pointer"}}>⬆ import
                  <input type="file" accept=".json,application/json" style={{display:"none"}} onChange={async e=>{
                    const f=e.target.files&&e.target.files[0]; if(!f) return;
                    try{ const j=JSON.parse(await f.text());
                      if(j.conn) setConn(c=>({...c,...j.conn}));
                      if(j.stores) setStores(s=>({...s,...j.stores}));
                      setStoreMsg("✓ settings imported"); }
                    catch(err){ setStoreMsg("import failed — "+(err.message||err)); }
                    e.target.value="";
                  }}/>
                </label>
              </div>
            </div>

            <div style={{fontSize:10,letterSpacing:1,color:"#7ee8fa",textShadow:"0 0 8px #7ee8fa",margin:"2px 0 8px"}}>STORES · EXPORT</div>
            <div style={{padding:"8px 10px",borderRadius:10,background:"rgba(126,232,250,0.03)",border:"1px solid #182146",marginBottom:10}}>
              <div style={{fontSize:9,color:"#7a86b8",lineHeight:1.5,marginBottom:8}}>The graph is JSON, so it round-trips to disk or to your <b style={{color:"#7cffc4"}}>Qdrant</b> / <b style={{color:"#7cffc4"}}>Neo4j</b> boxes for multi-GB corpora. (Browser→server is CORS-gated; runs clean from the native shell.)</div>
              <div style={{display:"flex",gap:6,marginBottom:10}}>
                <button onClick={exportGraphJson} style={{...btn,flex:1,color:"#05060f",background:"#7cffc4",border:"none"}}>⬇ graph.json</button>
              </div>
              <div style={{fontSize:9,letterSpacing:1,color:"#9ad8ff",margin:"0 0 5px"}}>QDRANT</div>
              <input value={stores.qdrantUrl} onChange={e=>setStores(s=>({...s,qdrantUrl:e.target.value}))} placeholder="http://192.168.0.252:6333" style={{...inS,marginBottom:5}}/>
              <div style={{display:"flex",gap:5,marginBottom:5}}>
                <input value={stores.qdrantColl} onChange={e=>setStores(s=>({...s,qdrantColl:e.target.value}))} placeholder="collection" style={{...inS,flex:1}}/>
                <input type="password" value={stores.qdrantKey} onChange={e=>setStores(s=>({...s,qdrantKey:e.target.value}))} placeholder="api-key (opt)" style={{...inS,flex:1}}/>
              </div>
              <div style={{display:"flex",gap:5,marginBottom:10}}>
                <button onClick={qdrantPush} style={{...btn,flex:1}}>↑ push vectors</button>
                <button onClick={qdrantPull} style={{...btn,flex:1}}>↓ pull</button>
              </div>
              <div style={{fontSize:9,letterSpacing:1,color:"#9ad8ff",margin:"0 0 5px"}}>NEO4J</div>
              <input value={stores.neoUrl} onChange={e=>setStores(s=>({...s,neoUrl:e.target.value}))} placeholder="http://192.168.0.46:7474" style={{...inS,marginBottom:5}}/>
              <div style={{display:"flex",gap:5,marginBottom:5}}>
                <input value={stores.neoDb} onChange={e=>setStores(s=>({...s,neoDb:e.target.value}))} placeholder="database" style={{...inS,width:78}}/>
                <input value={stores.neoUser} onChange={e=>setStores(s=>({...s,neoUser:e.target.value}))} placeholder="user" style={{...inS,flex:1}}/>
                <input type="password" value={stores.neoPass} onChange={e=>setStores(s=>({...s,neoPass:e.target.value}))} placeholder="password" style={{...inS,flex:1}}/>
              </div>
              <div style={{display:"flex",gap:5}}>
                <button onClick={neo4jPush} style={{...btn,flex:1}}>↑ push graph</button>
                <button onClick={neo4jPull} style={{...btn,flex:1}}>↓ pull</button>
              </div>
              {storeMsg&&<div style={{fontSize:9.5,color:storeMsg.startsWith("✓")?"#7cffc4":storeMsg.includes("failed")||storeMsg.includes("error")||storeMsg.includes("HTTP")?"#ff9b8a":"#ffd479",marginTop:8,lineHeight:1.4,wordBreak:"break-word"}}>{storeMsg}</div>}
              {storeBusy&&<button onClick={cancelStore} style={{...btn,width:"100%",marginTop:8,color:"#ffb4a8",borderColor:"#5a2a2a",background:"rgba(166,64,54,0.12)"}}>✕ cancel transfer</button>}
            </div>
            <div style={{fontSize:10,letterSpacing:1,color:"#5d6796",textAlign:"center",margin:"4px 0 10px"}}>— ingest / load —</div>
            <div style={{display:"flex",gap:8,marginBottom:8}}>
              <div style={{flex:1}}>
                <label style={{display:"block",fontSize:10,color:"#7a86b8",marginBottom:4}}>view budget · {load.maxNodes>=4000?"all":load.maxNodes}</label>
                <input type="range" min="50" max="4000" step="50" value={load.maxNodes} onChange={e=>setLoad(s=>({...s,maxNodes:parseInt(e.target.value)}))} style={{width:"100%",accentColor:"#7cffc4"}}/>
              </div>
              <div style={{flex:1}}>
                <label style={{display:"block",fontSize:10,color:"#7a86b8",marginBottom:4}}>min edge · {load.minConf.toFixed(2)}</label>
                <input type="range" min="0" max="0.9" step="0.05" value={load.minConf} onChange={e=>setLoad(s=>({...s,minConf:parseFloat(e.target.value)}))} style={{width:"100%",accentColor:"#7cffc4"}}/>
              </div>
            </div>
            <label style={{display:"block",textAlign:"center",padding:"12px",border:"1.5px dashed #7cffc4",borderRadius:10,cursor:"pointer",fontSize:12,color:"#7cffc4",background:"rgba(124,255,196,0.05)",marginBottom:8}}>
              ⬆ load graph JSON (any name)
              <input type="file" accept=".json,application/json" style={{display:"none"}} onChange={e=>e.target.files[0]&&loadGraphJson(e.target.files[0])}/>
            </label>
            {load.shown>0&&<div style={{fontSize:10,color:load.shown<load.total?"#ffd479":"#7cffc4",textAlign:"center",marginBottom:8}}>graph holds all {load.total} nodes · drawing {load.shown}{load.shown<load.total?" (most-connected — raise view budget or lower min-edge to draw more; nothing is lost)":" (all of them)"}</div>}
            {load.err&&<div style={{fontSize:10,color:"#ff7a7a",textAlign:"center",marginBottom:8}}>{load.err}</div>}
            <label style={{display:"block",fontSize:10,color:"#7a86b8",marginBottom:4}}>edge threshold · {ingest.thresh.toFixed(2)} (cosine)</label>
            <input type="range" min="0.3" max="0.95" step="0.01" value={ingest.thresh} onChange={e=>setIngest(s=>({...s,thresh:parseFloat(e.target.value)}))} style={{width:"100%",accentColor:"#ffd479",marginBottom:12}}/>
            {ingest.busy ? (
              <div style={{padding:"6px 0"}}>
                <div style={{fontSize:11,color:"#ffd479",textAlign:"center",marginBottom:4}}>◌ {ingest.count}/{ingest.total} sources</div>
                {ingest.phase&&<div style={{fontSize:9.5,color:"#9aa6d8",textAlign:"center",marginBottom:6,lineHeight:1.4,wordBreak:"break-word"}}>{ingest.phase}…</div>}
                <div style={{height:4,borderRadius:4,background:"#1b2140",overflow:"hidden",marginBottom:8}}><div style={{height:"100%",width:(ingest.total?Math.round(ingest.count/ingest.total*100):0)+"%",background:"#ffd479",transition:"width .2s"}}/></div>
                <button onClick={()=>{ingestCancel.current=true;}} style={{...btn,width:"100%",color:"#ffb4a8",borderColor:"#5a2a2a",background:"rgba(166,64,54,0.12)"}}>✕ cancel ingest</button>
              </div>
            ) : (<>
              {/* rich doc ingestor: a document node + embedded passage nodes, added to the graph */}
              <label style={{display:"block",textAlign:"center",padding:"16px 12px",border:"1.5px dashed #ffd479",borderRadius:10,cursor:"pointer",fontSize:12,color:"#ffd479",background:"rgba(255,212,121,0.06)",marginBottom:8}}>
                📄 ingest documents — PDF · TXT · MD · HTML
                <div style={{fontSize:9.5,color:"#b9a567",marginTop:4,fontWeight:400}}>bulk-load papers &amp; articles · each becomes a doc node + embedded passages · adds to your graph</div>
                <input type="file" accept=".pdf,.txt,.md,.markdown,.html,.htm,.json,.csv,.tex,.rtf" multiple style={{display:"none"}} onChange={e=>{ingestDocs(e.target.files);e.target.value="";}}/>
              </label>
              <div style={{padding:"8px 10px",borderRadius:10,background:"rgba(126,232,250,0.04)",border:"1px solid #182146",marginBottom:8}}>
                <div style={{fontSize:9,letterSpacing:1,color:"#9ad8ff",marginBottom:5}}>OR FETCH URLs (one per line)</div>
                <textarea value={ingest.urls} onChange={e=>setIngest(s=>({...s,urls:e.target.value}))} rows={2} placeholder="https://arxiv.org/abs/…&#10;https://example.com/paper.pdf" style={{...inS,width:"100%",resize:"vertical",fontFamily:"inherit",marginBottom:6}}/>
                <button onClick={()=>ingestUrls(ingest.urls)} style={{...btn,width:"100%",color:"#05060f",background:"#7ee8fa",border:"none"}}>↓ fetch &amp; ingest URLs</button>
                <div style={{fontSize:8.5,color:"#5d6796",marginTop:5,lineHeight:1.4}}>URL fetch is CORS-gated in the browser preview — runs clean in the native shell. PDFs &amp; pages both work.</div>
              </div>
              <label style={{display:"block",textAlign:"center",padding:"10px",border:"1.5px dashed #2a3a6a",borderRadius:10,cursor:"pointer",fontSize:11,color:"#9ad8ff",background:"rgba(126,232,250,0.03)"}}>
                or a folder of .md files (replaces graph)
                <input type="file" webkitdirectory="" directory="" multiple style={{display:"none"}} onChange={e=>ingestFiles(e.target.files)}/>
              </label>
            </>)}
            {ingest.err&&<div style={{fontSize:10,color:"#ffb4a8",marginTop:8,lineHeight:1.4,background:"rgba(166,64,54,0.1)",border:"1px solid #5a2a2a",borderRadius:8,padding:"6px 8px"}}>{ingest.err}</div>}
            {graphVer>0&&<div style={{fontSize:10,color:"#7cffc4",textAlign:"center",marginTop:8}}>✓ live graph: {graph.current.nodes.length} nodes · {graph.current.edges.length} edges</div>}
          </div>
        )}
        {tab==="spatial" && (<>
        <svg ref={svgRef} xmlns="http://www.w3.org/2000/svg" width="100%" viewBox={`0 0 ${W} ${H}`} preserveAspectRatio={big?"none":"xMidYMid meet"}
          onWheel={onWheel} onMouseDown={onDown} onMouseMove={onMove} onMouseUp={onUp} onMouseLeave={onUp} onDoubleClick={onDoubleClick}
          onTouchStart={onTouchStart} onTouchMove={onTouchMove} onTouchEnd={onTouchEnd} onContextMenu={(e)=>e.preventDefault()}
          style={{display:"block",width:"100%",flex:big?"1 1 auto":"none",minHeight:big?0:"auto",height:big?"100%":"auto",touchAction:"none",cursor:nodeDrag.current||lookDrag.current?"grabbing":"grab",background:"radial-gradient(ellipse at 50% 40%, #0b1030 0%, #05060f 70%)"}}>
          <defs>
            <radialGradient id="halo" cx="50%" cy="50%" r="50%"><stop offset="0%" stopColor="#fff" stopOpacity="0.9"/><stop offset="100%" stopColor="#fff" stopOpacity="0"/></radialGradient>
            {Object.entries(ONTO).map(([k,v])=>(
              <radialGradient key={k} id={`sph-${k}`} cx="36%" cy="32%" r="72%">
                <stop offset="0%" stopColor="#ffffff" stopOpacity="0.95"/><stop offset="16%" stopColor={v.c} stopOpacity="0.9"/>
                <stop offset="58%" stopColor={v.c} stopOpacity="0.42"/><stop offset="100%" stopColor="#04060f" stopOpacity="1"/>
              </radialGradient>
            ))}
            <radialGradient id="vig" cx="50%" cy="50%" r="50%"><stop offset="0%" stopColor="#000" stopOpacity="0"/><stop offset="62%" stopColor="#000" stopOpacity="0"/><stop offset="100%" stopColor="#000" stopOpacity="0.55"/></radialGradient>
            {Object.entries(REL).map(([k,v])=>(<marker key={k} id={`arrow-${k}`} markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L7,3 L0,6 Z" fill={v.c}/></marker>))}
          </defs>

          {Array.from({length:60}).map((_,i)=>{const x=(i*137.5)%W,y=(i*89.3)%H;return <circle key={i} cx={x} cy={y} r={(i%3)*0.4+0.3} fill="#8893c8" opacity={0.25}/>;})}

          {/* orientation-aware constellations: drift with heading, faint by design */}
          {sky.on && (()=>{
            const hdg=skyHeading.current||0;
            const CON=[
              {n:"Orion",az:20,stars:[[0,0],[8,4],[16,2],[6,14],[14,16],[10,24],[4,30]],links:[[0,1],[1,2],[1,3],[2,4],[3,4],[3,5],[4,5],[5,6]]},
              {n:"Ursa Major",az:120,stars:[[0,0],[10,2],[20,0],[28,6],[26,16],[16,18],[8,14]],links:[[0,1],[1,2],[2,3],[3,4],[4,5],[5,6],[6,0]]},
              {n:"Cassiopeia",az:220,stars:[[0,8],[8,0],[16,8],[24,2],[32,10]],links:[[0,1],[1,2],[2,3],[3,4]]},
              {n:"Scorpius",az:300,stars:[[0,0],[6,4],[12,6],[18,10],[20,18],[16,26],[22,30]],links:[[0,1],[1,2],[2,3],[3,4],[4,5],[5,6]]},
            ];
            return CON.map((c,ci)=>{
              let rel=((c.az-hdg+540)%360)-180;
              if(Math.abs(rel)>90) return null;
              const cx=W/2+(rel/90)*(W*0.46), cy=60+(ci*90)%(H-160);
              return (
                <g key={c.n} opacity={0.16} style={{filter:"drop-shadow(0 0 2px #aab8ff)"}}>
                  {c.links.map(([a,b],li)=>(<line key={li} x1={cx+c.stars[a][0]} y1={cy+c.stars[a][1]} x2={cx+c.stars[b][0]} y2={cy+c.stars[b][1]} stroke="#9fb0ff" strokeWidth={0.5}/>))}
                  {c.stars.map(([sx,sy],si)=>(<circle key={si} cx={cx+sx} cy={cy+sy} r={si===0?1.3:0.9} fill="#dfe7ff"/>))}
                  <text x={cx+4} y={cy-4} fontSize={6} fill="#7a86b8" opacity={0.7} style={{letterSpacing:1}}>{c.n}</text>
                </g>
              );
            });
          })()}

          {graph.current.edges.map((e,i)=>{
            const a=project(map[e.from]?.p),b=project(map[e.to]?.p); if(!a||!b) return null;
            const active=e.from===sel||e.to===sel; const col=REL[e.rel].c; const wgt=0.8+e.conf*3.4;
            const mx=(a.x+b.x)/2,my=(a.y+b.y)/2; const nx=-(b.y-a.y),ny=(b.x-a.x); const nl=Math.hypot(nx,ny)||1;
            const bow=Math.sin(t*1.2+i)*14*(cfg.breathe/0.5); const cx=mx+(nx/nl)*bow,cy=my+(ny/nl)*bow;
            return (
              <path key={i} d={`M${a.x},${a.y} Q${cx},${cy} ${b.x},${b.y}`} fill="none" stroke={col} strokeWidth={wgt} strokeLinecap="round"
                opacity={active?1:0.26} markerEnd={edgeMode==="vectors"?`url(#arrow-${e.rel})`:undefined}
                strokeDasharray={edgeMode==="flow"?"6 9":"none"} style={{filter:active?`drop-shadow(0 0 ${6*G}px ${col})`:"none"}}>
                {edgeMode==="flow"&&active&&<animate attributeName="stroke-dashoffset" from="15" to="0" dur="0.7s" repeatCount="indefinite"/>}
              </path>
            );
          })}

          {projected.sort((a,b)=>b.sp.z-a.sp.z).map((n,idx)=>{
            const r=Math.max(5,22*(FOV/n.sp.z)*0.5); const col=ONTO[n.onto].c;
            const isSel=n.id===sel,isHov=n.id===hover,isGrab=n.id===grabbed;
            const sh=shatter.current[n.id]||0; const grow=sh>0?(1-sh)*0.9+0.1:1;
            const pulse=1+Math.sin(t*1.6+idx*0.9)*0.05*(cfg.breathe/0.5);
            const rr=r*pulse*(isGrab?1.18:1)*grow; const wob=Math.sin(t*2+idx)*0.04;
            const isHit=searchHits.has(n.id), isGhost=!!n.ghost, isAdding=search.adding.includes(n.id);
            return (
              <g key={n.id} opacity={isGhost?0.46:1} onClick={()=>onClickNode(n.id)} onMouseEnter={()=>setHover(n.id)} onMouseLeave={()=>setHover(null)}
                 onContextMenu={(e)=>{e.preventDefault();setCtx({x:e.clientX,y:e.clientY,id:n.id,ghost:isGhost,title:n.title});}} style={{cursor:isGhost?"copy":"grab"}}>
                {isHit&&<circle cx={n.sp.x} cy={n.sp.y} r={rr*3.1} fill="none" stroke="#ffd479" strokeWidth={2.2} opacity={0.45+0.4*Math.sin(t*3+idx)} style={{filter:"drop-shadow(0 0 9px #ffd479)",pointerEvents:"none"}}/>}
                {isGhost&&<circle cx={n.sp.x} cy={n.sp.y} r={rr*2} fill="none" stroke="#9fb0ff" strokeWidth={1.3} strokeDasharray="3 6" opacity={isAdding?0.9:0.55} style={{filter:"drop-shadow(0 0 5px #3a4a7a)",pointerEvents:"none"}}>{isAdding&&<animateTransform attributeName="transform" type="rotate" from={`0 ${n.sp.x} ${n.sp.y}`} to={`360 ${n.sp.x} ${n.sp.y}`} dur="1.1s" repeatCount="indefinite"/>}</circle>}
                {isGhost&&<text x={n.sp.x} y={n.sp.y-rr-7} textAnchor="middle" fontSize={9.5} fill="#9fb0ff" opacity={0.85} style={{pointerEvents:"none",letterSpacing:0.5}}>{isAdding?"⋯ adding":"＋ wiki"}</text>}
                <circle cx={n.sp.x} cy={n.sp.y} r={rr*2.6} fill="url(#halo)" opacity={((isSel||isGrab)?0.55:0.2)*G} style={{mixBlendMode:"screen"}}/>
                <circle cx={n.sp.x} cy={n.sp.y} r={rr*1.6} fill={col} opacity={((isSel||isGrab)?0.36:0.13)*G} style={{filter:`blur(${rr*0.45}px)`,mixBlendMode:"screen"}}/>
                <ellipse cx={n.sp.x} cy={n.sp.y} rx={rr*(1+wob)} ry={rr*(1-wob)} fill={`url(#sph-${n.onto})`} stroke={col} strokeWidth={isSel||isGrab?1.8:0.8} strokeOpacity={0.9} style={{filter:`drop-shadow(0 0 ${(isGrab?22:isSel?16:7)*G}px ${col})`}}/>
                {cfg.vignette&&<ellipse cx={n.sp.x} cy={n.sp.y} rx={rr*(1+wob)} ry={rr*(1-wob)} fill="url(#vig)" style={{mixBlendMode:"multiply"}}/>}
                <ellipse cx={n.sp.x-rr*0.04} cy={n.sp.y-rr*0.04} rx={rr*0.97} ry={rr*0.97} fill="none" stroke={col} strokeWidth={rr*0.09} strokeOpacity={0.5} strokeDasharray={`${rr*2.6} ${rr*8}`} strokeDashoffset={rr*1.3} style={{mixBlendMode:"screen"}}/>
                <ellipse cx={n.sp.x-rr*0.34} cy={n.sp.y-rr*0.36} rx={rr*0.26} ry={rr*0.2} fill="#fff" opacity={0.8} style={{filter:`blur(${rr*0.08}px)`}}/>
                {isGrab&&(()=>{ const cg=rr*1.42; const spin=t*22%360; const ring=(rx,ry,rot)=>(<ellipse cx={n.sp.x} cy={n.sp.y} rx={rx} ry={ry} transform={`rotate(${rot} ${n.sp.x} ${n.sp.y})`} fill="none" stroke={col} strokeWidth={1.1} strokeOpacity={0.85} style={{filter:`drop-shadow(0 0 4px ${col})`}}/>);
                  return (<g style={{pointerEvents:"none"}}>{ring(cg,cg*0.34,spin)}{ring(cg,cg*0.34,spin+60)}{ring(cg,cg*0.34,spin+120)}<circle cx={n.sp.x} cy={n.sp.y} r={cg} fill="none" stroke={col} strokeWidth={1.1} strokeOpacity={0.9} style={{filter:`drop-shadow(0 0 5px ${col})`}}/>{ring(cg*0.34,cg,spin*0.5)}{[0,60,120,180,240,300].map(a=>{const rad=(a+spin)*Math.PI/180;return <circle key={a} cx={n.sp.x+Math.cos(rad)*cg} cy={n.sp.y+Math.sin(rad)*cg} r={1.8} fill={col} style={{filter:`drop-shadow(0 0 3px ${col})`}}/>;})}</g>);
                })()}
                {(isHov||isSel||isGrab||r>9)&&<text x={n.sp.x} y={n.sp.y+rr+14} textAnchor="middle" fontSize={Math.max(9,Math.min(13,r*0.7))} fill="#e6ecff" style={{pointerEvents:"none",textShadow:"0 0 6px #000"}}>{n.title}</text>}
              </g>
            );
          })}

          {eggs.current.map((egg,i)=>{
            let x,y,trailPts;
            if(egg.mode==="ballistic"){ x=egg.x; y=egg.y; trailPts=[0,1,2,3].map(k=>({x:egg.x-egg.vx*k*1.4,y:egg.y-egg.vy*k*1.4,k})); }
            else { const ease=egg.t*egg.t; x=W/2+(egg.tx-W/2)*ease; y=(H-30)+(egg.ty-(H-30))*ease-Math.sin(egg.t*Math.PI)*60; trailPts=[0,1,2,3].map(k=>{const tt=Math.max(0,egg.t-k*0.04);return{x:W/2+(egg.tx-W/2)*(tt*tt),y:(H-30)+(egg.ty-(H-30))*(tt*tt)-Math.sin(tt*Math.PI)*60,k};}); }
            const age=egg.mode==="ballistic"?(9-egg.life):egg.t*4; const spinV=egg.spinV||0.5; const rot=age*spinV*60;
            const tumble=age*spinV*1.3; const barrel=0.66+0.34*Math.abs(Math.cos(tumble));
            const sd=egg.seed||0; const band1=(sd%5)+2,band2=((sd>>3)%4)+1,dots=(sd>>6)%6+3; const hue2=(egg.hue+40+(sd%80))%360;
            return (
              <g key={`egg${i}`}>
                {trailPts.map(tp=>(<circle key={tp.k} cx={tp.x} cy={tp.y} r={2.4-tp.k*0.5} fill={egg.faberge?"#ffe7a3":"#e2e8ff"} opacity={0.32-tp.k*0.07} style={{filter:"blur(0.8px)"}}/>))}
                <ellipse cx={x} cy={y+19} rx={11} ry={3.5} fill="#000" opacity={0.24} style={{filter:"blur(2px)"}}/>
                <g transform={`translate(${x},${y}) rotate(${rot}) scale(${barrel}, 1)`}>
                  <defs>
                    <radialGradient id={`eg${i}`} cx="36%" cy="26%" r="82%">
                      <stop offset="0%" stopColor={`hsl(${egg.hue},78%,88%)`}/>
                      <stop offset="42%" stopColor={`hsl(${egg.hue},64%,58%)`}/>
                      <stop offset="100%" stopColor={`hsl(${egg.hue},66%,28%)`}/>
                    </radialGradient>
                  </defs>
                  <ellipse rx={13} ry={18} fill={`url(#eg${i})`} stroke={egg.faberge?"#f6d77a":`hsl(${egg.hue},52%,26%)`} strokeWidth={egg.faberge?1.4:0.7} style={{filter:"drop-shadow(0 1px 3px rgba(0,0,0,0.5))"}}/>
                  <ellipse cx={-4.6} cy={-8} rx={3.4} ry={5.4} fill="#ffffff" opacity={0.55} style={{filter:"blur(1.6px)"}}/>
                  {Array.from({length:band1}).map((_,b)=>{const yy=-18+(b+1)*(36/(band1+1));return <line key={b} x1={-13} y1={yy} x2={13} y2={yy} stroke={`hsl(${hue2},80%,75%)`} strokeWidth={1} opacity={0.8}/>;})}
                  <path d="M0,-18 V18" stroke="#fff8" strokeWidth={0.8}/>
                  {Array.from({length:band2}).map((_,b)=>{const yy=-10+b*8;return <path key={b} d={`M-8,${yy} q8,6 16,0`} stroke="#ffffffcc" strokeWidth={0.8} fill="none"/>;})}
                  {Array.from({length:dots}).map((_,b)=>{const a=(b/dots)*Math.PI*2+sd;return <circle key={b} cx={Math.cos(a)*7} cy={Math.sin(a)*10} r={1.4} fill={`hsl(${hue2},90%,85%)`}/>;})}
                  {egg.faberge&&(<g>
                    {[-9,-3,3,9].map((gx,b)=>(<line key={`la${b}`} x1={gx-6} y1={-15} x2={gx+6} y2={15} stroke="#f4d77a" strokeWidth={0.7} opacity={0.85}/>))}
                    {[-9,-3,3,9].map((gx,b)=>(<line key={`lb${b}`} x1={gx-6} y1={15} x2={gx+6} y2={-15} stroke="#f4d77a" strokeWidth={0.7} opacity={0.85}/>))}
                    <ellipse rx={13} ry={3.2} fill="none" stroke="#f4d77a" strokeWidth={1.1} opacity={0.9}/>
                    {[-11,-3.6,3.6,11].map((cy,b)=>(<circle key={`cb${b}`} cx={0} cy={cy} r={1.8} fill={`hsl(${hue2},92%,72%)`} stroke="#f4d77a" strokeWidth={0.5}/>))}
                    <path d="M-3,-18 L0,-23 L3,-18 Z" fill="#f4d77a"/>
                    <circle cx={0} cy={-23.4} r={1.4} fill={`hsl(${hue2},92%,78%)`} stroke="#f4d77a" strokeWidth={0.4}/>
                  </g>)}
                </g>
              </g>
            );
          })}
          {shards.current.map((sh,i)=>(<polygon key={`sh${i}`} points={`${sh.x},${sh.y} ${sh.x+sh.sz},${sh.y+sh.sz*0.6} ${sh.x+sh.sz*0.3},${sh.y+sh.sz*1.4}`} fill={sh.hue} opacity={sh.life*0.9} style={{filter:`drop-shadow(0 0 3px ${sh.hue})`}}/>))}

          {reticle&&(()=>{
            const over=reticle.over||0; const qv=over>0?over*6:0;
            const jx=reticle.x+(Math.random()-0.5)*qv, jy=reticle.y+(Math.random()-0.5)*qv;
            const glitch=over>1; const pwHue=90-reticle.power*90;
            const ringCol=glitch?`hsl(${Math.random()*360},100%,60%)`:`hsl(${pwHue},90%,58%)`;
            return (
              <g style={{pointerEvents:"none"}} opacity={glitch&&Math.random()>0.7?0.4:1}>
                <line x1={W/2} y1={H-30} x2={jx} y2={jy} stroke={ringCol} strokeWidth={1+reticle.power*2} strokeDasharray={glitch?"2 3":"3 4"} opacity={0.7}/>
                <circle cx={jx} cy={jy} r={16+reticle.power*10} fill="none" stroke={ringCol} strokeWidth={2} style={{filter:`drop-shadow(0 0 ${6+over*8}px ${ringCol})`}}/>
                <circle cx={jx} cy={jy} r={2.5} fill="#fff"/>
                {[[-22,0,-9,0],[9,0,22,0],[0,-22,0,-9],[0,9,0,22]].map((l,k)=>(<line key={k} x1={jx+l[0]} y1={jy+l[1]} x2={jx+l[2]} y2={jy+l[3]} stroke="#fff" strokeWidth={1.5}/>))}
                {glitch&&Array.from({length:5}).map((_,k)=>(<rect key={k} x={jx-30+Math.random()*60} y={jy-30+Math.random()*60} width={10+Math.random()*30} height={1+Math.random()*3} fill={`hsl(${Math.random()*360},100%,60%)`} opacity={0.6}/>))}
                {glitch&&<text x={jx} y={jy-28} textAnchor="middle" fontSize={9} fill="#ff5a5a" style={{filter:"drop-shadow(0 0 4px #ff0000)"}}>{Math.random()>0.5?"⚠ OVERCHARGE":"█▓▒ T4RG3T_L0$T"}</text>}
                <g transform={`translate(${jx+30}, ${jy-24})`}>
                  <rect x={-2} y={-2} width={12} height={52} rx={2} fill="rgba(0,0,0,0.5)" stroke="#2a3a6a" strokeWidth={1}/>
                  {Array.from({length:8}).map((_,k)=>{const cp=(8-k)/8;const lit=reticle.power>=cp-0.06;const co=glitch&&Math.random()>0.5;return <rect key={k} x={0} y={k*6} width={8} height={5} rx={1} fill={lit?(co?`hsl(${Math.random()*360},100%,60%)`:`hsl(${90-cp*90},90%,55%)`):"#1a2240"} opacity={lit?(0.6+(over>0?Math.random()*0.4:0.4)):1}/>;})}
                </g>
              </g>
            );
          })()}
        </svg>

        {grabbed&&<div style={{position:"absolute",top:14,left:"50%",transform:"translateX(-50%)",padding:"6px 14px",borderRadius:20,background:"rgba(126,232,250,0.12)",border:"1px solid #2a4a7a",fontSize:11,color:"#9ad8ff",backdropFilter:"blur(8px)"}}>handling <b>{map[grabbed]?.title}</b> · drag hauls the field · Alt-grab plucks one · Shift = depth</div>}

        {diveInto&&(()=>{
          const dn=map[diveInto]; const col=ONTO[dn.onto].c; const links=graph.current.edges.filter(e=>e.from===diveInto||e.to===diveInto);
          // floating, DRAGGABLE card — no full-screen overlay (the field stays visible & reachable
          // behind it), no acrylic blur, just a dim solid panel. Grab the header to move it aside.
          const cardStyle=cardPos
            ? {left:cardPos.x,top:cardPos.y}
            : {right:20,top:54};
          return (
            <div style={{position:"absolute",zIndex:16,width:430,maxHeight:chat.open?"56%":"82%",overflowY:"auto",borderRadius:18,background:"rgba(7,10,24,0.96)",border:`1px solid ${col}55`,boxShadow:`0 18px 60px #000c, 0 0 0 1px ${col}22`,animation:"tsZoom 0.28s cubic-bezier(.2,.9,.3,1)",...cardStyle}}>
              <style>{`@keyframes tsZoom{from{opacity:0;transform:translateY(-8px) scale(0.98)}to{opacity:1;transform:none}}@keyframes pulse{0%,100%{opacity:1}50%{opacity:0.3}}`}</style>
                <div
                  onPointerDown={(e)=>{
                    if(e.target.closest("button"))return;          // don't start a drag from the ✕
                    const card=e.currentTarget.parentElement, par=card.offsetParent;
                    const cr=card.getBoundingClientRect(), pr=par.getBoundingClientRect();
                    cardDrag.current={ox:e.clientX-cr.left,oy:e.clientY-cr.top,pl:pr.left,pt:pr.top};
                    e.currentTarget.setPointerCapture(e.pointerId);
                  }}
                  onPointerMove={(e)=>{ const d=cardDrag.current; if(!d)return; setCardPos({x:e.clientX-d.pl-d.ox,y:e.clientY-d.pt-d.oy}); }}
                  onPointerUp={(e)=>{ cardDrag.current=null; try{e.currentTarget.releasePointerCapture(e.pointerId);}catch(_){} }}
                  style={{position:"relative",height:118,borderRadius:"18px 18px 0 0",overflow:"hidden",cursor:"move",background:`radial-gradient(circle at 78% 22%, ${col}, #060912 80%)`,touchAction:"none"}}>
                  <div style={{position:"absolute",inset:0,background:"linear-gradient(to top, #060912 6%, #060912dd 28%, transparent 74%)"}}/>
                  <div style={{position:"absolute",top:8,left:"50%",transform:"translateX(-50%)",width:34,height:4,borderRadius:4,background:"#ffffff33"}}/>
                  <div style={{position:"absolute",bottom:12,left:18,right:60}}>
                    <div style={{fontSize:10,letterSpacing:1,color:col,textShadow:`0 0 8px ${col}`}}>{ONTO[dn.onto].label.toUpperCase()}{dn.url?" · LINKED SOURCE":dn.raw?" · DOCUMENT":" · SEMANTIC NODE"}</div>
                    <div style={{fontSize:21,fontWeight:800,color:"#fff",textShadow:"0 2px 12px #000"}}>{dn.title}</div>
                  </div>
                  <button onClick={()=>{setDiveInto(null);setCardPos(null);}} style={{position:"absolute",top:10,right:12,background:"rgba(0,0,0,0.45)",border:`1px solid ${col}55`,borderRadius:8,color:"#fff",cursor:"pointer",fontSize:11,padding:"3px 9px",zIndex:2}}>✕ surface</button>
                </div>
                <div style={{padding:"16px 20px"}}
                  onMouseUp={(ev)=>{
                    const s=window.getSelection&&window.getSelection();
                    const txt=s&&s.toString().trim();
                    if(txt&&txt.length>0){
                      const rect=ev.currentTarget.getBoundingClientRect();
                      setSelAct(null); setSelVoice(null); setSelDraft(null);
                      setSelMenu({x:ev.clientX-rect.left+20,y:ev.clientY-rect.top+96,text:txt});
                    } else { setSelMenu(null); setSelAct(null); }
                  }}>
                  {(()=>{
                    const nbrObjs=links.map(e=>{const oid=e.from===diveInto?e.to:e.from;return{id:oid,title:map[oid]?.title};}).filter(o=>o.title);
                    const rels=[...new Set(links.map(e=>REL[e.rel]?.label).filter(Boolean))];
                    const dim=(nodeVecs.current[dn.id]||[]).length;
                    const loading=!dn.raw&&!!dn.url&&bodyFetch[dn.id]!=="done";  // spinner ONLY while in flight — settles to factoid on CORS fail
                    // factoid as JSX so the neighbour names are real links into those nodes, not a tease
                    const factoid=(muted)=>{const lc=muted?"#9aa3d0":col;const shown=nbrObjs.slice(0,5);return (<>
                      {`${dn.title} is classified as ${ONTO[dn.onto]?.label||dn.onto} in your knowledge-space`}
                      {dim?`, embedded as a ${dim}-dimensional vector`:""}
                      {nbrObjs.length
                        ? (<>{`. It connects to ${nbrObjs.length} neighbour${nbrObjs.length>1?"s":""}${rels.length?` through ${rels.join(", ")} links`:""}: `}
                            {shown.map((o,i)=>(<span key={o.id} onClick={(ev)=>{ev.stopPropagation();setSelMenu(null);setDiveInto(o.id);}} style={{color:lc,cursor:"pointer",textDecoration:"underline",textDecorationColor:lc+"66",textUnderlineOffset:2}}>{o.title}{i<shown.length-1?", ":""}</span>))}
                            {nbrObjs.length>5?`, and ${nbrObjs.length-5} more`:"."}</>)
                        : ". It has no outbound links yet — drag it beside related baubles, or run a similarity rebuild, to weave it in."}
                    </>);};
                    const escRaw=(s)=>String(s||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
                    return (<>
                      <p style={{fontSize:13,lineHeight:1.7,color:"#c8d2f5",margin:"0 0 14px",whiteSpace:"pre-wrap"}}>
                        <span style={{float:"left",fontSize:30,fontWeight:800,color:col,lineHeight:0.9,paddingRight:8,textShadow:`0 0 10px ${col}`}}>{dn.title[0]}</span>
                        {dn.raw
                          ? <span ref={cardBodyRef} dangerouslySetInnerHTML={{__html:escRaw(dn.raw)}}/>
                          : (<>{loading ? "Fetching the opening passage…" : factoid(false)}{!loading&&" Select any text here to highlight, annotate, or send it to your notebook."}</>)}
                      </p>
                      {dn.raw&&(
                        <p style={{fontSize:11,lineHeight:1.6,color:"#6b76a6",fontStyle:"italic",margin:"0 0 6px",borderTop:"1px solid #ffffff10",paddingTop:10}}>{factoid(true)}</p>
                      )}
                    </>);
                  })()}
                  {(()=>{
                    // SOURCE & CITATION — real provenance: clickable original link + when it entered your space
                    const lbl=srcLabel(dn); const created=dn.created||SESSION_START; const cited=fmtStamp(Date.now());
                    return (<div style={{margin:"16px 0 4px",padding:"10px 12px",borderRadius:10,background:"rgba(126,232,250,0.04)",border:"1px solid #ffffff10"}}>
                      <div style={{fontSize:10,letterSpacing:1,color:"#7a86b8",marginBottom:7}}>SOURCE &amp; CITATION</div>
                      {dn.url
                        ? (<a href={dn.url} target="_blank" rel="noopener noreferrer" onClick={ev=>ev.stopPropagation()} style={{display:"flex",alignItems:"center",gap:7,fontSize:12,color:"#9ad8ff",textDecoration:"none",wordBreak:"break-all"}}>
                            <span style={{flexShrink:0}}>↗</span><span><b style={{color:"#c8d2f5"}}>{lbl||"Original source"}</b> — {dn.title}</span></a>)
                        : (<div style={{fontSize:12,color:"#6b76a6",fontStyle:"italic"}}>No external source — {dn.raw?"locally authored / ingested":"semantic node (no document yet)"}</div>)}
                      <div style={{fontSize:10.5,color:"#7e8ab8",marginTop:7,lineHeight:1.7}}>
                        <div>Added to your space · <span style={{color:"#9aa6d8"}}>{fmtStamp(created)}</span></div>
                        {dn.url&&<div style={{fontFamily:"var(--font-mono, monospace)",color:"#5d6796",fontSize:10,marginTop:3}}>{lbl?lbl+", ":""}“{dn.title}.” Retrieved {cited}.</div>}
                      </div>
                    </div>);
                  })()}
                  <div style={{fontSize:10,letterSpacing:1,color:"#7a86b8",margin:"16px 0 8px"}}>OUTBOUND VECTORS</div>
                  {links.map((e,i)=>{const other=e.from===diveInto?e.to:e.from;return (<div key={i} onClick={()=>{setSelMenu(null);setDiveInto(other);}} style={{display:"flex",alignItems:"center",gap:8,padding:"8px 10px",marginBottom:5,borderRadius:10,cursor:"pointer",background:"rgba(255,255,255,0.03)",border:`1px solid ${REL[e.rel].c}22`}}><span style={{width:7,height:7,borderRadius:7,background:REL[e.rel].c,boxShadow:`0 0 6px ${REL[e.rel].c}`}}/><span style={{fontSize:11,color:REL[e.rel].c,width:56}}>{REL[e.rel].label}</span><span style={{flex:1,fontSize:13,color:"#e6ecff"}}>{map[other].title}</span><span style={{fontSize:14,color:"#5d6796"}}>↳</span></div>);})}
                  {(()=>{const mine=annos.filter(a=>a.nodeId===dn.id);return mine.length>0&&(<><div style={{fontSize:10,letterSpacing:1,color:"#7a86b8",margin:"16px 0 8px"}}>YOUR MARKS ({mine.length})</div>
                  {mine.map((a)=>{const mc=a.kind==="voice"?"#ff7a92":hlc(a.color);return (<div key={a.ts} style={{fontSize:11,color:"#9aa6d8",padding:"4px 0",borderLeft:`2px solid ${mc}`,paddingLeft:8,marginBottom:4}}><span style={{color:mc}}>{a.kind==="voice"?"🎙 voice":a.kind}</span> · "{a.text.slice(0,42)}{a.text.length>42?"…":""}"{a.note&&<span style={{display:"block",fontStyle:"italic",color:"#7e8ab8"}}>📝 {a.note}</span>}{a.kind==="voice"&&a.audio&&<button onClick={()=>{try{new Audio(a.audio).play();}catch(e){}}} style={{display:"inline-block",marginTop:3,border:`1px solid ${mc}66`,background:"transparent",color:mc,cursor:"pointer",fontSize:10,padding:"1px 7px",borderRadius:6}}>▶ play</button>}</div>);})}</>);})()}
                </div>
                {selMenu&&!selDraft&&!selVoice&&selAct!=="hl"&&(
                  // stage 1 — textual context menu. Nothing is marked until you pick an action.
                  <div style={{position:"absolute",left:Math.min(selMenu.x,180),top:selMenu.y,zIndex:9,display:"flex",alignItems:"stretch",overflow:"hidden",background:"rgba(12,16,38,0.97)",backdropFilter:"blur(14px)",border:`1px solid ${col}55`,borderRadius:10,boxShadow:`0 10px 40px #000a, 0 0 24px ${col}22`}}>
                    {[
                      {k:"hl",  ic:"🖍",label:"Highlight", on:()=>setSelAct("hl")},
                      {k:"an",  ic:"✎", label:"Annotate", on:()=>setSelDraft({color:"green",note:""})},
                      {k:"sh",  ic:"⤴", label:"Share",     on:()=>{ try{navigator.clipboard&&navigator.clipboard.writeText(selMenu.text);}catch(e){} setSelMenu(null); window.getSelection&&window.getSelection().removeAllRanges(); }},
                      {k:"vo",  ic:"🎙",label:"Voice note",on:()=>startVoice(dn.id,selMenu.text)},
                    ].map((a,i)=>(
                      <button key={a.k} title={a.label} onClick={a.on}
                        style={{display:"flex",alignItems:"center",gap:6,border:"none",borderLeft:i?"1px solid #ffffff14":"none",background:"transparent",color:"#cdd7ff",cursor:"pointer",fontSize:11.5,padding:"7px 11px",whiteSpace:"nowrap"}}
                        onMouseEnter={e=>e.currentTarget.style.background=col+"22"} onMouseLeave={e=>e.currentTarget.style.background="transparent"}>
                        <span style={{fontSize:13}}>{a.ic}</span>{a.label}
                      </button>
                    ))}
                  </div>
                )}
                {selMenu&&selAct==="hl"&&!selVoice&&(
                  // stage 2 — colour picker, only after Highlight was chosen
                  <div style={{position:"absolute",left:Math.min(selMenu.x,210),top:selMenu.y,zIndex:10,display:"flex",alignItems:"center",gap:3,padding:5,background:"rgba(12,16,38,0.98)",backdropFilter:"blur(14px)",border:`1px solid ${col}77`,borderRadius:10,boxShadow:`0 10px 40px #000a, 0 0 24px ${col}22`}}>
                    <button title="back" onClick={()=>setSelAct(null)} style={{border:"none",background:"transparent",color:"#8893c4",cursor:"pointer",fontSize:13,padding:"2px 5px"}}>‹</button>
                    {HLC.map(h=>(
                      <button key={h.k} title={"highlight · "+h.k} onClick={()=>{ setAnnos(a=>[...a,{ts:Date.now()+Math.random(),kind:"highlight",text:selMenu.text,color:h.k,nodeId:dn.id}]); setSelMenu(null); setSelAct(null); window.getSelection&&window.getSelection().removeAllRanges(); }}
                        style={{width:19,height:19,borderRadius:5,border:"1px solid #ffffff33",background:h.c,cursor:"pointer",padding:0}}/>
                    ))}
                  </div>
                )}
                {selVoice&&(
                  // stage 2 (voice) — live recorder
                  <div style={{position:"absolute",left:Math.min(selMenu?.x||20,150),top:selMenu?.y||20,zIndex:11,width:230,padding:11,background:"rgba(12,16,38,0.98)",backdropFilter:"blur(14px)",border:`1px solid ${selVoice.err?"#ff6b6b":"#ff5470"}99`,borderRadius:12,boxShadow:"0 14px 44px #000b"}}>
                    {selVoice.err
                      ? (<><div style={{fontSize:11,color:"#ffb3b3",lineHeight:1.5,marginBottom:8}}>🎙 {selVoice.err}</div>
                         <button onClick={()=>{setSelVoice(null);setSelMenu(null);setSelAct(null);}} style={{...btn,width:"100%"}}>close</button></>)
                      : (<><div style={{display:"flex",alignItems:"center",gap:8,marginBottom:9}}><span style={{width:10,height:10,borderRadius:10,background:"#ff5470",boxShadow:"0 0 10px #ff5470",animation:"pulse 1s infinite"}}/><span style={{fontSize:11,color:"#ffd0d8"}}>recording your voice note…</span></div>
                         <div style={{fontSize:10,color:"#9aa6d8",lineHeight:1.4,marginBottom:9,maxHeight:34,overflow:"hidden"}}>“{(selVoice.text||"").slice(0,80)}{(selVoice.text||"").length>80?"…":""}”</div>
                         <div style={{display:"flex",gap:6}}>
                           <button onClick={stopVoice} style={{...btn,flex:1,color:"#05060f",background:"#ff7a92",border:"none"}}>■ stop &amp; save</button>
                           <button onClick={cancelVoice} style={{...btn}}>cancel</button>
                         </div></>)}
                  </div>
                )}
                {selMenu&&selDraft&&(
                  <div style={{position:"absolute",left:Math.min(selMenu.x,170),top:selMenu.y,zIndex:10,width:248,padding:10,background:"rgba(12,16,38,0.98)",backdropFilter:"blur(14px)",border:`1px solid ${col}77`,borderRadius:12,boxShadow:`0 14px 44px #000b, 0 0 26px ${col}22`}}>
                    <div style={{fontSize:10,color:"#9aa6d8",lineHeight:1.4,marginBottom:7,maxHeight:48,overflow:"hidden"}}>“{selMenu.text.slice(0,100)}{selMenu.text.length>100?"…":""}”</div>
                    <div style={{display:"flex",gap:4,marginBottom:7}}>
                      {HLC.map(h=>(<button key={h.k} title={h.k} onClick={()=>setSelDraft(d=>({...d,color:h.k}))} style={{width:18,height:18,borderRadius:5,border:selDraft.color===h.k?"2px solid #fff":"1px solid #ffffff22",background:h.c,cursor:"pointer",padding:0}}/>))}
                    </div>
                    <textarea autoFocus value={selDraft.note} onChange={e=>setSelDraft(d=>({...d,note:e.target.value}))}
                      onKeyDown={e=>{ if(e.key==="Enter"&&(e.metaKey||e.ctrlKey)){e.preventDefault(); const t=selMenu.text; setAnnos(a=>[...a,{ts:Date.now()+Math.random(),kind:selDraft.note.trim()?"annotate":"highlight",text:t,note:selDraft.note.trim()||undefined,color:selDraft.color,nodeId:dn.id}]); setSelDraft(null); setSelMenu(null); window.getSelection&&window.getSelection().removeAllRanges(); } if(e.key==="Escape") setSelDraft(null); }}
                      placeholder="type a note…  (⌘/Ctrl+Enter to save)" rows={3}
                      style={{width:"100%",boxSizing:"border-box",resize:"vertical",fontSize:12,lineHeight:1.5,color:"#e7ecff",background:"#070b1c",border:"1px solid #233060",borderRadius:8,padding:"7px 8px",outline:"none",fontFamily:"inherit"}}/>
                    <div style={{display:"flex",gap:6,marginTop:7}}>
                      <button onClick={()=>{ const t=selMenu.text; setAnnos(a=>[...a,{ts:Date.now()+Math.random(),kind:selDraft.note.trim()?"annotate":"highlight",text:t,note:selDraft.note.trim()||undefined,color:selDraft.color,nodeId:dn.id}]); setSelDraft(null); setSelMenu(null); window.getSelection&&window.getSelection().removeAllRanges(); }} style={{...btn,flex:1,color:"#05060f",background:"#7cffc4",border:"none"}}>save note</button>
                      <button onClick={()=>setSelDraft(null)} style={{...btn}}>cancel</button>
                    </div>
                  </div>
                )}
            </div>
          );
        })()}

        {journey&&(
          <div style={{position:"absolute",inset:0,background:"rgba(4,6,16,0.82)",backdropFilter:"blur(3px)",display:"flex",flexDirection:"column",padding:18}}>
            <div style={{display:"flex",alignItems:"center",gap:10,marginBottom:8}}>
              <span style={{fontSize:11,letterSpacing:1,color:"#7ee8fa",textShadow:"0 0 8px #7ee8fa"}}>◷ THE 411 · YOUR JOURNEY</span>
              <span style={{fontSize:10,color:"#7a86b8"}}>{trail.length} stops · 4 or Esc to dive back in</span>
            </div>
            <svg viewBox={`0 0 ${W} ${H-60}`} style={{flex:1,width:"100%"}}>
              {(()=>{ const pts=trail.map((id,i)=>{const x=60+(i/Math.max(1,trail.length-1))*(W-120);const y=(H-60)/2+Math.sin(i*1.3)*90+Math.cos(i*0.7)*40;return{id,x,y,onto:map[id].onto,title:map[id].title};});
                return (<>{pts.slice(1).map((p,i)=>{const a=pts[i];const mx=(a.x+p.x)/2,my=(a.y+p.y)/2-28;return <path key={i} d={`M${a.x},${a.y} Q${mx},${my} ${p.x},${p.y}`} fill="none" stroke="#3a4a7a" strokeWidth={2} strokeDasharray="2 5" markerEnd="url(#jar)"/>;})}
                  <defs><marker id="jar" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto"><path d="M0,0 L6,3 L0,6 Z" fill="#5a6aaa"/></marker></defs>
                  {pts.map((p,i)=>(<g key={i} style={{cursor:"pointer"}} onClick={()=>{setSel(p.id);setJourney(false);}}><circle cx={p.x} cy={p.y} r={14} fill="#0a0e22" stroke={ONTO[p.onto].c} strokeWidth={2} style={{filter:`drop-shadow(0 0 8px ${ONTO[p.onto].c})`}}/><text x={p.x} y={p.y+30} textAnchor="middle" fontSize={10} fill="#dfe7ff">{p.title}</text><text x={p.x} y={p.y+4} textAnchor="middle" fontSize={9} fill="#7a86b8">{i+1}</text></g>))}</>);
              })()}
            </svg>
          </div>
        )}

        <div style={{position:"absolute",top:0,left:0,height:"100%",width:200,transform:trayOpen?"translateX(0)":"translateX(-200px)",transition:"transform 0.32s cubic-bezier(.2,.9,.3,1)",background:"rgba(10,14,34,0.66)",backdropFilter:"blur(16px)",borderRight:"1px solid #1f2a55",padding:14,boxShadow:"8px 0 40px #0008",overflowY:"auto"}}>
          <div style={{fontSize:10,letterSpacing:1,color:"#7ee8fa",textShadow:"0 0 8px #7ee8fa",marginBottom:10}}>CONTROLS</div>
          <Slider label="float / drift" min="0.62" max="0.95" step="0.01" val={cfg.damp} on={set("damp")} fmt={v=>v.toFixed(2)}/>
          <Slider label="look coast" min="0.7" max="0.95" step="0.01" val={cfg.lookDamp} on={set("lookDamp")} fmt={v=>v.toFixed(2)}/>
          <Slider label="spin coast" min="0.85" max="0.98" step="0.01" val={cfg.spinDecay} on={set("spinDecay")} fmt={v=>v.toFixed(2)}/>
          <Slider label="spring stiffness" min="0.004" max="0.05" step="0.002" val={cfg.spring} on={set("spring")} fmt={v=>v.toFixed(3)}/>
          <Slider label="node spacing" min="3000" max="18000" step="500" val={cfg.charge} on={set("charge")} fmt={v=>(v/1000).toFixed(1)+"k"}/>
          <Slider label="breathing" min="0" max="1.4" step="0.05" val={cfg.breathe} on={set("breathe")} fmt={v=>v.toFixed(2)}/>
          <Slider label="glow" min="0.3" max="2" step="0.1" val={cfg.glow} on={set("glow")} fmt={v=>v.toFixed(1)}/>
          <label style={{display:"flex",alignItems:"center",gap:8,fontSize:11,color:"#9aa6d8",marginTop:10,cursor:"pointer"}}><input type="checkbox" checked={cfg.vignette} onChange={e=>setCfg(c=>({...c,vignette:e.target.checked}))}/>bubble vignettes</label>
          <label title="when you stop touching it, the field eases back to centre" style={{display:"flex",alignItems:"center",gap:8,fontSize:11,color:"#9aa6d8",marginTop:8,cursor:"pointer"}}><input type="checkbox" checked={cfg.recenterIdle} onChange={e=>setCfg(c=>({...c,recenterIdle:e.target.checked}))}/>self-right (keep centered)</label>
          <button onClick={()=>setCfg(c=>({damp:0.78,lookDamp:0.86,spring:0.018,charge:9000,breathe:0.5,glow:1,vignette:true,recenterIdle:true,spinDecay:0.94,przewalski:c.przewalski}))} style={{...btn,width:"100%",marginTop:12}}>reset feel</button>
          <label title="spread the herd — repel nodes apart so you can read between them (3-finger spread does this live)" style={{display:"flex",alignItems:"center",gap:8,fontSize:11,color:HUD_ACCENT,marginTop:22,cursor:"pointer",opacity:0.95}}><input type="checkbox" checked={!!cfg.przewalski} onChange={e=>setCfg(c=>({...c,przewalski:e.target.checked}))}/>spread mode <span style={{opacity:0.6}}>(Przewalski)</span></label>
        </div>
        <button onClick={()=>setTrayOpen(o=>!o)} style={{position:"absolute",top:"50%",marginTop:-34,left:trayOpen?200:0,transition:"left 0.32s cubic-bezier(.2,.9,.3,1)",width:26,height:68,background:"rgba(14,18,42,0.82)",backdropFilter:"blur(10px)",border:"1px solid #1f2a55",borderLeft:trayOpen?"1px solid #1f2a55":"none",borderRadius:"0 10px 10px 0",color:"#9ad8ff",cursor:"pointer",display:"flex",alignItems:"center",justifyContent:"center",fontSize:13,padding:0}}>{trayOpen?"‹":"›"}</button>

        {showHud&&selNode&&(
          <div style={{position:"absolute",top:14,right:14,width:230,padding:14,borderRadius:14,background:"rgba(14,18,42,0.55)",backdropFilter:"blur(14px)",border:`1px solid ${ONTO[selNode.onto].c}44`,boxShadow:`0 0 30px ${ONTO[selNode.onto].c}22`}}>
            <div style={{fontSize:10,letterSpacing:1,color:ONTO[selNode.onto].c,textShadow:`0 0 8px ${ONTO[selNode.onto].c}`}}>{ONTO[selNode.onto].label.toUpperCase()}</div>
            <div style={{fontSize:17,fontWeight:700,margin:"4px 0 8px"}}>{selNode.title}</div>
            <div style={{fontSize:11,color:"#9aa6d8",marginBottom:8}}>{reResolving?"◌ re-resolving vectors…":`${selEdges.length} semantic vectors`}</div>
            {selEdges.map((e,i)=>{const other=e.from===sel?e.to:e.from;const dir=e.from===sel?"→":"←";return (<div key={i} style={{display:"flex",alignItems:"center",gap:6,fontSize:11,padding:"3px 0"}}><span style={{width:6,height:6,borderRadius:6,background:REL[e.rel].c,boxShadow:`0 0 6px ${REL[e.rel].c}`}}/><span style={{color:REL[e.rel].c,width:52}}>{REL[e.rel].label}</span><span style={{color:"#7a86b8"}}>{dir}</span><span style={{flex:1,color:"#dfe7ff",cursor:"pointer"}} onClick={()=>setSel(other)}>{map[other].title}</span><span style={{color:"#5d6796",fontSize:9}}>{(e.conf*100|0)}%</span></div>);})}
          </div>
        )}

        {showHud&&(
          <div style={{position:"absolute",bottom:chat.open?"calc(40% - 2px)":48,right:12,zIndex:15,padding:"8px 12px",borderRadius:10,background:"rgba(10,14,34,0.78)",backdropFilter:"blur(10px)",border:"1px solid #243056",fontSize:10,color:"#8893c8",lineHeight:1.7,maxWidth:300,pointerEvents:"none",transition:"bottom .2s ease"}}>
            <div><b style={{color:"#cdd6ff"}}>hold</b> aim+charge → release fire · <b style={{color:"#cdd6ff"}}>← →</b> spin <span style={{opacity:0.6}}>(or shift/right-drag &amp; fling)</span> · <b style={{color:"#cdd6ff"}}>scroll</b> fall fwd · <b style={{color:"#cdd6ff"}}>grab</b> haul · <b style={{color:"#cdd6ff"}}>E</b> reticle</div>
            <div style={{marginTop:4,display:"flex",gap:8,flexWrap:"wrap"}}>{Object.values(ONTO).map(o=>(<span key={o.label} style={{display:"flex",alignItems:"center",gap:3}}><span style={{width:7,height:7,borderRadius:7,background:o.c,boxShadow:`0 0 5px ${o.c}`}}/>{o.label}</span>))}</div>
          </div>
        )}
        </>)}

        {tab==="reader" && (
          <ReaderView
            nodes={graph.current.nodes}
            edges={graph.current.edges}
            sel={readerId}
            onSel={setReaderId}
            annos={annos}
            addAnno={(o)=>setAnnos(a=>[...a,{ts:Date.now()+Math.random(),...o}])}
            delAnno={(ts)=>setAnnos(a=>a.filter(x=>x.ts!==ts))}
            onSpatial={(id)=>{setSel(id);setReaderId(id);setTab("spatial");cam.current.vFwd+=5;interact.current=performance.now();}}
            onChat={(title)=>{setChat(c=>({...c,open:true}));sendChat("Tell me about "+title+".");}}
          />
        )}

        {/* the chat panel is also the search/navbar, so it is always mounted — the toggle
            collapses it to just the input bar rather than removing it entirely */}
        <ChatPanel
          chat={chat}
          setChat={setChat}
          onSend={()=>sendChat()}
          onSearch={()=>runSearch()}
          onAddGhost={addGhost}
          onSource={onSource}
          collapsed={!chat.open}
          onToggle={()=>setChat(c=>({...c,open:!c.open}))}
          connOk={!!normBase(conn.chatHost)}
        />

        {ctx && (()=>{ const node=byId()[ctx.id]; if(!node) return null; return (
          <>
            <div onClick={()=>setCtx(null)} onContextMenu={(e)=>{e.preventDefault();setCtx(null);}} style={{position:"fixed",inset:0,zIndex:9998}}/>
            <div style={{position:"fixed",left:Math.min(ctx.x,window.innerWidth-180),top:Math.min(ctx.y,window.innerHeight-120),zIndex:9999,minWidth:160,background:"rgba(10,14,32,0.97)",backdropFilter:"blur(14px)",border:"1px solid #2a3766",borderRadius:10,boxShadow:"0 14px 40px #000a",overflow:"hidden",fontSize:12}}>
              <div style={{padding:"7px 11px",color:"#7a86b8",borderBottom:"1px solid #1b2140",fontSize:10,letterSpacing:0.5,whiteSpace:"nowrap",overflow:"hidden",textOverflow:"ellipsis",maxWidth:200}}>{ctx.title}</div>
              {ctx.ghost
                ? <>
                    <button onClick={()=>{addGhost(ctx.id);setCtx(null);}} style={{display:"block",width:"100%",textAlign:"left",padding:"9px 11px",background:"none",border:"none",color:"#7cffc4",cursor:"pointer",fontWeight:600}}>＋ add to your RAG</button>
                    <button onClick={()=>{clearGhosts([ctx.id]);setSearch(s=>({...s,ghosts:s.ghosts.filter(g=>g!==ctx.id)}));setGraphVer(v=>v+1);setCtx(null);}} style={{display:"block",width:"100%",textAlign:"left",padding:"9px 11px",background:"none",border:"none",color:"#9aa6cc",cursor:"pointer",borderTop:"1px solid #1b2140"}}>✕ dismiss ghost</button>
                  </>
                : <button onClick={()=>{setSel(ctx.id);setReaderId(ctx.id);setTab("reader");setCtx(null);}} style={{display:"block",width:"100%",textAlign:"left",padding:"9px 11px",background:"none",border:"none",color:"#9ad8ff",cursor:"pointer"}}>▤ open in reader</button>}
              {node.url&&<button onClick={()=>{window.open(node.url,"_blank","noopener");setCtx(null);}} style={{display:"block",width:"100%",textAlign:"left",padding:"9px 11px",background:"none",border:"none",color:"#9ad8ff",cursor:"pointer",borderTop:"1px solid #1b2140"}}>↗ open source</button>}
            </div>
          </>
        );})()}

        {notes && (
          <Notebook
            annos={annos}
            nodes={graph.current.nodes}
            onClose={()=>setNotes(false)}
            onOpenNode={(id)=>{ if(id){setReaderId(id);setTab("reader");} setNotes(false); }}
            del={(ts)=>setAnnos(a=>a.filter(x=>x.ts!==ts))}
            upd={(ts,patch)=>setAnnos(a=>a.map(x=>x.ts===ts?{...x,...patch}:x))}
          />
        )}

      </div>

      <div style={{flexShrink:0,display:"flex",alignItems:"center",justifyContent:"flex-end",gap:10,padding:"4px 12px",borderTop:"1px solid #161b38",background:"linear-gradient(90deg,#070818,#0a0d22)",fontFamily:"ui-monospace, SFMono-Regular, Menlo, monospace",fontSize:10,color:"#5a6493",letterSpacing:"0.03em",userSelect:"none",whiteSpace:"nowrap",overflow:"hidden"}}>
        <span style={{color:"#7ee8fa",textShadow:"0 0 6px rgba(126,232,250,0.4)"}}>{APP_VERSION}</span>
        <span style={{color:"#2d3352"}}>·</span>
        <span style={{overflow:"hidden",textOverflow:"ellipsis",color:HUD_ACCENT,opacity:0.92,textShadow:"0 0 5px rgba(126,232,250,0.3)"}}>{APP_COPYRIGHT}</span>
      </div>
    </div>
  );
}

function Slider({label,min,max,step,val,on,fmt}){
  return (
    <div style={{marginBottom:10}}>
      <div style={{display:"flex",justifyContent:"space-between",fontSize:10,color:"#9aa6d8",marginBottom:3}}><span>{label}</span><span style={{color:"#7ee8fa"}}>{fmt(val)}</span></div>
      <input type="range" min={min} max={max} step={step} value={val} onChange={on} style={{width:"100%",accentColor:"#7ee8fa",height:3}}/>
    </div>
  );
}

const btn={background:"rgba(126,232,250,0.08)",color:"#9ad8ff",border:"1px solid #1f2a55",borderRadius:8,padding:"4px 10px",fontSize:11,cursor:"pointer"};

const inS={width:"100%",boxSizing:"border-box",background:"#0a0e22",border:"1px solid #1f2a55",borderRadius:8,color:"#dfe7ff",fontSize:11,padding:"6px 8px"};

// ── highlighter palette: colour-code marks so the notebook can group by colour ──
const HLC=[
  {k:"yellow",c:"#ffd479"},{k:"green",c:"#7cffc4"},{k:"cyan",c:"#7ee8fa"},
  {k:"pink",c:"#ff9ecd"},{k:"violet",c:"#b388ff"},{k:"orange",c:"#ffb066"},
];
const hlc=(k)=>(HLC.find(h=>h.k===k)||HLC[0]).c;
const hexA=(hex,a)=>{const h=String(hex||"#ffd479").replace("#","");const n=parseInt(h.length===3?h.split("").map(x=>x+x).join(""):h,16);return `rgba(${(n>>16)&255},${(n>>8)&255},${n&255},${a})`;};
// Whitespace-tolerant, MULTI-NODE highlighter. A real selection in an Evernote /
// UpNote / ChatGPT export almost always crosses inline tags (<a>,<b>,<em>) and line
// breaks, so the old single-textnode `indexOf` matched nothing and nothing painted.
// This concatenates every text node, matches on collapsed whitespace, then wraps each
// spanned text-node slice in its own <mark> (surroundContents can't cross nodes).
function paintMark(root,raw,color,note){
  if(!root||!raw) return false;
  const want=String(raw).replace(/\s+/g," ").trim().toLowerCase(); if(!want) return false;
  const tw=document.createTreeWalker(root,NodeFilter.SHOW_TEXT,null);
  let norm="",prevSp=true,tn; const map=[];               // map[normIdx] = {node,offset}
  while((tn=tw.nextNode())){
    if(tn.parentNode&&tn.parentNode.closest&&tn.parentNode.closest("mark.ts-hl,script,style")) continue;
    const v=tn.nodeValue;
    for(let i=0;i<v.length;i++){ const ch=v[i];
      if(/\s/.test(ch)){ if(prevSp) continue; norm+=" "; map.push({node:tn,offset:i}); prevSp=true; }
      else { norm+=ch.toLowerCase(); map.push({node:tn,offset:i}); prevSp=false; } }
  }
  const at=norm.indexOf(want); if(at<0) return false;
  const end=at+want.length, runs=new Map();              // node -> [minOff,maxOff]
  for(let k=at;k<end;k++){ const m=map[k]; if(!m) continue; const r=runs.get(m.node);
    if(!r) runs.set(m.node,[m.offset,m.offset]); else { if(m.offset<r[0])r[0]=m.offset; if(m.offset>r[1])r[1]=m.offset; } }
  const c=hlc(color);
  runs.forEach((rng,node)=>{ try{
    const r=document.createRange(); r.setStart(node,rng[0]); r.setEnd(node,rng[1]+1);
    const mk=document.createElement("mark"); mk.className="ts-hl";
    // neon highlighter: bright saturated wash + outer glow + crisp inner edge, dark ink for punch
    mk.style.cssText="background:"+hexA(c,.78)+";color:#0a0c16;font-weight:600;border-radius:3px;padding:0 2px;"
      +"box-shadow:0 0 8px "+hexA(c,.9)+",0 0 2px "+hexA(c,1)+",inset 0 0 0 1px "+hexA(c,1)+";"
      +"text-shadow:0 0 1px "+hexA(c,.5)+(note?";cursor:help":"");
    if(note) mk.title="📝 "+note;
    mk.appendChild(r.extractContents()); r.insertNode(mk);
  }catch(e){} });
  return true;
}
function download(name,text,mime){ try{ const b=new Blob([text],{type:mime||"application/octet-stream"}); const u=URL.createObjectURL(b); const a=document.createElement("a"); a.href=u; a.download=name; document.body.appendChild(a); a.click(); a.remove(); setTimeout(()=>URL.revokeObjectURL(u),2000);}catch(e){} }
// collated marks → portable Markdown, grouped by topic (the unit you actually re-read)
function marksToMd(marks,titleOf){
  const byTopic={}; marks.forEach(a=>{const t=a.topic||"(untagged)"; (byTopic[t]=byTopic[t]||[]).push(a);});
  let md="# ThinkSpace Notebook\n\n_"+marks.length+" marks_\n";
  Object.keys(byTopic).sort().forEach(t=>{ md+="\n## "+t+"\n\n"; byTopic[t].forEach(a=>{
    md+="- ["+(a.color||"yellow")+"] “"+(a.text||"").trim()+"”  \n  _— "+(titleOf(a.nodeId)||a.nodeId||"source")+"_"+(a.note?"  \n  📝 "+a.note:"")+"\n"; }); });
  return md;
}
// print via a hidden iframe (popup-blocker / sandbox safe)
function printMarks(marks,titleOf){
  const esc=s=>String(s||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
  const byTopic={}; marks.forEach(a=>{const t=a.topic||"(untagged)"; (byTopic[t]=byTopic[t]||[]).push(a);});
  let body=""; Object.keys(byTopic).sort().forEach(t=>{ body+='<h2>'+esc(t)+'</h2>'; byTopic[t].forEach(a=>{ const c=hlc(a.color);
    body+='<div class=m><span class=sw style="background:'+c+'"></span><div><blockquote>'+esc(a.text)+'</blockquote><div class=src>— '+esc(titleOf(a.nodeId)||a.nodeId||"")+'</div>'+(a.note?'<div class=note>📝 '+esc(a.note)+'</div>':'')+'</div></div>'; }); });
  const html='<!doctype html><meta charset=utf-8><title>ThinkSpace Notebook</title><style>body{font:14px/1.6 -apple-system,system-ui,sans-serif;color:#111;max-width:720px;margin:32px auto;padding:0 22px}h1{font-size:24px;margin:0 0 4px}h2{margin:26px 0 8px;border-bottom:1px solid #ddd;padding-bottom:4px;font-size:15px;color:#444}.m{display:flex;gap:10px;margin:11px 0;break-inside:avoid}.sw{width:12px;height:12px;border-radius:3px;flex-shrink:0;margin-top:5px}blockquote{margin:0;font-size:14px}.src{color:#777;font-size:12px;margin-top:2px}.note{color:#1a7f37;font-style:italic;font-size:13px;margin-top:3px}</style><h1>ThinkSpace Notebook</h1><p style="color:#777;font-size:12px">'+marks.length+' marks</p>'+body;
  const f=document.createElement("iframe"); f.style.cssText="position:fixed;right:0;bottom:0;width:0;height:0;border:0;opacity:0";
  document.body.appendChild(f); const d=f.contentWindow.document; d.open(); d.write(html); d.close();
  setTimeout(()=>{ try{ f.contentWindow.focus(); f.contentWindow.print(); }catch(e){} setTimeout(()=>f.remove(),1500); },350);
}

// ── connection editor (used twice: embeddings + chat) ──
// Real combobox: a native <datalist> only renders a short, host-capped slice and
// won't scroll a long list — so 347 models showed as a stub. This is type-to-filter
// PLUS a fully scrollable click list (maxHeight + overflow) over every option.
function ModelCombo({value,onChange,options,placeholder,busy,onList}){
  const [open,setOpen]=useState(false);
  const [dirty,setDirty]=useState(false); // true once the user types — only then do we filter
  const boxRef=useRef(null);
  useEffect(()=>{ const h=(e)=>{ if(boxRef.current&&!boxRef.current.contains(e.target)) setOpen(false); }; document.addEventListener("mousedown",h); return ()=>document.removeEventListener("mousedown",h); },[]);
  const q=String(value||"").toLowerCase();
  // opening (focus or ▾) shows EVERY model; we only narrow once the user actually types,
  // otherwise a committed selection filtered the list down to its own single row ("defective").
  const filt=(dirty&&q)?options.filter(o=>o.toLowerCase().includes(q)):options;
  const shown=open?filt:[];
  return (
    <div ref={boxRef} style={{position:"relative"}}>
      <div style={{display:"flex",gap:6}}>
        <input value={value} onChange={e=>{onChange(e.target.value);setDirty(true);setOpen(true);}} onFocus={()=>{setDirty(false);setOpen(true);}} placeholder={placeholder} style={{...inS,flex:1}}/>
        <button onClick={()=>{setDirty(false);setOpen(o=>!o);}} title="show all models" style={{...btn,padding:"0 9px"}}>{open?"▴":"▾"}</button>
        <button onClick={onList} disabled={busy} title="refresh model list" style={{...btn,whiteSpace:"nowrap",opacity:busy?0.6:1}}>{busy?"…":"↻"}</button>
      </div>
      {open&&options.length>0&&(
        <div style={{position:"absolute",top:"100%",left:0,right:0,zIndex:60,marginTop:4,maxHeight:260,overflowY:"auto",background:"#0a0e22",border:"1px solid #2a3a6a",borderRadius:8,boxShadow:"0 14px 44px rgba(0,0,0,0.6)"}}>
          {shown.length?shown.map(o=>(
            <div key={o} onClick={()=>{onChange(o);setOpen(false);setDirty(false);}} title={o}
              style={{padding:"6px 10px",fontSize:11,cursor:"pointer",color:o===value?"#7ee8fa":"#cdd6ff",background:o===value?"rgba(126,232,250,0.08)":"transparent",whiteSpace:"nowrap",overflow:"hidden",textOverflow:"ellipsis"}}
              onMouseEnter={e=>e.currentTarget.style.background="rgba(126,232,250,0.14)"}
              onMouseLeave={e=>e.currentTarget.style.background=o===value?"rgba(126,232,250,0.08)":"transparent"}>{o}</div>
          )):<div style={{padding:"6px 10px",fontSize:11,color:"#5d6796"}}>no match — clear the field to see all</div>}
        </div>
      )}
    </div>
  );
}
function ConnFields({kind,conn,setConn,models,busy,err,onList,title,accent,phHost,phModel}){
  const up=(field,val)=>setConn(c=>({...c,[kind+field]:val}));
  const host=conn[kind+"Host"]||"", model=conn[kind+"Model"]||"", key=conn[kind+"Key"]||"";
  const provider=conn[kind+"Provider"]==="openai"?"openai":"ollama"; // anthropic retired → openai-compat covers z.ai/GLM/openrouter
  const keyPh=provider==="openai"?"API key (sk-… / sk-or-… / z.ai)":"API key (Ollama cloud — optional)";
  const hostPh=phHost;
  return (
    <div style={{marginBottom:12,padding:"10px",borderRadius:10,background:"rgba(126,232,250,0.03)",border:"1px solid #182146"}}>
      <div style={{display:"flex",alignItems:"center",marginBottom:6}}>
        <span style={{fontSize:10,letterSpacing:1,color:accent,textTransform:"uppercase"}}>{title}</span>
        <div style={{flex:1}}/>
        <select value={provider} onChange={e=>up("Provider",e.target.value)} style={{...inS,width:"auto",padding:"2px 6px",fontSize:10,color:"#cdd6ff"}}>
          <option value="ollama">Ollama</option>
          <option value="openai">OpenAI-compat</option>
        </select>
      </div>
      <input value={host} onChange={e=>up("Host",e.target.value)} placeholder={hostPh} style={{...inS,marginBottom:6}}/>
      <ModelCombo value={model} onChange={v=>up("Model",v)} options={models} placeholder={phModel} busy={busy} onList={onList}/>
      <input type="password" value={key} onChange={e=>up("Key",e.target.value)} placeholder={keyPh} style={{...inS,marginTop:6}}/>
      {err
        ? <div style={{fontSize:9,color:"#ff9b8a",marginTop:4,lineHeight:1.4,wordBreak:"break-word"}}>⚠ {err}</div>
        : models.length>0&&<div style={{fontSize:9,color:"#5d6796",marginTop:4}}>{models.length} models · type to filter</div>}
    </div>
  );
}

// ── collated notebook: every highlight + note from every doc, in one place,
//    organisable by colour / topic so a research thread reads top-to-bottom ──
function Notebook({annos,nodes,onClose,onOpenNode,del,upd}){
  const [grp,setGrp]=useState("topic");   // topic | color | doc
  const [filt,setFilt]=useState("");        // colour filter ("" = all)
  const [q,setQ]=useState("");
  const [edit,setEdit]=useState(null);      // {ts,field:"topic"|"note",val} inline editor (prompt() is blocked in sandboxed previews)
  const titleOf=(id)=>{const n=nodes.find(x=>x.id===id);return n?n.title:id;};
  let marks=annos.filter(a=>a.kind==="highlight"||a.kind==="annotate");
  if(filt) marks=marks.filter(a=>(a.color||"yellow")===filt);
  if(q){ const ql=q.toLowerCase(); marks=marks.filter(a=>(a.text||"").toLowerCase().includes(ql)||(a.note||"").toLowerCase().includes(ql)||(a.topic||"").toLowerCase().includes(ql)); }
  const keyOf=(a)=> grp==="color"?(a.color||"yellow") : grp==="doc"?(titleOf(a.nodeId)||"—") : (a.topic||"(untagged)");
  const groups={}; marks.forEach(a=>{const k=keyOf(a);(groups[k]=groups[k]||[]).push(a);});
  const gkeys=Object.keys(groups).sort((a,b)=>a==="(untagged)"?1:b==="(untagged)"?-1:a.localeCompare(b));
  const total=annos.filter(a=>a.kind==="highlight"||a.kind==="annotate").length;
  const GBTN=(k,lab)=>(<button key={k} onClick={()=>setGrp(k)} style={{border:"none",borderRadius:6,padding:"4px 9px",fontSize:10,cursor:"pointer",color:grp===k?"#05060f":"#9ad8ff",background:grp===k?"#ffd479":"rgba(126,232,250,0.08)"}}>{lab}</button>);
  return (
    <div style={{position:"absolute",inset:0,zIndex:20,display:"flex",flexDirection:"column",background:"rgba(6,9,22,0.975)",backdropFilter:"blur(12px)"}}>
      <div style={{display:"flex",alignItems:"center",gap:8,flexWrap:"wrap",padding:"12px 14px",borderBottom:"1px solid #161b38",background:"linear-gradient(90deg,#0a0d22,#0b0a18)"}}>
        <span style={{fontSize:13,letterSpacing:1,color:"#ffd479",textShadow:"0 0 10px #ffd479",fontWeight:700}}>✑ NOTEBOOK</span>
        <span style={{fontSize:11,color:"#7a86b8"}}>{marks.length}{filt||q?" / "+total:""} marks</span>
        <div style={{flex:1}}/>
        <span style={{fontSize:10,color:"#5d6796"}}>group</span>
        <div style={{display:"flex",gap:3}}>{GBTN("topic","topic")}{GBTN("color","colour")}{GBTN("doc","document")}</div>
        <span onClick={onClose} style={{cursor:"pointer",color:"#7a86b8",fontSize:16,marginLeft:4}}>✕</span>
      </div>
      <div style={{display:"flex",alignItems:"center",gap:8,flexWrap:"wrap",padding:"8px 14px",borderBottom:"1px solid #11152e"}}>
        <input value={q} onChange={e=>setQ(e.target.value)} placeholder="search marks…" style={{...inS,width:180}}/>
        <span style={{fontSize:10,color:"#5d6796"}}>colour</span>
        <button onClick={()=>setFilt("")} title="all colours" style={{width:18,height:18,borderRadius:5,border:filt===""?"2px solid #fff":"1px solid #2a3a6a",background:"conic-gradient(#ffd479,#7cffc4,#7ee8fa,#ff9ecd,#b388ff,#ffb066,#ffd479)",cursor:"pointer",padding:0}}/>
        {HLC.map(h=>(<button key={h.k} onClick={()=>setFilt(filt===h.k?"":h.k)} title={h.k} style={{width:18,height:18,borderRadius:5,border:filt===h.k?"2px solid #fff":"1px solid #ffffff22",background:h.c,cursor:"pointer",padding:0}}/>))}
        <div style={{flex:1}}/>
        <button onClick={()=>{try{navigator.clipboard&&navigator.clipboard.writeText(marksToMd(marks,titleOf));}catch(e){}}} style={btn} title="copy notebook as Markdown">⤴ copy</button>
        <button onClick={()=>download("thinkspace-notebook.md",marksToMd(marks,titleOf),"text/markdown")} style={btn}>⬇ .md</button>
        <button onClick={()=>download("thinkspace-notebook.json",JSON.stringify(marks,null,2),"application/json")} style={btn}>⬇ .json</button>
        <button onClick={()=>printMarks(marks,titleOf)} style={{...btn,color:"#05060f",background:"#ffd479",border:"none"}}>🖨 print</button>
      </div>
      <div style={{flex:1,overflowY:"auto",padding:"6px 14px 26px"}}>
        {marks.length===0&&<div style={{textAlign:"center",color:"#5d6796",fontSize:12,padding:"50px 20px"}}>No marks yet — highlight or ✎ annotate text in the <b style={{color:"#9ad8ff"}}>▤ reader</b> and it collects here. Group by colour or topic to line up a research thread (values, then calculations…) across documents.</div>}
        {gkeys.map(gk=>(
          <div key={gk} style={{marginTop:14}}>
            <div style={{display:"flex",alignItems:"center",gap:8,margin:"0 0 8px",position:"sticky",top:0,background:"rgba(6,9,22,0.96)",padding:"4px 0",zIndex:1}}>
              {grp==="color"&&<span style={{width:11,height:11,borderRadius:3,background:hlc(gk)}}/>}
              <span style={{fontSize:11,letterSpacing:1,color:"#cdd6ff",textTransform:"uppercase",fontWeight:700}}>{gk}</span>
              <span style={{fontSize:10,color:"#5d6796"}}>{groups[gk].length}</span>
              <div style={{flex:1,height:1,background:"#161b38"}}/>
            </div>
            {groups[gk].map(a=>{const c=hlc(a.color);return (
              <div key={a.ts} style={{display:"flex",gap:9,padding:"8px 0",borderBottom:"1px solid #0e1330"}}>
                <div style={{display:"flex",flexDirection:"column",gap:3,flexShrink:0}}>
                  {HLC.map(h=>(<button key={h.k} onClick={()=>upd(a.ts,{color:h.k})} title={h.k} style={{width:11,height:11,borderRadius:3,border:(a.color||"yellow")===h.k?"1px solid #fff":"1px solid transparent",background:h.c,cursor:"pointer",padding:0,opacity:(a.color||"yellow")===h.k?1:0.5}}/>))}
                </div>
                <div style={{flex:1,minWidth:0}}>
                  <div style={{fontSize:12.5,color:"#dbe3ff",lineHeight:1.5,borderLeft:`2px solid ${c}`,paddingLeft:9}}>{a.kind==="voice"&&<span style={{color:"#ff7a92"}}>🎙 </span>}“{a.text}”</div>
                  {a.note&&<div style={{fontSize:11.5,color:"#7cffc4",fontStyle:"italic",marginTop:4,paddingLeft:9}}>📝 {a.note}</div>}
                  {a.kind==="voice"&&a.audio&&<button onClick={()=>{try{new Audio(a.audio).play();}catch(e){}}} style={{marginTop:5,marginLeft:9,border:"1px solid #ff7a9266",background:"transparent",color:"#ff7a92",cursor:"pointer",fontSize:10.5,padding:"1px 9px",borderRadius:6}}>▶ play voice note</button>}
                  <div style={{display:"flex",alignItems:"center",gap:8,marginTop:5,flexWrap:"wrap"}}>
                    <button onClick={()=>onOpenNode(a.nodeId)} style={{border:"none",background:"transparent",color:"#7ee8fa",cursor:"pointer",fontSize:10.5,padding:0}}>↪ {titleOf(a.nodeId)}</button>
                    <button onClick={()=>setEdit({ts:a.ts,field:"topic",val:a.topic||""})} style={{border:"1px solid #2a3a6a",background:"rgba(126,232,250,0.06)",color:a.topic?"#cdd6ff":"#5d6796",cursor:"pointer",fontSize:10,padding:"1px 7px",borderRadius:20}}>{a.topic?"# "+a.topic:"+ topic"}</button>
                    <button onClick={()=>setEdit({ts:a.ts,field:"note",val:a.note||""})} style={{border:"none",background:"transparent",color:"#9aa6d8",cursor:"pointer",fontSize:10.5,padding:0}}>✎ note</button>
                    <div style={{flex:1}}/>
                    <button onClick={()=>del(a.ts)} title="remove" style={{border:"none",background:"transparent",color:"#5d6796",cursor:"pointer",fontSize:12}}>✕</button>
                  </div>
                  {edit&&edit.ts===a.ts&&(
                    <div style={{display:"flex",gap:6,marginTop:6}}>
                      <input autoFocus value={edit.val} onChange={e=>setEdit(s=>({...s,val:e.target.value}))}
                        onKeyDown={e=>{ if(e.key==="Enter"){ const v=edit.val.trim(); if(edit.field==="topic") upd(a.ts,{topic:v}); else upd(a.ts,{note:v||undefined,kind:v?"annotate":a.kind}); setEdit(null); } if(e.key==="Escape") setEdit(null); }}
                        placeholder={edit.field==="topic"?"topic (groups marks across docs)…":"note… (Enter to save)"}
                        style={{...inS,flex:1}}/>
                      <button onClick={()=>{ const v=edit.val.trim(); if(edit.field==="topic") upd(a.ts,{topic:v}); else upd(a.ts,{note:v||undefined,kind:v?"annotate":a.kind}); setEdit(null); }} style={{...btn,color:"#05060f",background:"#7cffc4",border:"none"}}>save</button>
                    </div>
                  )}
                </div>
              </div>
            );})}
          </div>
        ))}
      </div>
    </div>
  );
}

// ── PDF flipbook: render pages to canvas with page-turn (pdf.js, lazy-loaded from
//    CDN). Falls back to a native <iframe> viewer if the lib or the file is blocked
//    (CORS / offline) so a PDF is never just a dead link. ──
function PdfFlip({url}){
  const [st,setSt]=useState("init");          // init | ok | fallback
  const [doc,setDoc]=useState(null);
  const [page,setPage]=useState(1);
  const [pages,setPages]=useState(0);
  const [flip,setFlip]=useState("");
  const cvRef=useRef(null);
  useEffect(()=>{ let dead=false; setSt("init"); setDoc(null); setPage(1);
    (async()=>{
      try{
        if(!window.pdfjsLib){
          await new Promise((res,rej)=>{ const s=document.createElement("script"); s.src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.min.js"; s.onload=res; s.onerror=rej; document.head.appendChild(s); });
          if(window.pdfjsLib) window.pdfjsLib.GlobalWorkerOptions.workerSrc="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js";
        }
        if(!window.pdfjsLib) throw new Error("pdf.js unavailable");
        const pdf=await window.pdfjsLib.getDocument(url).promise;
        if(dead) return; setDoc(pdf); setPages(pdf.numPages); setPage(1); setSt("ok");
      }catch(e){ if(!dead) setSt("fallback"); }
    })();
    return ()=>{dead=true;};
  },[url]);
  useEffect(()=>{ if(st!=="ok"||!doc) return; let dead=false;
    (async()=>{ try{
      const pg=await doc.getPage(page); if(dead) return;
      const cv=cvRef.current; if(!cv) return;
      const base=pg.getViewport({scale:1});
      const avail=(cv.parentNode&&cv.parentNode.clientWidth||620)-8;
      const scale=Math.max(0.4,Math.min(2.2,avail/base.width));
      const vp=pg.getViewport({scale});
      cv.width=vp.width; cv.height=vp.height;
      await pg.render({canvasContext:cv.getContext("2d"),viewport:vp}).promise;
    }catch(e){} })();
    return ()=>{dead=true;};
  },[st,doc,page]);
  const go=(d)=>{ setFlip(d>0?"next":"prev"); setPage(p=>Math.max(1,Math.min(pages||1,p+d))); setTimeout(()=>setFlip(""),260); };
  if(st==="fallback") return (
    <div>
      <div style={{fontSize:11,color:"#ffd479",marginBottom:8}}>⚠ inline flipbook blocked (CORS / offline) — native viewer below.</div>
      <iframe src={url} title="pdf" style={{width:"100%",height:600,border:"1px solid #1f2a55",borderRadius:10,background:"#fff"}}/>
      <a href={url} target="_blank" rel="noopener" style={{display:"inline-block",marginTop:8,fontSize:11,color:"#7ee8fa"}}>↗ open original PDF</a>
    </div>
  );
  return (
    <div style={{textAlign:"center"}}>
      <div style={{minHeight:200,perspective:1600,display:"flex",justifyContent:"center"}}>
        {st==="init" ? <div style={{color:"#7a86b8",fontSize:12,padding:"60px 0"}}>◌ opening flipbook…</div>
          : <canvas ref={cvRef} style={{maxWidth:"100%",borderRadius:8,boxShadow:"0 14px 50px #000b",background:"#fff",transformOrigin:flip==="next"?"left center":"right center",transform:flip?"rotateY("+(flip==="next"?"-14deg":"14deg")+")":"none",transition:"transform .26s ease"}}/>}
      </div>
      {pages>0&&(
        <div style={{display:"inline-flex",alignItems:"center",gap:12,marginTop:14,padding:"7px 14px",borderRadius:30,background:"rgba(14,18,42,0.8)",border:"1px solid #233060"}}>
          <button onClick={()=>go(-1)} disabled={page<=1} style={{...btn,opacity:page<=1?0.4:1,padding:"4px 12px"}}>◀</button>
          <span style={{fontSize:11,color:"#bcc6ee",fontVariantNumeric:"tabular-nums",minWidth:70}}>page {page} / {pages}</span>
          <button onClick={()=>go(1)} disabled={page>=pages} style={{...btn,opacity:page>=pages?0.4:1,padding:"4px 12px"}}>▶</button>
          <a href={url} target="_blank" rel="noopener" style={{fontSize:10,color:"#7ee8fa",marginLeft:4}}>↗ original</a>
        </div>
      )}
    </div>
  );
}

// ── reader lens: linear, navigable html/markdown ──
function ReaderView({nodes,edges,sel,onSel,annos,addAnno,delAnno,onSpatial,onChat}){
  const [q,setQ]=useState("");
  const [sort,setSort]=useState("az");
  const [menu,setMenu]=useState(null);
  const [act,setAct]=useState(null);       // null=textual menu; "hl"=colour stage
  const [voice,setVoice]=useState(null);   // live voice-note recording {text,err}
  const recRef=useRef(null);
  const [draft,setDraft]=useState(null);   // {text,color,note} inline note composer (prompt() is blocked in sandboxed previews)
  const recVoice=async(text)=>{
    if(!navigator.mediaDevices||!window.MediaRecorder){ setVoice({text,err:"Mic recording isn't available here — try the native shell."}); return; }
    try{
      const stream=await navigator.mediaDevices.getUserMedia({audio:true});
      const rec=new MediaRecorder(stream); const chunks=[]; recRef.current=rec;
      rec.ondataavailable=e=>{ if(e.data&&e.data.size) chunks.push(e.data); };
      rec.onstop=()=>{ stream.getTracks().forEach(t=>t.stop()); const blob=new Blob(chunks,{type:rec.mimeType||"audio/webm"}); const fr=new FileReader(); fr.onload=()=>{ addAnno({kind:"voice",text,audio:fr.result,nodeId:node.id}); }; fr.readAsDataURL(blob); setVoice(null); setMenu(null); setAct(null); };
      rec.start(); setVoice({text});
    }catch(e){ setVoice({text,err:"Mic permission denied or blocked in this preview."}); }
  };
  const stopRec=()=>{ try{ recRef.current&&recRef.current.state!=="inactive"&&recRef.current.stop(); }catch(e){} };
  const cancelRec=()=>{ try{ if(recRef.current){ recRef.current.onstop=null; recRef.current.state!=="inactive"&&recRef.current.stop(); } }catch(e){} setVoice(null); setMenu(null); setAct(null); };
  const [remote,setRemote]=useState({});   // nodeId -> {st:'loading'|'ok'|'err', html, err} for link-only nodes
  const bodyRef=useRef(null);
  const byId={}; nodes.forEach(n=>byId[n.id]=n);
  const deg={}; edges.forEach(e=>{deg[e.from]=(deg[e.from]||0)+1; deg[e.to]=(deg[e.to]||0)+1;});
  let list=nodes.filter(n=>!q||(n.title||"").toLowerCase().includes(q.toLowerCase())||(n.raw||"").toLowerCase().includes(q.toLowerCase()));
  list=[...list].sort((a,b)=> sort==="links"?((deg[b.id]||0)-(deg[a.id]||0)) : sort==="type"?String(a.onto).localeCompare(String(b.onto)) : String(a.title||"").localeCompare(String(b.title||"")));
  const node=byId[sel]||nodes[0];
  if(!node) return <div style={{position:"absolute",inset:0,zIndex:8,display:"flex",alignItems:"center",justifyContent:"center",color:"#5d6796",fontSize:13,background:"#05060f"}}>no documents — load a graph or ingest a folder in ⊞ data</div>;
  const oc=ONTO[node.onto]||{c:"#7ee8fa",label:"node"};
  const bodyHtml=node.html?scrubHtml(node.html):mdToHtml(node.raw||node.title||"");
  // ── link-only nodes (recall.it bookmarks, PDF refs) carry a url but little/no body.
  //    Retrieve & render the content instead of showing a dead list entry. ──
  const mediaUrl=Array.isArray(node.media)?(node.media.find(m=>typeof m==="string"&&/^https?:|\.pdf/i.test(m))||""):"";
  const srcUrl=node.url||mediaUrl||"";
  const isPdf=/\.pdf($|[?#])/i.test(srcUrl)||/\.pdf($|[?#])/i.test(node.title||"");
  const thin=!node.html && (node.raw||"").replace(/\s+/g,"").length<40;
  const linkOnly=!!srcUrl && /^https?:/i.test(srcUrl) && thin && !isPdf;
  const rm=remote[node.id]||{};
  const repaintDeps=rm.st==="ok"?rm.html:bodyHtml;
  const links=edges.filter(e=>e.from===node.id||e.to===node.id).map(e=>{const oid=e.from===node.id?e.to:e.from;return {oid,node:byId[oid],rel:e.rel};}).filter(x=>x.node);
  const resolve=(href)=>{ if(!href) return null; let h=String(href).trim(); if(/^https?:|^mailto:/i.test(h)) return {ext:h}; h=h.replace(/^#/,""); try{h=decodeURIComponent(h);}catch(e){} const low=h.toLowerCase(); const hit=nodes.find(n=>String(n.id).toLowerCase()===low)||nodes.find(n=>String(n.title||"").toLowerCase()===low)||nodes.find(n=>low.length>2&&String(n.title||"").toLowerCase().includes(low)); return hit?{id:hit.id}:{ext:href}; };
  const onBodyClick=(e)=>{ const a=e.target.closest&&e.target.closest("a"); if(!a) return; e.preventDefault(); const r=resolve(a.getAttribute("href")); if(r&&r.id) onSel(r.id); else if(r&&r.ext) window.open(r.ext,"_blank","noopener"); };
  const onBodyUp=(e)=>{ const s=window.getSelection&&window.getSelection(); const text=s?String(s).trim():""; if(!text){setMenu(null);setDraft(null);setAct(null);setVoice(null);return;} setDraft(null); setAct(null); setVoice(null); const host=e.currentTarget.getBoundingClientRect(); let rect=host; try{rect=s.getRangeAt(0).getBoundingClientRect();}catch(err){} setMenu({x:rect.left-host.left+rect.width/2,y:rect.top-host.top,w:host.width,text}); };
  const myAnnos=(annos||[]).filter(a=>a.nodeId===node.id);
  // Re-paint highlights/notes after EVERY render. The body is innerHTML, so any
  // <mark> we inject by hand is wiped whenever the reader re-renders — persisting
  // the selected text and re-wrapping it here is what makes a highlight actually stick.
  useEffect(()=>{
    const root=bodyRef.current; if(!root) return;
    root.querySelectorAll("mark.ts-hl").forEach(m=>{ const p=m.parentNode; while(m.firstChild) p.insertBefore(m.firstChild,m); p.removeChild(m); p.normalize&&p.normalize(); });
    myAnnos.filter(a=>(a.kind==="highlight"||a.kind==="annotate")&&a.text).forEach(a=>paintMark(root,a.text,a.color,a.note));
  },[node.id,repaintDeps,annos]);
  // retrieve link-only node content (best-effort; CORS may block → iframe fallback in render)
  useEffect(()=>{
    if(!linkOnly) return; const id=node.id; if(remote[id]&&remote[id].st!=="err") return;
    let dead=false; setRemote(r=>({...r,[id]:{st:"loading"}}));
    (async()=>{
      try{
        const res=await fetch(srcUrl,{redirect:"follow"});
        const txt=await res.text();
        const bm=txt.match(/<body[^>]*>([\s\S]*?)<\/body>/i);
        let html=bm?bm[1]:txt;
        html=html.replace(/<(script|style|nav|header|footer|aside)[\s\S]*?<\/\1>/gi,"");
        if(!dead) setRemote(r=>({...r,[id]:{st:"ok",html}}));
      }catch(e){ if(!dead) setRemote(r=>({...r,[id]:{st:"err",err:String((e&&e.message)||e)}})); }
    })();
    return ()=>{dead=true;};
  },[node.id,linkOnly,srcUrl]);
  return (
    <div style={{position:"absolute",inset:0,zIndex:8,display:"flex",background:"radial-gradient(ellipse at 50% 0%, #0b1030 0%, #05060f 70%)"}}>
      <div style={{width:230,flexShrink:0,borderRight:"1px solid #161b38",display:"flex",flexDirection:"column",background:"rgba(8,11,28,0.6)"}}>
        <div style={{padding:"10px 10px 8px"}}>
          <input value={q} onChange={e=>setQ(e.target.value)} placeholder="filter…" style={{...inS}}/>
          <div style={{display:"flex",gap:4,marginTop:6}}>
            {[["az","A·Z"],["links","links"],["type","type"]].map(([k,lab])=>(
              <button key={k} onClick={()=>setSort(k)} style={{flex:1,border:"none",borderRadius:6,padding:"4px 0",fontSize:9,cursor:"pointer",color:sort===k?"#05060f":"#9ad8ff",background:sort===k?"#7ee8fa":"rgba(126,232,250,0.08)"}}>{lab}</button>
            ))}
          </div>
        </div>
        <div style={{flex:1,overflowY:"auto",padding:"0 6px 10px"}}>
          {list.map(n=>{const c=(ONTO[n.onto]||{}).c||"#7ee8fa";const on=n.id===node.id;return (
            <div key={n.id} onClick={()=>onSel(n.id)} style={{display:"flex",alignItems:"center",gap:7,padding:"7px 8px",borderRadius:8,cursor:"pointer",marginBottom:2,background:on?"rgba(126,232,250,0.12)":"transparent",border:on?"1px solid #2a4a7a":"1px solid transparent"}}>
              <span style={{width:7,height:7,borderRadius:7,flexShrink:0,background:c,boxShadow:`0 0 6px ${c}`}}/>
              <span style={{flex:1,fontSize:11,color:on?"#fff":"#bcc6ee",whiteSpace:"nowrap",overflow:"hidden",textOverflow:"ellipsis"}}>{n.title}</span>
              <span style={{fontSize:9,color:"#5d6796"}}>{deg[n.id]||0}</span>
            </div>
          );})}
          {list.length===0&&<div style={{fontSize:10,color:"#5d6796",padding:10,textAlign:"center"}}>nothing matches</div>}
        </div>
      </div>
      <div style={{flex:1,position:"relative",overflowY:"auto"}}>
        <div style={{maxWidth:680,margin:"0 auto",padding:"26px 30px 60px"}}>
          <div style={{fontSize:10,letterSpacing:1,color:oc.c,textShadow:`0 0 8px ${oc.c}`,textTransform:"uppercase"}}>{oc.label}</div>
          <h1 style={{fontSize:26,fontWeight:800,margin:"6px 0 14px",color:"#f2f5ff",lineHeight:1.15}}>{node.title}</h1>
          <div style={{display:"flex",gap:8,marginBottom:18,flexWrap:"wrap"}}>
            <button onClick={()=>onSpatial(node.id)} style={{...btn,borderColor:"#2a4a7a"}}>◎ show in space</button>
            <button onClick={()=>onChat(node.title)} style={{...btn,color:"#05060f",background:"#7cffc4",border:"none"}}>✦ ask about this</button>
            {srcUrl&&<a href={srcUrl} target="_blank" rel="noopener" style={{...btn,textDecoration:"none",display:"inline-flex",alignItems:"center",color:"#9ad8ff"}}>{isPdf?"📕 PDF source":"↗ source"}</a>}
          </div>
          {isPdf ? (
            <PdfFlip url={srcUrl}/>
          ) : linkOnly ? (
            rm.st==="ok" ? (
              <div ref={bodyRef} onClick={onBodyClick} onMouseUp={onBodyUp} style={{fontSize:14,lineHeight:1.75,color:"#c9d2f5",wordBreak:"break-word"}} dangerouslySetInnerHTML={{__html:scrubHtml(rm.html)}}/>
            ) : (
              <div style={{fontSize:13,color:"#9aa6d8",lineHeight:1.7}}>
                <div style={{marginBottom:10}}>{rm.st==="loading"?("◌ retrieving "+srcUrl+" …"):("⚠ couldn't pull the page inline (likely CORS). Showing it live below.")}</div>
                {rm.err&&<div style={{fontSize:10,color:"#ff9b8a",marginBottom:10,wordBreak:"break-word"}}>{rm.err}</div>}
                <iframe src={srcUrl} title={node.title} style={{width:"100%",height:560,border:"1px solid #1f2a55",borderRadius:10,background:"#fff"}}/>
              </div>
            )
          ) : (
            <div ref={bodyRef} onClick={onBodyClick} onMouseUp={onBodyUp} style={{fontSize:14,lineHeight:1.75,color:"#c9d2f5",wordBreak:"break-word"}} dangerouslySetInnerHTML={{__html:bodyHtml}}/>
          )}
          {links.length>0&&(
            <div style={{marginTop:28,paddingTop:16,borderTop:"1px solid #161b38"}}>
              <div style={{fontSize:10,letterSpacing:1,color:"#7a86b8",marginBottom:10}}>LINKS</div>
              <div style={{display:"flex",gap:8,flexWrap:"wrap"}}>
                {links.map((l,i)=>{const c=(ONTO[l.node.onto]||{}).c||"#7ee8fa";const rl=(REL[l.rel]||{}).label||l.rel;return (
                  <button key={i} onClick={()=>onSel(l.oid)} style={{display:"flex",alignItems:"center",gap:6,padding:"6px 11px",borderRadius:20,cursor:"pointer",fontSize:11,color:"#dfe7ff",background:"rgba(14,18,42,0.7)",border:`1px solid ${c}55`}}>
                    <span style={{width:6,height:6,borderRadius:6,background:c,boxShadow:`0 0 5px ${c}`}}/>{l.node.title}<span style={{fontSize:9,color:"#7a86b8"}}>{rl}</span>
                  </button>
                );})}
              </div>
            </div>
          )}
          {myAnnos.length>0&&(
            <div style={{marginTop:28,paddingTop:16,borderTop:"1px solid #161b38"}}>
              <div style={{fontSize:10,letterSpacing:1,color:"#7a86b8",marginBottom:10}}>YOUR MARKS · {myAnnos.length} <span style={{color:"#4a5378"}}>(saved)</span></div>
              {myAnnos.map((a)=>{const ac=hlc(a.color);return (
                <div key={a.ts} style={{display:"flex",alignItems:"flex-start",gap:8,fontSize:11.5,color:"#c2ccf0",padding:"6px 0",borderLeft:`2px solid ${ac}`,paddingLeft:9,marginBottom:5}}>
                  <span style={{flex:1}}><span style={{color:ac,fontSize:9,letterSpacing:.5,textTransform:"uppercase"}}>{a.kind}</span><br/>"{a.text.slice(0,90)}{a.text.length>90?"…":""}"{a.note&&<span style={{display:"block",color:"#9aa6d8",marginTop:3,fontStyle:"italic"}}>📝 {a.note}</span>}</span>
                  <button onClick={()=>delAnno&&delAnno(a.ts)} title="remove" style={{border:"none",background:"transparent",color:"#5d6796",cursor:"pointer",fontSize:13,padding:"0 2px"}}>✕</button>
                </div>
              );})}
            </div>
          )}
        </div>
        {menu&&!draft&&!voice&&act!=="hl"&&(
          // stage 1 — textual menu; no mark is made until an action is chosen
          <div style={{position:"absolute",left:Math.max(8,Math.min(menu.x-100,menu.w-220)),top:Math.max(4,menu.y-44),display:"flex",overflow:"hidden",borderRadius:10,background:"rgba(14,18,42,0.97)",backdropFilter:"blur(12px)",border:"1px solid #2a3a6a",boxShadow:"0 10px 30px #000a",zIndex:9}}>
            {[
              {k:"hl", ic:"🖍",label:"Highlight", on:()=>setAct("hl")},
              {k:"an", ic:"✎", label:"Annotate", on:()=>setDraft({text:menu.text,color:"green",note:""})},
              {k:"sh", ic:"⤴", label:"Share",     on:()=>{ try{navigator.clipboard&&navigator.clipboard.writeText(menu.text);}catch(e){} setMenu(null); if(window.getSelection)window.getSelection().removeAllRanges(); }},
              {k:"vo", ic:"🎙",label:"Voice note",on:()=>recVoice(menu.text)},
            ].map((a,i)=>(
              <button key={a.k} title={a.label} onClick={a.on}
                style={{display:"flex",alignItems:"center",gap:6,border:"none",borderLeft:i?"1px solid #ffffff14":"none",background:"transparent",color:"#cdd7ff",cursor:"pointer",fontSize:11.5,padding:"7px 11px",whiteSpace:"nowrap"}}
                onMouseEnter={e=>e.currentTarget.style.background="#7ee8fa22"} onMouseLeave={e=>e.currentTarget.style.background="transparent"}>
                <span style={{fontSize:13}}>{a.ic}</span>{a.label}
              </button>
            ))}
          </div>
        )}
        {menu&&act==="hl"&&!voice&&(
          // stage 2 — colour picker, shown only after Highlight
          <div style={{position:"absolute",left:Math.max(8,Math.min(menu.x-90,menu.w-180)),top:Math.max(4,menu.y-44),display:"flex",alignItems:"center",gap:3,padding:5,borderRadius:10,background:"rgba(12,16,38,0.98)",backdropFilter:"blur(12px)",border:"1px solid #2a3a6a",boxShadow:"0 10px 30px #000a",zIndex:10}}>
            <button title="back" onClick={()=>setAct(null)} style={{border:"none",background:"transparent",color:"#8893c4",cursor:"pointer",fontSize:13,padding:"2px 5px"}}>‹</button>
            {HLC.map(h=>(
              <button key={h.k} title={"highlight · "+h.k} onClick={()=>{ addAnno({kind:"highlight",text:menu.text,color:h.k,nodeId:node.id}); setMenu(null); setAct(null); if(window.getSelection)window.getSelection().removeAllRanges(); }}
                style={{width:19,height:19,borderRadius:5,border:"1px solid #ffffff33",background:h.c,cursor:"pointer",padding:0}}/>
            ))}
          </div>
        )}
        {menu&&voice&&(
          <div style={{position:"absolute",left:Math.max(8,Math.min(menu.x-115,menu.w-238)),top:Math.max(4,menu.y-10),width:230,padding:11,borderRadius:12,background:"rgba(12,16,38,0.98)",backdropFilter:"blur(14px)",border:`1px solid ${voice.err?"#ff6b6b":"#ff5470"}99`,boxShadow:"0 14px 40px #000b",zIndex:11}}>
            {voice.err
              ? (<><div style={{fontSize:11,color:"#ffb3b3",lineHeight:1.5,marginBottom:8}}>🎙 {voice.err}</div>
                 <button onClick={()=>{setVoice(null);setMenu(null);setAct(null);}} style={{...btn,width:"100%"}}>close</button></>)
              : (<><div style={{display:"flex",alignItems:"center",gap:8,marginBottom:9}}><span style={{width:10,height:10,borderRadius:10,background:"#ff5470",boxShadow:"0 0 10px #ff5470",animation:"pulse 1s infinite"}}/><span style={{fontSize:11,color:"#ffd0d8"}}>recording…</span></div>
                 <div style={{display:"flex",gap:6}}>
                   <button onClick={stopRec} style={{...btn,flex:1,color:"#05060f",background:"#ff7a92",border:"none"}}>■ stop &amp; save</button>
                   <button onClick={cancelRec} style={{...btn}}>cancel</button>
                 </div></>)}
          </div>
        )}
        {menu&&draft&&(
          <div style={{position:"absolute",left:Math.max(8,Math.min(menu.x-130,menu.w-272)),top:Math.max(4,menu.y-6),width:264,padding:10,borderRadius:12,background:"rgba(12,16,38,0.98)",backdropFilter:"blur(14px)",border:"1px solid #2a3a6a",boxShadow:"0 14px 40px #000b",zIndex:10}}>
            <div style={{fontSize:10,color:"#9aa6d8",lineHeight:1.4,marginBottom:7,maxHeight:48,overflow:"hidden"}}>“{draft.text.slice(0,110)}{draft.text.length>110?"…":""}”</div>
            <div style={{display:"flex",gap:4,marginBottom:7}}>
              {HLC.map(h=>(<button key={h.k} title={h.k} onClick={()=>setDraft(d=>({...d,color:h.k}))} style={{width:18,height:18,borderRadius:5,border:draft.color===h.k?"2px solid #fff":"1px solid #ffffff22",background:h.c,cursor:"pointer",padding:0}}/>))}
            </div>
            <textarea autoFocus value={draft.note} onChange={e=>setDraft(d=>({...d,note:e.target.value}))}
              onKeyDown={e=>{ if(e.key==="Enter"&&(e.metaKey||e.ctrlKey)){e.preventDefault(); addAnno({kind:draft.note.trim()?"annotate":"highlight",text:draft.text,note:draft.note.trim()||undefined,color:draft.color,nodeId:node.id}); setDraft(null); setMenu(null); if(window.getSelection)window.getSelection().removeAllRanges(); } if(e.key==="Escape"){setDraft(null);} }}
              placeholder="type a note…  (⌘/Ctrl+Enter to save)" rows={3}
              style={{width:"100%",boxSizing:"border-box",resize:"vertical",fontSize:12,lineHeight:1.5,color:"#e7ecff",background:"#070b1c",border:"1px solid #233060",borderRadius:8,padding:"7px 8px",outline:"none",fontFamily:"inherit"}}/>
            <div style={{display:"flex",gap:6,marginTop:7}}>
              <button onClick={()=>{ addAnno({kind:draft.note.trim()?"annotate":"highlight",text:draft.text,note:draft.note.trim()||undefined,color:draft.color,nodeId:node.id}); setDraft(null); setMenu(null); if(window.getSelection)window.getSelection().removeAllRanges(); }} style={{...btn,flex:1,color:"#05060f",background:"#7cffc4",border:"none"}}>save note</button>
              <button onClick={()=>setDraft(null)} style={{...btn}}>cancel</button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ── chat-with-knowledge side panel (RAG over the graph) ──
function ChatPanel({chat,setChat,onSend,onSearch,onAddGhost,onSource,onToggle,collapsed,connOk}){
  const scroller=useRef(null);
  const mode=chat.mode||"ask";
  const dispatch=()=>{ if(mode==="search") onSearch(); else onSend(); };
  useEffect(()=>{ const el=scroller.current; if(el) el.scrollTop=el.scrollHeight; },[chat.msgs,chat.busy,collapsed]);
  const onKey=(e)=>{ if(e.key==="Enter"&&!e.shiftKey){ e.preventDefault(); dispatch(); } };
  return (
    <div style={{position:"absolute",left:0,right:0,bottom:0,width:"100%",height:collapsed?"auto":"40%",minHeight:collapsed?0:240,zIndex:13,display:"flex",flexDirection:"column",background:"rgba(8,11,28,0.94)",backdropFilter:"blur(18px)",borderTop:"1px solid #1f2a55",boxShadow:"0 -12px 50px #000a"}}>
      <div style={{display:"flex",alignItems:"center",gap:8,padding:collapsed?"8px 14px":"12px 14px",borderBottom:collapsed?"none":"1px solid #161b38"}}>
        <span style={{width:8,height:8,borderRadius:8,background:"#7cffc4",boxShadow:"0 0 10px #7cffc4"}}/>
        <strong style={{fontSize:12,letterSpacing:1}}>{collapsed?"SEARCH · ASK YOUR KNOWLEDGE":"CHAT WITH KNOWLEDGE"}</strong>
        <div style={{flex:1}}/>
        {!collapsed&&<button onClick={()=>setChat(c=>({...c,msgs:[]}))} title="clear" style={{...btn,padding:"3px 8px"}}>⌫</button>}
        <button onClick={onToggle} title={collapsed?"expand chat history":"collapse to search bar"} style={{...btn,padding:"3px 9px"}}>{collapsed?"▴":"▾"}</button>
      </div>
      {!collapsed&&<div ref={scroller} style={{flex:1,overflowY:"auto",padding:"14px",width:"100%",maxWidth:1000,margin:"0 auto"}}>
        {chat.msgs.length===0&&(
          <div style={{fontSize:12,color:"#7a86b8",lineHeight:1.7,padding:"6px 2px"}}>
            Ask anything about the nodes you've loaded — answers cite the baubles they came from, and the citations are clickable.
            {!connOk&&<div style={{marginTop:10,padding:"8px 10px",borderRadius:8,background:"rgba(255,212,121,0.08)",border:"1px solid #4a3a1a",color:"#ffd479",fontSize:11}}>⚠ Set a <b>Chat host</b> in ⊞ data → CHAT first (Ollama ip:port or an OpenAI-compatible URL + key).</div>}
          </div>
        )}
        {chat.msgs.map((m,i)=>{const me=m.role==="user";const busyLast=chat.busy&&i===chat.msgs.length-1&&!m.content;return (
          <div key={i} style={{marginBottom:14}}>
            <div style={{fontSize:9,letterSpacing:1,color:me?"#7ee8fa":"#7cffc4",marginBottom:4}}>{me?"YOU":"KB"}</div>
            <div style={{fontSize:13,lineHeight:1.65,color:me?"#dfe7ff":"#c9d2f5",whiteSpace:"pre-wrap",wordBreak:"break-word",background:me?"rgba(126,232,250,0.06)":"transparent",padding:me?"8px 10px":"0",borderRadius:me?10:0}}>{busyLast?"◌":m.content}</div>
            {m.sources&&m.sources.length>0&&(
              <div style={{display:"flex",gap:6,flexWrap:"wrap",marginTop:8}}>
                {m.sources.map((s,j)=>{const c=(ONTO[s.onto]||{}).c||"#7ee8fa";return (
                  <button key={j} onClick={()=>onSource(s.id)} title="in your graph — show source" style={{display:"flex",alignItems:"center",gap:5,padding:"3px 9px",borderRadius:14,cursor:"pointer",fontSize:10,color:"#dfe7ff",background:"rgba(14,18,42,0.8)",border:`1px solid ${c}55`}}>
                    <span style={{width:5,height:5,borderRadius:5,background:c,boxShadow:`0 0 5px ${c}`}}/>{s.title}
                  </button>
                );})}
              </div>
            )}
            {m.ghosts&&m.ghosts.length>0&&(
              <div style={{marginTop:8}}>
                <div style={{fontSize:9,letterSpacing:0.5,color:"#7a86b8",marginBottom:5}}>NOT IN YOUR RAG — ADD FROM WIKIPEDIA</div>
                <div style={{display:"flex",gap:6,flexWrap:"wrap"}}>
                  {m.ghosts.map((g,j)=>(
                    <button key={j} onClick={()=>onAddGhost(g.id)} title={g.snippet||"add to your graph"} style={{display:"flex",alignItems:"center",gap:5,padding:"3px 9px",borderRadius:14,cursor:"pointer",fontSize:10,color:"#9fb0ff",background:"rgba(14,18,42,0.55)",border:"1px dashed #3a4a7a"}}>
                      ＋ {g.title}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>
        );})}
      </div>}
      <div style={{padding:"10px 14px 12px",borderTop:collapsed?"none":"1px solid #161b38",width:"100%",maxWidth:1000,margin:"0 auto"}}>
        <div style={{display:"flex",gap:3,marginBottom:8,background:"#0a0e22",border:"1px solid #1f2a55",borderRadius:9,padding:2,width:"fit-content"}}>
          {[["ask","✦ ask"],["search","🔍 search"]].map(([k,lab])=>(
            <button key={k} onClick={()=>setChat(c=>({...c,mode:k}))} style={{border:"none",borderRadius:7,padding:"3px 12px",fontSize:11,cursor:"pointer",fontWeight:mode===k?700:400,color:mode===k?"#05060f":"#9ad8ff",background:mode===k?(k==="search"?"#ffd479":"#7cffc4"):"transparent",transition:"all .15s"}}>{lab}</button>
          ))}
          <span style={{alignSelf:"center",marginLeft:8,fontSize:10,color:"#7a86b8"}}>{mode==="search"?"locate in your knowledge (gold) · web fills gaps (ghosts)":"answer a question using your knowledge"}</span>
        </div>
        <div style={{display:"flex",gap:8,alignItems:"flex-end"}}>
          <textarea value={chat.q} onChange={e=>setChat(c=>({...c,q:e.target.value}))} onKeyDown={onKey} rows={1} placeholder={mode==="search"?"locate a topic in your knowledge + web…":"ask a question of your knowledge…"} style={{...inS,resize:"none",maxHeight:120,fontFamily:"inherit",lineHeight:1.5}}/>
          <button onClick={dispatch} disabled={chat.busy||!chat.q.trim()} style={{...btn,color:"#05060f",background:chat.busy||!chat.q.trim()?"#3a4a6a":(mode==="search"?"#ffd479":"#7cffc4"),border:"none",padding:"8px 12px",opacity:chat.busy?0.7:1}}>{mode==="search"?"🔍":"↑"}</button>
        </div>
      </div>
    </div>
  );
}

export default ThinkSpace;

/* To mount (e.g. src/main.tsx):
import { createRoot } from "react-dom/client";
import { StrictMode } from "react";
import ThinkSpace from "./thinkspace-navigatorv1107";
createRoot(document.getElementById("root")!).render(<StrictMode><ThinkSpace /></StrictMode>);
*/
