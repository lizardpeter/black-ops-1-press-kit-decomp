"""Measure the source mask/blur fixture independently of the runtime renderer."""
import sys
from pathlib import Path
from PIL import Image,ImageFilter,ImageChops,ImageStat
root=Path(sys.argv[1])
# Three button interiors exclude header/footer and mask edges.
regions=[(250,145,410,211),(250,267,410,333),(250,389,410,455)]
def load(name):
    im=Image.open(root/name).convert('RGB');assert im.size==(1280,720);return im
def energy(im):return sum(sum(ImageStat.Stat(im.crop(r)).mean) for r in regions)/9
for engine in ('ffdec','ruffle'):
    plain=load('sidebar-plain-'+engine+'.png')
    blur=load('sidebar-blur-'+engine+'.png')
    p,b=energy(plain),energy(blur)
    assert p>5,(engine,'empty unfiltered button reference',p)
    print(f'{engine}: plain button mean {p:.3f}; filtered {b:.3f}; retained {b/p:.3%}')
    # A vertical 15px box blur cannot erase the interior of a large masked button.
    # This measures evidence without changing the production parity threshold.
    if b/p<.1:print(f'{engine}: masked button interiors lost during filtering')

plain=load('sidebar-plain-ffdec.png');flat=load('sidebar-flat-plain-ffdec.png');blur=load('sidebar-flat-blur-ffdec.png')
mae=sum(ImageStat.Stat(ImageChops.difference(plain,flat)).mean)/3
print(f'Flattened mask reference plain MAE {mae:.5f}/255; blurred button energy {energy(blur):.4f}/{energy(flat):.4f}')
assert mae<0.15,'reference rewrite changed unfiltered source art'
assert energy(blur)>0.8*energy(flat),'reference rewrite does not repair filtered button loss'
assert ImageChops.difference(flat,blur).getbbox() is not None,'reference rewrite lost blur'
