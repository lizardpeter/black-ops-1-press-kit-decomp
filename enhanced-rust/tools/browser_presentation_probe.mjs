import {chromium} from 'playwright';
import {mkdir,writeFile} from 'node:fs/promises';
import {PNG} from 'pngjs';
const out=process.argv[2];await mkdir(out,{recursive:true});
const flags=[
 ['swiftshader',['--enable-unsafe-webgpu','--enable-unsafe-swiftshader','--use-angle=swiftshader']],
 ['vulkan',['--enable-unsafe-webgpu','--enable-unsafe-swiftshader','--use-angle=swiftshader','--enable-features=Vulkan','--disable-vulkan-surface']],
 ['default',['--enable-unsafe-webgpu','--enable-unsafe-swiftshader']],
 ['swiftshader-shell',['--enable-unsafe-webgpu','--enable-unsafe-swiftshader','--use-angle=swiftshader']]
];
function stats(buf){const p=PNG.sync.read(buf);let hi=0,opaque=0,sum=0;for(let i=0;i<p.data.length;i+=4){let v=Math.max(p.data[i],p.data[i+1],p.data[i+2]);hi+=v>32;opaque+=p.data[i+3]===255;sum+=v;}return {w:p.width,h:p.height,bright:hi/(p.width*p.height),opaque:opaque/(p.width*p.height),mean:sum/(p.width*p.height)}}
let results=[];
for(const [name,args] of flags){
 let browser;const result={name};
 try{
  browser=await chromium.launch({...(name.endsWith('-shell')?{}:{channel:'chromium'}),headless:true,args});
  const page=await browser.newPage({viewport:{width:1280,height:720},deviceScaleFactor:1});
  page.on('console',m=>console.log(name,m.type(),m.text()));page.on('pageerror',e=>console.log(name,'pageerror',String(e)));
  await page.goto('http://127.0.0.1:8766/?debug=1');
  await page.waitForFunction(()=>window.presskit?.player.ready(),{},{timeout:30000});
  await page.waitForTimeout(500);
  const b=await page.screenshot({path:out+'/'+name+'-app.png'});result.app=stats(b);
  // Test the browser's canvas compositor independently of Rust/wgpu.
  result.direct=await page.evaluate(async()=>{
    const adapter=await navigator.gpu.requestAdapter();const device=await adapter.requestDevice();
    const c=document.createElement('canvas');c.width=640;c.height=360;c.style='position:fixed;inset:0;width:640px;height:360px;z-index:999';document.body.appendChild(c);
    const ctx=c.getContext('webgpu');const format=navigator.gpu.getPreferredCanvasFormat();ctx.configure({device,format,alphaMode:'opaque'});
    const e=device.createCommandEncoder();const p=e.beginRenderPass({colorAttachments:[{view:ctx.getCurrentTexture().createView(),clearValue:{r:1,g:0,b:0,a:1},loadOp:'clear',storeOp:'store'}]});p.end();device.queue.submit([e.finish()]);await device.queue.onSubmittedWorkDone();
    const data=c.toDataURL('image/png');return {format,data};
  });
  const data=Buffer.from(result.direct.data.split(',')[1],'base64');await writeFile(out+'/'+name+'-direct.png',data);result.direct={format:result.direct.format,stats:stats(data)};
  result.compositor=stats(await page.screenshot({path:out+'/'+name+'-compositor.png'}));
 }catch(e){result.error=String(e);}
 finally{await browser?.close();results.push(result);console.log(JSON.stringify(result));}
}
await writeFile(out+'/probe.json',JSON.stringify(results,null,2));
if(!results.some(x=>x.app?.bright>0.03))throw Error('No browser configuration presents actual press-kit artwork');
