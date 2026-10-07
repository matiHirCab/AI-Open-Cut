"""Verify immutable hero plates using independent scalar equations, never production output."""
import math,json,hashlib
from functools import cache, wraps
from pathlib import Path
WORDS=[(2336985851,938826240,1456866664),(84929298,829401672,2771797279),(2877292033,4199349311,3397877525),(4067787588,1783773585,2445685191),(3863548237,661299992,4101546638),(2606412158,1268270209,2067976603),(2944314353,2443517269,3235942571),(2958880360,2302107493,891655201)]
SIDE=80; ORIGIN=-8


def raster_cache(function):
 """Memoize pure scalar rasters; mutable callers always receive isolated copies."""
 @cache
 def evaluate(source):
  return tuple(function(list(source)))
 @wraps(function)
 def isolated(source):
  return list(evaluate(tuple(source)))
 return isolated

def lin(v):return v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4

def encode(v):return max(0,min(255,math.floor(255*(12.92*v if v<=.0031308 else 1.055*v**(1/2.4)-.055)+.5)))

def idx(x,y):return ((y-ORIGIN)*SIDE+(x-ORIGIN))*4

def empty():return [0.0]*(SIDE*SIDE*4)

def over(dst,i,rgb,a):
 for c in range(3):dst[i+c]=rgb[c]+dst[i+c]*(1-a)
 dst[i+3]=a+dst[i+3]*(1-a)

@raster_cache
def gaussian(src):
 weights=[math.exp(-.5*k*k) for k in range(-3,4)];s=sum(weights);weights=[v/s for v in weights]
 for vertical in [False,True]:
  dst=empty()
  for y in range(SIDE):
   for x in range(SIDE):
    i=(y*SIDE+x)*4
    for k,w in zip(range(-3,4),weights):
     sx=x+(0 if vertical else k);sy=y+(k if vertical else 0)
     if 0<=sx<SIDE and 0<=sy<SIDE:
      j=(sy*SIDE+sx)*4
      for c in range(4):dst[i+c]+=src[j+c]*w
  src=dst
 return src

def tint(src):
 color=[lin(v) for v in [.15,.8,.3]]
 for i in range(0,len(src),4):
  for c in range(3):src[i+c]=src[i+c]*(1-.45)+color[c]*src[i+3]*.45
 return src

@raster_cache
def glow(src):
 blurred=gaussian(src);dst=empty();color=[lin(v) for v in [.9,.25,.1]]
 for i in range(0,len(src),4):
  a=blurred[i+3]*.6*.8
  for c in range(3):dst[i+c]=src[i+c]+color[c]*a*(1-src[i+3])
  dst[i+3]=src[i+3]+a*(1-src[i+3])
 return dst

@cache
def plate(t,variant='baseline'):
 hero=empty();width=min(48,t*.08) if variant!='no-mask' else 48
 for y in range(40):
  for x in range(48):
   if x+.5<width:
    i=idx(x,y)
    for c,v in enumerate([.2,.55,.85]):hero[i+c]=lin(math.floor(v*255+.5)/255)
    hero[i+3]=1
 hero=gaussian(hero)
 effects=['tint','glow'] if variant=='reverse' else ['glow','tint']
 for e in effects:
  if variant=='no-'+e:continue
  hero=glow(hero) if e=='glow' else tint(hero)
 group=empty()
 for y in range(20,28):
  for x in range(-6,6):over(group,idx(x,y),[lin(math.floor(v*255+.5)/255)*(191/255) for v in [.75,.125,.25]],191/255)
 for y in range(-6,46):
  for x in range(-6,54):
   gx=x+8;gy=y+12;wx=gx+4;wy=gy
   if variant!='no-matte' and not(4<=wx<48 and 8<=wy<56):continue
   j=idx(x,y);i=idx(gx,gy);over(group,i,hero[j:j+3],hero[j+3])
 if variant!='no-clip':
  for y in range(ORIGIN,ORIGIN+SIDE):
   for x in range(ORIGIN,ORIGIN+SIDE):
    if not(0<=x<64 and 0<=y<64):group[idx(x,y):idx(x,y)+4]=[0.0]*4
 if variant!='no-flash':
  strength=.65*(1-(t-300)/300) if 300<=t<600 else 0
  color=[lin(v) for v in [.8,.65,.9]]
  for i in range(0,len(group),4):
   for c in range(3):group[i+c]+=(group[i+3]-group[i+c])*color[c]*strength
 if variant!='no-particles':
  color=[lin(v) for v in [.95,.8,.2]]
  for a,b,c in WORDS:
   cx=64*a/2**32;phase=((t%700)+c/2**32*700)%700;cy=(64*b/2**32+12*phase/1000)%64
   for y in range(math.floor(cy-2)-1,math.ceil(cy+2)+1):
    for x in range(math.floor(cx-2)-1,math.ceil(cx+2)+1):
     n=sum((x+(col+.5)/4-cx)**2+(y+(row+.5)/4-cy)**2<=4 for row in range(4) for col in range(4));alpha=.75*n/16
     over(group,idx(x,y),[v*alpha for v in color],alpha)
 result=bytearray()
 for y in range(64):
  for x in range(64):
   i=idx(x-4,y);result.extend(encode(group[i+c]) for c in range(3));result.append(255)
 return bytes(result)


