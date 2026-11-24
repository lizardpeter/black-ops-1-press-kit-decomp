import init, { WebPlayer } from './pkg/presskit_web.js';
const canvas=document.querySelector('#gpu'),video=document.querySelector('#video'),stage=document.querySelector('#stage');
const status=document.querySelector('#status'),counter=document.querySelector('#counter'),retry=document.querySelector('#retry');
const assets=new URL('./assets/',location.href);
const asset=p=>new URL(p,assets).href;
const pending=new Map(),failed=new Map();
let player,stopped=false,last=performance.now(),musicWanted=false,mediaGeneration=0;
const music=new Audio(asset('art/sounds/1.mp3'));music.loop=true;music.volume=.65;
let audioContext;const buffers=new Map();
function notice(s){status.hidden=!s;status.textContent=s||'';}
function audioUnlock(){if(!audioContext)audioContext=new AudioContext();if(audioContext.state==='suspended')audioContext.resume().catch(()=>{});}
async function sound(id){if(!audioContext)return;try{if(!buffers.has(id)){buffers.set(id,fetch(asset(`art/sounds/${id}.mp3`)).then(r=>{if(!r.ok)throw Error(r.status);return r.arrayBuffer();}).then(b=>audioContext.decodeAudioData(b)));}const b=await buffers.get(id);const s=audioContext.createBufferSource();s.buffer=b;s.connect(audioContext.destination);s.start();}catch(e){buffers.delete(id);console.warn('sound',id,e);}}
function playMedia(path){const generation=++mediaGeneration;video.pause();video.src=asset(path);video.load();video.play().then(()=>{if(generation===mediaGeneration){retry.hidden=true;notice('');}}).catch(e=>{if(generation===mediaGeneration){notice(`Playback needs your input: ${e.message}`);retry.hidden=false;}});}
function stopMedia(){mediaGeneration++;video.pause();video.removeAttribute('src');video.load();retry.hidden=true;}
video.addEventListener('ended',()=>{if(!stopped&&player){player.media_complete();stopMedia();dispatch();}});
video.addEventListener('error',()=>{if(video.getAttribute('src'))notice('Cannot decode or load original video. Escape returns to menu.');});
retry.addEventListener('click',()=>{audioUnlock();video.play().then(()=>{retry.hidden=true;notice('');}).catch(e=>notice(e.message));if(musicWanted)music.play().catch(()=>{});});
async function xml(path){try{const r=await fetch(asset(path));if(!r.ok)throw Error(r.status);const doc=new DOMParser().parseFromString(await r.text(),'application/xml');if(doc.querySelector('parsererror'))throw Error('Invalid XML');player.xml_loaded([...doc.querySelectorAll('images > pic > image')].map(e=>e.textContent));dispatch();}catch(e){notice(`Gallery XML: ${e.message}`);}}
function dispatch(){for(const [kind,value] of player.effects())switch(kind){
 case 'fullscreen':stage.requestFullscreen?.().catch(()=>{});break;
 case 'quit':stopped=true;stopMedia();music.pause();notice('Press kit closed. Reload to reopen.');break;
 case 'play':playMedia(value);break;
 case 'stop':stopMedia();break;
 case 'toggle':if(video.paused)video.play().catch(e=>notice(e.message));else video.pause();break;
 case 'seek':if(Number.isFinite(video.duration))video.currentTime=Number(value)*video.duration;break;
 case 'music':musicWanted=value==='1';if(musicWanted)music.play().catch(()=>{});else{music.pause();music.currentTime=0;}break;
 case 'sound':sound(value);break;
 case 'open':window.open(value.startsWith('http://')?value:asset(value),'_blank','noopener,noreferrer');break;
 case 'xml':xml(value);break;
 case 'image':load(value);break;
}}
function load(path){if(pending.has(path)||failed.has(path))return;const promise=fetch(asset(path)).then(r=>{if(!r.ok)throw Error(`${r.status} ${path}`);return r.blob();}).then(b=>createImageBitmap(b,{premultiplyAlpha:'none'})).then(b=>{try{player.upload(path,b);}finally{b.close();}}).catch(e=>{failed.set(path,e);console.error(e);notice(`Asset unavailable: ${path}`);}).finally(()=>pending.delete(path));pending.set(path,promise);}
function resize(){const r=canvas.getBoundingClientRect();player.resize(Math.max(1,Math.round(r.width*devicePixelRatio)),Math.max(1,Math.round(r.height*devicePixelRatio)));}
function point(e){const r=canvas.getBoundingClientRect();return [(e.clientX-r.left)*canvas.width/r.width,(e.clientY-r.top)*canvas.height/r.height];}
canvas.addEventListener('pointermove',()=>player?.pointer());
canvas.addEventListener('pointerup',e=>{if(!player||stopped)return;audioUnlock();canvas.focus();player.click(...point(e));dispatch();});
window.addEventListener('keydown',e=>{if(!player||stopped||!['Enter','Escape','ArrowLeft','ArrowRight',' '].includes(e.key))return;e.preventDefault();audioUnlock();if(e.key==='Escape'&&player.scene()==='Intro')stopMedia();player.key(e.key);dispatch();});
function overlay(){const scene=player.scene();counter.hidden=!['Video','Gallery'].includes(scene);if(counter.hidden)return;const r=canvas.getBoundingClientRect(),s=Math.min(r.width/1280,r.height/720),x=(r.width-1280*s)/2,y=(r.height-720*s)/2;counter.style.left=`${x+(scene==='Video'?828:865)*s}px`;counter.style.top=`${y+(player.controls_y()+4)*s}px`;counter.style.width=`${126*s}px`;counter.style.fontSize=`${16*s}px`;counter.textContent=scene==='Video'?player.media_time(video.currentTime,video.duration):player.gallery_counter();}
function frame(now){if(stopped)return;try{player.advance(Math.min(250,now-last));last=now;dispatch();for(const path of player.needed())load(path);player.draw(video);overlay();requestAnimationFrame(frame);}catch(e){notice(`WebGPU: ${e.message||e}`);console.error(e);}}
try{if(!navigator.gpu)throw Error('WebGPU unavailable (use HTTPS or localhost with WebGPU enabled)');await init();const layered=(await fetch(asset('art/background/221.png'),{method:'HEAD'})).ok;player=await WebPlayer.create(canvas,layered);resize();new ResizeObserver(resize).observe(canvas);notice('');requestAnimationFrame(frame);if(new URLSearchParams(location.search).has('debug'))window.presskit={player,video};}catch(e){notice(`Initialization failed: ${e.message||e}`);console.error(e);}
