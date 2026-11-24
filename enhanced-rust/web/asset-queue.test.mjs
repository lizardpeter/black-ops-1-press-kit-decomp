import {test} from 'node:test';import assert from 'node:assert/strict';import {AssetQueue} from './asset-queue.mjs';
const flush=()=>new Promise(resolve=>setImmediate(resolve));
const abortError=()=>Object.assign(new Error('Aborted'),{name:'AbortError'});
function deferredLoader(started,signals,release){
 return (path,{signal})=>new Promise((resolve,reject)=>{
  started.push(path);signals.set(path,signal);release.set(path,resolve);
  signal.addEventListener('abort',()=>reject(abortError()),{once:true});
 });
}
test('demand first, bounded, deduplicated, and obsolete work is cancelled',async()=>{
 const started=[],signals=new Map(),release=new Map();
 const q=new AssetQueue(deferredLoader(started,signals,release),{limit:2});
 q.pump(['now','now'],['ahead','stale']);await flush();
 assert.deepEqual(started,['now','ahead']);
 q.pump(['now'],['other']);await flush();
 assert.deepEqual(started,['now','ahead','other']);
 assert.equal(signals.get('ahead').aborted,true);
 assert.equal(q.active.size,2);
 release.get('now')();await flush();
 q.pump(['new'],['other']);await flush();
 assert.deepEqual(started,['now','ahead','other','new']);
 assert.equal(q.active.size,2);
 release.get('other')();release.get('new')();await flush();
 assert.equal(q.active.size,0);
});
test('visible demand evicts active prefetch even if it remains in lookahead',async()=>{
 const started=[],signals=new Map(),release=new Map();
 const q=new AssetQueue(deferredLoader(started,signals,release),{limit:2});
 q.pump([],['prefetch-a','prefetch-b']);await flush();
 q.pump(['visible'],['prefetch-a','prefetch-b']);await flush();
 assert.deepEqual(started,['prefetch-a','prefetch-b','visible']);
 assert.equal(signals.get('prefetch-a').aborted,true);
 assert.equal(q.active.has('visible'),true);
 assert.equal(q.active.has('prefetch-b'),true);
 assert.equal(q.active.size,2);
 release.get('visible')();release.get('prefetch-b')();await flush();
});
test('bounded retry backoff and terminal error',async()=>{
 let n=0,errors=0;
 const q=new AssetQueue(async()=>{n++;throw Error('offline');},{retryMs:10,maxAttempts:2,onError:()=>errors++});
 q.pump(['a'],[],0);await flush();q.pump(['a'],[],5);await flush();assert.equal(n,1);
 q.pump(['a'],[],10);await flush();q.pump(['a'],[],1000);await flush();assert.equal(n,2);assert.equal(errors,1);
});
