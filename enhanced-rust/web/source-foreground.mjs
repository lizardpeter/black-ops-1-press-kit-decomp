// Let the actual Rust pack decoder determine whether a menu is required.
// Never infer the contract from a growing list of JavaScript magic numbers.
export async function installSourceForeground(player,fetchAsset){
 const response=await fetchAsset('art/foreground-source/timeline.pfg');
 if(!response.ok)return false;
 const needsMenu=player.install_foreground(new Uint8Array(await response.arrayBuffer()));
 if(needsMenu){
  const menu=await fetchAsset('art/foreground-source/menu.pkm');
  if(!menu.ok)throw Error('Missing source menu vector pack');
  player.install_menu(new Uint8Array(await menu.arrayBuffer()));
 }
 return true;
}
