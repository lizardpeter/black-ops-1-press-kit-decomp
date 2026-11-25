import {chromium} from 'playwright';
import assert from 'node:assert/strict';
import {mkdir,writeFile} from 'node:fs/promises';
const out=process.argv[2];
await mkdir(out,{recursive:true});
const browser=await chromium.launch({channel:'chromium',headless:true,args:['--enable-unsafe-webgpu','--enable-unsafe-swiftshader','--use-angle=swiftshader','--enable-features=Vulkan','--disable-vulkan-surface','--autoplay-policy=no-user-gesture-required']});
const page=await browser.newPage({viewport:{width:1280,height:720},deviceScaleFactor:1,locale:'en-US'});
const faults=[],messages=[],badRequests=[];
page.on('pageerror',e=>faults.push(String(e)));
page.on('console',m=>{messages.push(m.type()+': '+m.text());if(m.type()==='error')console.error(m.text());});
page.on('response',r=>{if(r.status()>=400)badRequests.push(r.status()+' '+r.url());});
const state=()=>page.evaluate(()=>({scene:window.presskit?.player.scene(),frame:window.presskit?.player.frame(),ready:window.presskit?.player.ready(),status:document.querySelector('#status').textContent,history:window.presskit?.frameHistory}));
const wait=async(scene,frame)=>{await page.waitForFunction(([scene,frame])=>window.presskit?.player.scene()===scene&&window.presskit.player.frame()===frame&&window.presskit.player.ready(),[scene,frame],{timeout:60000});};
async function shot(name){await page.screenshot({path:out+'/'+name+'.png',timeout:60000});}
async function click(x,y){const r=await page.locator('#gpu').boundingBox();await page.mouse.click(r.x+x*r.width/1280,r.y+y*r.height/720);}
try{
 await page.goto('http://127.0.0.1:8766/?debug=1');
 await wait('Launcher',1);await shot('launcher-720p');
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
  await wait('Wallpapers',81);await shot('wallpapers-'+h+'p');
  console.log('Browser canvas verified '+w+'x'+h);
 }
 assert.deepEqual(faults,[]);assert.deepEqual(badRequests,[]);
 const status=await state();assert.equal(status.status,'');
 console.log('Launcher, welcome, all three menus, menu switching, 1440p/4K resizing passed in Chromium WebGPU');
}finally{
 await writeFile(out+'/browser-log.txt',JSON.stringify({faults,badRequests,messages,state:await state().catch(()=>null)},null,2));
 await browser.close();
}
