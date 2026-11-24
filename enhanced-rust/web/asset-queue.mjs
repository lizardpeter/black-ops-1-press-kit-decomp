// Bounded, demand-first fetch/decode queue. Visible assets displace speculative work.
export class AssetQueue {
  constructor(load, {limit=4, retryMs=1500, maxAttempts=3, onError=()=>{}}={}) {
    this.load=load;this.limit=limit;this.retryMs=retryMs;this.maxAttempts=maxAttempts;this.onError=onError;
    this.active=new Map();this.failures=new Map();
  }
  pump(required,ahead,now=performance.now()) {
    const demand=[...new Set(required)];
    const demandSet=new Set(demand);
    const prefetch=[...new Set(ahead)].filter(path=>!demandSet.has(path));
    const wanted=new Set([...demand,...prefetch]);
    const skipped=new Set();

    for(const [path,job] of this.active) {
      if(!wanted.has(path)) {
        job.controller.abort();
        this.active.delete(path);
      } else if(demandSet.has(path)) {
        job.priority=0;
      }
    }

    const start=(path,priority)=>{
      if(this.active.has(path))return;
      const fail=this.failures.get(path);
      if(fail&&(fail.count>=this.maxAttempts||now<fail.retryAt))return;
      const controller=new AbortController();
      const job={controller,priority};
      const promise=Promise.resolve().then(()=>this.load(path,{signal:controller.signal}))
        .then(()=>this.failures.delete(path))
        .catch(error=>{
          if(controller.signal.aborted||error?.name==='AbortError')return;
          const count=(this.failures.get(path)?.count||0)+1;
          this.failures.set(path,{count,retryAt:now+this.retryMs*count});
          if(count>=this.maxAttempts)this.onError(path,error);
        })
        .finally(()=>{if(this.active.get(path)===job)this.active.delete(path);});
      job.promise=promise;
      this.active.set(path,job);
    };

    // Required work may evict in-flight prefetch even if that prefetch remains
    // useful later. It will be reconsidered on the next pump.
    for(const path of demand) {
      if(this.active.has(path))continue;
      while(this.active.size>=this.limit) {
        const candidate=[...this.active].find(([activePath])=>!demandSet.has(activePath));
        if(!candidate)break;
        candidate[1].controller.abort();
        this.active.delete(candidate[0]);
        skipped.add(candidate[0]);
      }
      if(this.active.size>=this.limit)break;
      start(path,0);
    }
    for(const path of prefetch) {
      if(this.active.size>=this.limit)break;
      if(!skipped.has(path))start(path,1);
    }
  }
}
