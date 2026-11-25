import {test} from 'node:test';
import assert from 'node:assert/strict';
import {installSourceForeground} from './source-foreground.mjs';
const data=(bytes)=>({ok:true,arrayBuffer:async()=>Uint8Array.from(bytes).buffer});
test('PFG7 and later load menu when the decoder requires it',async()=>{
 for(const version of [55,56,57,58]){
  const calls=[];let installed=false;
  const player={install_foreground(bytes){assert.equal(bytes[3],version);return true;},install_menu(bytes){assert.deepEqual([...bytes],[1,2,3]);installed=true;}};
  assert.equal(await installSourceForeground(player,async p=>{calls.push(p);return calls.length===1?data([80,70,71,version]):data([1,2,3]);}),true);
  assert.equal(installed,true);assert.equal(calls.length,2);
 }
});
test('missing required menu is fatal, while a legacy pack needs no menu',async()=>{
 let n=0;const fetch=async()=>++n===1?data([80,70,71,49]):{ok:false};
 await assert.rejects(()=>installSourceForeground({install_foreground:()=>true},fetch),/Missing source menu/);
 n=0;assert.equal(await installSourceForeground({install_foreground:()=>false},fetch),true);assert.equal(n,1);
});
