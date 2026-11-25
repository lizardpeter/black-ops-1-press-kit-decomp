"""Decode source JPEG pixels in the embedded ICC space before sRGB GPU upload.
Do not alter image dimensions or synthesize artwork. Unprofiled source JPEGs
are returned as decoded RGB with no color conversion."""
import io
from PIL import Image,ImageCms

def decode_rgb(raw):
    with Image.open(io.BytesIO(raw)) as src:
        assert src.format=='JPEG'
        profile=src.info.get('icc_profile')
        rgb=src.convert('RGB')
    if not profile:return rgb,None
    embedded=ImageCms.getOpenProfile(io.BytesIO(profile))
    name=ImageCms.getProfileDescription(embedded).strip()
    converted=ImageCms.profileToProfile(rgb,embedded,ImageCms.createProfile('sRGB'),renderingIntent=ImageCms.Intent.RELATIVE_COLORIMETRIC,outputMode='RGB')
    assert converted.size==rgb.size
    # Do not leave a non-sRGB tag on already converted pixels.
    converted.info.pop('icc_profile',None)
    return converted,name
