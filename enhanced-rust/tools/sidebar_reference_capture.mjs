import {chromium} from 'playwright';
import {writeFile} from 'node:fs/promises';
const root=process.argv[2];
const browser=await chromium.launch({headless:true,args:['--enable-unsafe-swiftshader','--use-angle=swiftshader','--autoplay-policy=no-user-gesture-required']});
try{
 const page=await browser.newPage({viewport:{width:1280,height:720},deviceScaleFactor:1});
 page.on('pageerror',e=>console.error(e.message));
 for(const name of ['sidebar-plain','sidebar-blur']){
  await page.goto('http://127.0.0.1:8765/');
  await page.evaluate(async name=>{
   window.RufflePlayer.config={autoplay:'on',splashScreen:false,unmuteOverlay:'hidden',allowScriptAccess:false,allowNetworking:'none',backgroundColor:'#000000',preferredRenderer:'webgl'};
   const p=window.RufflePlayer.newest().createPlayer();
   p.style.cssText='width:1280px;height:720px;display:block';
   document.body.appendChild(p);
   await p.ruffle().load({url:name+'.swf',autoplay:'on',splashScreen:false,allowScriptAccess:false,allowNetworking:'none'});
  },name);
  await page.waitForTimeout(1500);
  await page.screenshot({path:root+'/'+name+'-ruffle.png'});
 }
}finally{await browser.close();}