def main():
 import argparse,struct
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--output',type=Path,help='Create a NEW directory; never overwrite authorities')
 args=parser.parse_args()
 if args.output and args.output.exists():raise FileExistsError(args.output)
 repo=Path(__file__).resolve().parents[1]
 catalog=json.loads((repo/'contracts/masked-hero-reveal-v1.json').read_text())
 frozen=repo/'contracts/fixtures/masked-hero-reveal-v1'
 data={}
 for record in catalog['plates']['records']:
  actual=plate(record['atMs'],record['variant'])
  assert hashlib.sha256(actual).hexdigest()==record['sha256'],record
  assert actual==(frozen/record['path']).read_bytes(),record
  data[record['path']]=actual
 for witness in catalog['plates']['witnesses']:
  baseline=plate(witness['atMs']); actual=plate(witness['atMs'],witness['variant'])
  assert hashlib.sha256(actual).hexdigest()==witness['counterfactualSha256'],witness
  data[f"{witness['variant']}-{witness['atMs']:04}.rgba"]=actual
  offset=(witness['pixel'][1]*64+witness['pixel'][0])*4
  assert list(baseline[offset:offset+4])==witness['baseline']
  assert list(actual[offset:offset+4])==witness['counterfactual']
  assert max(abs(a-b) for a,b in zip(baseline[offset:offset+4],actual[offset:offset+4]))==witness['maximumByteDelta']>1
 # Cache mutation must never affect a subsequent caller or its input.
 for function in (gaussian,glow):
  source=empty();expected=tuple(function(source));changed=function(source)
  changed[0]+=1
  assert tuple(function(source))==expected
  assert not any(source)
 # Independently generate every stereo PCM sample with original source phase.
 pcm=bytearray()
 for n in range(38400):
  amp=next(amp for start,end,amp in catalog['audio']['specification']['segments'] if start*48<=n<end*48)
  for frequency in [437,659]:
   v=32767*amp*math.sin(2*math.pi*frequency*n/48000)
   sample=math.floor(v+.5) if v>=0 else math.ceil(v-.5)
   pcm.extend(struct.pack('<h',sample))
 wav=b'RIFF'+struct.pack('<I',36+len(pcm))+b'WAVEfmt '+struct.pack('<IHHIIHH',16,1,2,48000,192000,4,16)+b'data'+struct.pack('<I',len(pcm))+pcm
 assert hashlib.sha256(wav).hexdigest()==catalog['audio']['sourceSha256']
 assert wav==(frozen/'source.wav').read_bytes()
 data['source.wav']=wav
 if args.output:
  args.output.mkdir(parents=True,exist_ok=False)
  for name,value in data.items():(args.output/name).write_bytes(value)
 print('Verified 16 complete RGBA plates, 8 counterfactual witnesses and 38400 stereo PCM frames')

if __name__=='__main__':main()
