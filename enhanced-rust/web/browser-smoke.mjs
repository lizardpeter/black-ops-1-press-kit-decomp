// Optional full browser test on an authorized, WebGPU-capable local machine.
// Install Playwright separately; this script does not deploy or publish assets.
import {chromium} from 'playwright';
import assert from 'node:assert/strict';
const url=process.env.PRESSKIT_URL||'http://127.0.0.1:8080/';
const browser=await chromium.launch({headless:process.env.HEADLESS==='1'});
try{
 const page=await browser.newPage({viewport:{width:1280,height:720}}),errors=[];
 page.on('pageerror',e=>errors.push(String(e)));page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
 await page.goto(new URL('?debug',url).href);await page.waitForFunction(()=>window.presskit?.player.ready(),{},{timeout:30000});
 const waitScene=async s=>page.waitForFunction(s=>window.presskit.player.scene()===s&&window.presskit.player.ready(),s,{timeout:30000});
 async function stagePoint(x,y){const b=await page.locator('#gpu').boundingBox();const s=Math.min(b.width/1280,b.height/720);return [b.x+(b.width-1280*s)/2+x*s,b.y+(b.height-720*s)/2+y*s];}
 async function click(x,y){await page.mouse.click(...await stagePoint(x,y));}
 await click(450,420);await page.waitForFunction(()=>window.presskit.video.currentTime>.5&&window.presskit.video.readyState>=2);
 await page.keyboard.press('Escape');await waitScene('Welcome');await click(250,270);await waitScene('Videos');await click(900,220);
 await waitScene('Video');await page.waitForFunction(()=>window.presskit.video.currentTime>.5);
 await page.keyboard.press('Space');await page.waitForFunction(()=>window.presskit.video.paused);
 const t=await page.evaluate(()=>window.presskit.video.currentTime);await page.waitForTimeout(350);assert.equal(await page.evaluate(()=>window.presskit.video.currentTime),t);
 await page.mouse.move(...await stagePoint(640,640));await page.waitForFunction(()=>window.presskit.player.controls_y()<650);
 const cy=await page.evaluate(()=>window.presskit.player.controls_y());await click(640,cy+53);
 await page.waitForFunction(()=>Math.abs(window.presskit.video.currentTime/window.presskit.video.duration-.5)<.03);
 await page.keyboard.press('Escape');await waitScene('Videos');await click(250,390);await waitScene('Screens');await click(962,444);await waitScene('Gallery');
 await page.waitForFunction(()=>window.presskit.player.gallery_counter()==='20 / 20'&&window.presskit.player.ready());await page.keyboard.press('ArrowLeft');
 await page.waitForFunction(()=>window.presskit.player.gallery_counter()==='19 / 20'&&window.presskit.player.ready());
 assert.deepEqual(errors,[]);console.log('WebGPU browser media, seek, controls, gallery and timeline passed');
}finally{await browser.close();}
