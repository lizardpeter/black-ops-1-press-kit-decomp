import {test} from 'node:test';import assert from 'node:assert/strict';import {AssetQueue} from './asset-queue.mjs';
const flush=()=>new Promise(resolve=>setImmediate(resolve));
test('demand first, bounded, deduplicated, no stale queued jobs',async()=>{
 const started=[],release=new Map();const q=new AssetQueue(p=>new Promise(r=>{started.push(p);release.set(p,r);}),{limit:2});
 q.pump(['now','now'],['ahead','stale']);await flush();assert.deepEqual(started,['now','ahead']);
 q.pump(['now'],['other']);await flush();assert.equal(started.length,2);release.get('now')();await flush();
 q.pump(['new'],['other']);await flush();assert.deepEqual(started,['now','ahead','new']);assert.equal(q.active.size,2);
 release.get('ahead')();release.get('new')();await flush();assert.equal(q.active.size,0);
});
test('bounded retry backoff and terminal error',async()=>{
 let n=0,errors=0;const q=new AssetQueue(async()=>{n++;throw Error('offline');},{retryMs:10,maxAttempts:2,onError:()=>errors++});
 q.pump(['a'],[],0);await flush();q.pump(['a'],[],5);await flush();assert.equal(n,1);
 q.pump(['a'],[],10);await flush();q.pump(['a'],[],1000);await flush();assert.equal(n,2);assert.equal(errors,1);
});
