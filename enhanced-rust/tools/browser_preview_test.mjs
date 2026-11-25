import {chromium} from 'playwright';
import assert from 'node:assert/strict';
import {PNG} from 'pngjs';
import {mkdir,writeFile} from 'node:fs/promises';
const out=process.argv[2];
await mkdir(out,{recursive:true});
const browser=await chromium.launch({channel:'chromium',headless:true,args:['--enable-unsafe-webgpu','--enable-unsafe-swiftshader','--use-angle=swiftshader','--enable-features=Vulkan','--autoplay-policy=no-user-gesture-required']});
const page=await browser.newPage({viewport:{width:1280,height:720},deviceScaleFactor:1,locale:'en-US'});
const faults=[],messages=[],badRequests=[];
await page.addInitScript(()=>{window.addEventListener('unhandledrejection',e=>console.error('Unhandled rejection',e.reason));});
page.on('pageerror',e=>faults.push(String(e)));
page.on('console',m=>{messages.push(m.type()+': '+m.text());if(m.type()==='error')console.error(m.text());});
page.on('response',r=>{if(r.status()>=400)badRequests.push(r.status()+' '+r.url());});
const state=()=>page.evaluate(()=>({scene:window.presskit?.player.scene(),frame:window.presskit?.player.frame(),ready:window.presskit?.player.ready(),status:document.querySelector('#status').textContent,history:window.presskit?.frameHistory}));
const wait=async(scene,frame)=>{await page.waitForFunction(([scene,frame])=>window.presskit?.player.scene()===scene&&window.presskit.player.frame()===frame&&window.presskit.player.ready(),[scene,frame],{timeout:60000});};
const pixelChecks=[],presentationFailures=[];
function pixelStats(png,name){
 let lit=0,opaque=0;const colors=new Set();
 for(let i=0;i<png.data.length;i+=4){
   const [r,g,b,a]=[png.data[i],png.data[i+1],png.data[i+2],png.data[i+3]];
   if(a>=240)opaque++;
   if(a>=240&&Math.max(r,g,b)>32){lit++;colors.add((r<<16)|(g<<8)|b);}
 }
 const total=png.width*png.height;return {name,width:png.width,height:png.height,lit:lit/total,opaque:opaque/total,colors:colors.size};
}
function visible(stats){return stats.lit>0.015&&stats.opaque>0.9&&stats.colors>100;}
async function saveCapture(name,image){
 const stats=pixelStats(image,name);pixelChecks.push(stats);console.log(JSON.stringify(stats));
 await writeFile(out+'/'+name+'.png',PNG.sync.write(image));return stats;
}
async function shot(name){
 // Freeze only the debug render loop; draw one final frame, request the surface
 // copy in that same submission, and capture the canvas before a new frame.
 let screen,stageStats;
 try{
  const snap=await page.evaluate(async()=>{
   const p=window.presskit;p.capturePaused=true;
   const supported=p.player.request_surface_capture();
   p.player.draw(p.video,-1);
   const canvas=document.querySelector('#gpu');
   let canvasData=null,canvasError=null;
   try{canvasData=canvas.toDataURL('image/png');}catch(e){canvasError=String(e);}
   const encode=async promise=>{
    const [width,height,data]=await Promise.race([promise,new Promise((_,reject)=>setTimeout(()=>reject(Error('GPU readback timed out')),15000))]);
    let binary='';for(let p=0;p<data.length;p+=32768)binary+=String.fromCharCode(...data.subarray(p,p+32768));
    return {width,height,base64:btoa(binary)};
   };
   const results=await Promise.allSettled([encode(p.player.capture_stage()),supported?encode(p.player.take_surface_capture()):Promise.reject(Error('surface COPY_SRC unsupported'))]);
   return {canvasData,canvasError,supported,results:results.map(r=>r.status==='fulfilled'?r.value:{error:String(r.reason)})};
  });
  for(const [i,tag] of [[0,'gpu-stage'],[1,'gpu-surface']]){
   const result=snap.results[i];
   if(result.error){console.warn(tag,result.error);continue;}
   const image=new PNG({width:result.width,height:result.height});image.data=Buffer.from(result.base64,'base64');
   const stats=await saveCapture(name+'-'+tag,image);if(i===0)stageStats=stats;
  }
  if(snap.canvasData?.startsWith('data:image/png;base64,')){
   const image=PNG.sync.read(Buffer.from(snap.canvasData.split(',')[1],'base64'));
   await saveCapture(name+'-canvas',image);
  }else console.warn('Canvas snapshot unavailable',snap.canvasError);
  const buf=await page.screenshot({path:out+'/'+name+'.png',timeout:60000});
  screen=pixelStats(PNG.sync.read(buf),name);pixelChecks.push(screen);console.log(JSON.stringify(screen));
  if(!visible(screen))presentationFailures.push(name);
  assert.ok(stageStats&&visible(stageStats),'Missing actual GPU stage artwork: '+name);
 }finally{await page.evaluate(()=>{window.presskit.capturePaused=false;});}
}
async function click(x,y){const r=await page.locator('#gpu').boundingBox();await page.mouse.click(r.x+x*r.width/1280,r.y+y*r.height/720);}
try{
 await page.goto('http://127.0.0.1:8766/?debug=1');
 await wait('Launcher',1);await page.waitForTimeout(250);await shot('launcher-720p');
 assert.deepEqual(faults,[],'launcher must render without WASM runtime exceptions');
 await click(533,449);
 await page.waitForFunction(()=>window.presskit?.player.scene()==='Intro',{},{timeout:15000});
 await page.keyboard.press('Escape');await wait('Welcome',75);await shot('welcome-720p');
 for(const [y,scene,frame] of [[180,'Wallpapers',81],[300,'Videos',91],[420,'Screens',101],[180,'Wallpapers',81]]){
  await page.evaluate(()=>{window.presskit.frameHistory.length=0;});
  await click(330,y);await wait(scene,frame);
  const s=await state();assert.ok(s.history.every(f=>[75,81,91,101].includes(f)),'unexpected intermediate blank menu: '+s.history);
  await shot(scene.toLowerCase()+'-720p');
 }
 // The actual browser surface must resize to the requested physical pixel count.
 await page.evaluate(async()=>{if(document.fullscreenElement)await document.exitFullscreen();});
 for(const [w,h] of [[2560,1440],[3840,2160],[1280,720]]){
  await page.setViewportSize({width:w,height:h});
  await page.waitForFunction(([w,h])=>{let c=document.querySelector('#gpu');return c.width===w&&c.height===h;},[w,h],{timeout:30000});
  await wait('Wallpapers',81);await page.waitForTimeout(300);await shot('wallpapers-'+h+'p');
  assert.deepEqual(faults,[],'resolution switch must not poison the WASM player');
  console.log('Browser canvas verified '+w+'x'+h);
 }
 assert.deepEqual(faults,[]);assert.deepEqual(badRequests,[]);
 const status=await state();assert.equal(status.status,'');
 assert.deepEqual(presentationFailures,[],'Browser presentation failed for one or more real captures (GPU stage captures retained)');
 console.log('Launcher, welcome, all three menus, menu switching, 1440p/4K resizing passed in Chromium WebGPU');
}finally{
 await writeFile(out+'/browser-log.txt',JSON.stringify({faults,badRequests,messages,pixelChecks,presentationFailures,state:await state().catch(()=>null)},null,2));
 await browser.close();
}
