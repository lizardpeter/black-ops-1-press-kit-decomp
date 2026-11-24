// Bounded, demand-first fetch/decode queue. No accumulated stale prefetch list.
export class AssetQueue {
  constructor(load, {limit=4, retryMs=1500, maxAttempts=3, onError=()=>{}}={}) {
    this.load=load;this.limit=limit;this.retryMs=retryMs;this.maxAttempts=maxAttempts;this.onError=onError;
    this.active=new Map();this.failures=new Map();
  }
  pump(required,ahead,now=performance.now()) {
    const wanted=[...new Set([...required,...ahead])];
    for (const path of wanted) {
      if(this.active.size>=this.limit)break;
      if(this.active.has(path))continue;
      const fail=this.failures.get(path);
      if(fail&&(fail.count>=this.maxAttempts||now<fail.retryAt))continue;
      const promise=Promise.resolve().then(()=>this.load(path)).then(()=>this.failures.delete(path)).catch(error=>{
        const count=(this.failures.get(path)?.count||0)+1;
        this.failures.set(path,{count,retryAt:now+this.retryMs*count});
        if(count>=this.maxAttempts)this.onError(path,error);
      }).finally(()=>this.active.delete(path));
      this.active.set(path,promise);
    }
  }
}
