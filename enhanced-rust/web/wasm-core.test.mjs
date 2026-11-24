import {test} from 'node:test';import assert from 'node:assert/strict';import {readFile} from 'node:fs/promises';
import {initSync,wasm_core_smoke} from './pkg/presskit_web.js';
test('actual compiled WASM executes original application flow',async()=>{
 const module=await readFile(new URL('./pkg/presskit_web_bg.wasm',import.meta.url));initSync({module});assert.equal(wasm_core_smoke(),true);
});
