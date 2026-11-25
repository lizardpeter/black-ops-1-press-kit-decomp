import {chromium} from 'playwright';
import {mkdir,writeFile} from 'node:fs/promises';
import {PNG} from 'pngjs';
const out=process.argv[2];await mkdir(out,{recursive:true});
const base=['--enable-unsafe-webgpu','--enable-unsafe-swiftshader'];
const variants=[
 ['explicit-swiftshader',[...base,'--use-angle=swiftshader','--use-webgpu-adapter=swiftshader']],
 ['swiftshader',[...base,'--use-angle=swiftshader']],
 ['vulkan',[...base,'--use-angle=swiftshader','--enable-features=Vulkan','--disable-vulkan-surface']],
 ['default',base],
];
function stats(buf){const p=PNG.sync.read(buf);let bright=0,opaque=0,red=0;const colors=new Set();for(let i=0;i<p.data.length;i+=4){const [r,g,b,a]=p.data.subarray(i,i+4);bright+=Math.max(r,g,b)>32;opaque+=a===255;red+=r>200&&g<25&&b<25;if(a>240&&Math.max(r,g,b)>32)colors.add((r<<16)|(g<<8)|b);}const n=p.width*p.height;return {w:p.width,h:p.height,bright:bright/n,opaque:opaque/n,red:red/n,colors:colors.size};}
let results=[];
for(const [name,args] of variants){
 let browser;const result={name,args,errors:[]};
 try{
  browser=await chromium.launch({channel:'chromium',headless:true,args});
  // Fresh page and fresh GPU device, with no WASM module or app initialization.
  const direct=await browser.newPage({viewport:{width:640,height:360},deviceScaleFactor:1});
  direct.on('console',m=>console.log(name,'direct',m.type(),m.text()));
  await direct.goto('http://127.0.0.1:8766/BUILD_INFO.txt');
  result.direct=await direct.evaluate(async()=>{
    document.body.innerHTML='';document.body.style.cssText='margin:0;background:black';
    const c=document.createElement('canvas');c.width=640;c.height=360;document.body.appendChild(c);
    const a=await navigator.gpu.requestAdapter();if(!a)throw Error('no adapter');const d=await a.requestDevice();
    const errors=[];d.addEventListener('uncapturederror',e=>errors.push(e.error.message));
    d.pushErrorScope('validation');
    const format=navigator.gpu.getPreferredCanvasFormat();
    const ctx=c.getContext('webgpu');
    ctx.configure({device:d,format,alphaMode:'opaque',usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.COPY_SRC});
    const t=ctx.getCurrentTexture();
    const off=d.createTexture({size:[4,4],format,usage:GPUTextureUsage.RENDER_ATTACHMENT|GPUTextureUsage.COPY_SRC});
    const b=d.createBuffer({size:512,usage:GPUBufferUsage.MAP_READ|GPUBufferUsage.COPY_DST});
    const e=d.createCommandEncoder();
    for(const tex of [off,t]){
      const p=e.beginRenderPass({colorAttachments:[{view:tex.createView(),clearValue:{r:1,g:0,b:0,a:1},loadOp:'clear',storeOp:'store'}]});p.end();
    }
    for(const [tex,offset] of [[off,0],[t,256]])e.copyTextureToBuffer({texture:tex},{buffer:b,offset,bytesPerRow:256},{width:1,height:1});
    d.queue.submit([e.finish()]);await b.mapAsync(GPUMapMode.READ);
    const bytes=new Uint8Array(b.getMappedRange());const offscreen=[...bytes.slice(0,4)],surface=[...bytes.slice(256,260)];b.unmap();
    const validation=await d.popErrorScope();window.keepGPU=[a,d,ctx,t];
    return {format,offscreen,surface,validation:validation?.message??null,errors,adapter:{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}};
  });
  const screenshot=await direct.screenshot({path:out+'/'+name+'-direct.png'});result.direct.screenshot=stats(screenshot);
  const expected=result.direct.format.startsWith('bgra')?[0,0,255,255]:[255,0,0,255];
  result.direct.ok=JSON.stringify(result.direct.offscreen)===JSON.stringify(expected)&&JSON.stringify(result.direct.surface)===JSON.stringify(expected)&&!result.direct.validation&&!result.direct.errors.length&&result.direct.screenshot.red>0.9;
  await direct.close();
  if(!result.direct.ok)throw Error('Independent WebGPU clear/readback/presentation failed');
  // Only a verified browser backend can be used to judge the application.
  const page=await browser.newPage({viewport:{width:1280,height:720},deviceScaleFactor:1});
  page.on('console',m=>{console.log(name,'app',m.type(),m.text());if(m.type()==='error')result.errors.push(m.text());});
  page.on('pageerror',e=>result.errors.push(String(e)));
  await page.goto('http://127.0.0.1:8766/?debug=1');
  await page.waitForFunction(()=>window.presskit?.player.ready(),{},{timeout:20000});
  await page.waitForTimeout(500);
  result.app=stats(await page.screenshot({path:out+'/'+name+'-app.png'}));
  result.app.ok=result.errors.length===0&&result.app.bright>0.015&&result.app.opaque>0.9&&result.app.colors>100;
 }catch(e){result.error=String(e);}
 finally{await browser?.close();results.push(result);console.log(JSON.stringify(result));}
}
await writeFile(out+'/probe.json',JSON.stringify(results,null,2));
if(!results.some(x=>x.direct?.ok))throw Error('No verified WebGPU browser backend: independent pixel readback or presentation failed');
if(!results.some(x=>x.direct?.ok&&x.app?.ok))throw Error('Independent WebGPU passed but actual press-kit presentation failed');
