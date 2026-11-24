import {test} from 'node:test';import assert from 'node:assert/strict';import {readFile} from 'node:fs/promises';
import {initSync,wasm_core_smoke,wasm_validate_foreground} from './pkg/presskit_web.js';
test('actual compiled WASM executes original application flow',async()=>{
 const module=await readFile(new URL('./pkg/presskit_web_bg.wasm',import.meta.url));initSync({module});assert.equal(wasm_core_smoke(),true);
});

test('compiled WASM parses original PFG3 JPEG placements and rejects corrupted assets', {skip:!process.env.PRESSKIT_FG_ASSETS},async()=>{
 const root=process.env.PRESSKIT_FG_ASSETS;
 const timeline=await readFile(root+'/timeline.pfg'),menu=await readFile(root+'/menu.pkm');
 const module=await readFile(new URL('./pkg/presskit_web_bg.wasm',import.meta.url));initSync({module});
 assert.equal(wasm_validate_foreground(timeline,menu),26);
 assert.throws(()=>wasm_validate_foreground(timeline.subarray(0,timeline.length-1),menu));
 assert.throws(()=>wasm_validate_foreground(timeline,menu.subarray(0,menu.length-1)));
 const wrong=Buffer.from(timeline);wrong[12]=2;assert.throws(()=>wasm_validate_foreground(wrong,menu));
});
