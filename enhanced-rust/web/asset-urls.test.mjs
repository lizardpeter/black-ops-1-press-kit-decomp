import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createAssetResolver,DEFAULT_R2_PAYLOAD_BASE} from './asset-urls.mjs';

test('original payload resolves directly to the public R2 bucket URL',()=>{
  const asset=createAssetResolver(new URL('https://presskit.example.test/assets/'));
  assert.equal(asset('payload/movies/1.BlackOpsMPTeaser.mp4'),'https://r2.houseofkublai.com/bo1-press-kit/payload/movies/1.BlackOpsMPTeaser.mp4');
  assert.equal(asset('payload/screens/screens.xml'),'https://r2.houseofkublai.com/bo1-press-kit/payload/screens/screens.xml');
  assert.equal(asset('payload/screens/20.jpg'),'https://r2.houseofkublai.com/bo1-press-kit/payload/screens/20.jpg');
  assert.equal(DEFAULT_R2_PAYLOAD_BASE.href,'https://r2.houseofkublai.com/bo1-press-kit/payload/');
});

test('small reconstructed runtime assets stay with the static website',()=>{
  const asset=createAssetResolver(new URL('https://presskit.example.test/preview/assets/'));
  assert.equal(asset('art/background-source/0.jpg'),'https://presskit.example.test/preview/assets/art/background-source/0.jpg');
  assert.equal(asset('art/foreground-source/timeline.pfg'),'https://presskit.example.test/preview/assets/art/foreground-source/timeline.pfg');
});

test('rejects absolute and traversal paths',()=>{
  const asset=createAssetResolver(new URL('https://presskit.example.test/assets/'));
  for(const path of ['/payload/screens/1.jpg','payload/../secret','art/./file.png',''])assert.throws(()=>asset(path));
});
