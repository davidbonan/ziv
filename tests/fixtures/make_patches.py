import zlib, struct, subprocess, sys
out = sys.argv[1]
BLOCK = 16
grid = [[(255,0,0),(0,255,0),(0,0,255)],[(255,255,255),(0,0,0),(128,128,128)]]
rows = b''
for row in grid:
    line = b'\x00' + b''.join(bytes(c)*BLOCK for c in row)
    rows += line*BLOCK
def chunk(tag, body):
    return struct.pack('>I', len(body)) + tag + body + struct.pack('>I', zlib.crc32(tag+body))
png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 48, 32, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b'')
open(f'{out}/patches.png','wb').write(png)
subprocess.run(['sips','-s','format','jpeg','-s','formatOptions','100',f'{out}/patches.png','--out',f'{out}/patches.jpg'],check=True,capture_output=True)
subprocess.run(['sips','-s','format','tiff',f'{out}/patches.png','--out',f'{out}/patches.tiff'],check=True,capture_output=True)
jpeg = open(f'{out}/patches.jpg','rb').read()
tiff = b'MM\x00\x2a\x00\x00\x00\x08' + b'\x00\x01' + b'\x01\x12\x00\x03\x00\x00\x00\x01\x00\x06\x00\x00' + b'\x00\x00\x00\x00'
exif = b'Exif\x00\x00' + tiff
app1 = b'\xff\xe1' + struct.pack('>H', len(exif)+2) + exif
# drop any existing APP1 so ours is the only orientation
body = jpeg[2:]
segments = b''
i = 0
while body[i] == 0xff and body[i+1] != 0xda:
    length = struct.unpack('>H', body[i+2:i+4])[0]
    if body[i+1] != 0xe1:
        segments += body[i:i+2+length]
    i += 2+length
open(f'{out}/patches_rotated_90_cw.jpg','wb').write(b'\xff\xd8' + app1 + segments + body[i:])
