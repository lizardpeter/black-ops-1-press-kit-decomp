import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createAssetResolver} from './asset-urls.mjs';

test('all original payload URLs resolve to the same-origin static site',()=>{
  const asset=createAssetResolver(new URL('https://presskit.example.test/preview/assets/'));
  assert.equal(asset('payload/intro/IntroSeq.mp4'),'https://presskit.example.test/preview/assets/payload/intro/IntroSeq.mp4');
  assert.equal(asset('payload/movies/6.Wagermatch.mp4'),'https://presskit.example.test/preview/assets/payload/movies/6.Wagermatch.mp4');
  assert.equal(asset('payload/screens/screens.xml'),'https://presskit.example.test/preview/assets/payload/screens/screens.xml');
  assert.equal(asset('payload/screens/20.jpg'),'https://presskit.example.test/preview/assets/payload/screens/20.jpg');
  assert.equal(asset('payload/wallpaper/2560x1440/COD7-Wallpaper-2560x1440.jpg'),'https://presskit.example.test/preview/assets/payload/wallpaper/2560x1440/COD7-Wallpaper-2560x1440.jpg');
  assert.equal(asset('art/background-source/timeline.pbg'),'https://presskit.example.test/preview/assets/art/background-source/timeline.pbg');
});

test('rejects absolute and traversal paths',()=>{
  const asset=createAssetResolver(new URL('https://presskit.example.test/assets/'));
  for(const path of ['/payload/screens/1.jpg','payload/../secret','art/./file.png',''])assert.throws(()=>asset(path));
});
