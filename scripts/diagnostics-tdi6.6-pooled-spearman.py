import random
def average_ranks(v):
    idx=sorted(range(len(v)),key=lambda i:(v[i],i)); r=[0.0]*len(v); s=0
    while s<len(idx):
        e=s+1
        while e<len(idx) and v[idx[e]]==v[idx[s]]: e+=1
        a=(s+1+e)/2.0
        for i in idx[s:e]: r[i]=a
        s=e
    return r
def pearson(a,b):
    n=len(a);ma=sum(a)/n;mb=sum(b)/n
    num=sum((x-ma)*(y-mb) for x,y in zip(a,b))
    da=sum((x-ma)**2 for x in a)**.5; db=sum((y-mb)**2 for y in b)**.5
    return num/(da*db)

random.seed(7)
N=5000
# Three blocks. Per-block SOURCE scalers (arm A1) and TARGET scalers (arm A2)
# differ from each other — that is the whole point.
src=[(4.10,1.02),(4.13,0.99),(4.12,1.01)]
tgt=[(0.71,0.73),(0.78,0.75),(0.74,0.72)]

y=[[random.gauss(1.0,0.8) for _ in range(N)] for _ in range(3)]
pred=[[random.gauss(0,1) for _ in range(N)] for _ in range(3)]

def pooled(scalers):
    t=[];p=[]
    for b in range(3):
        m,s=scalers[b]
        t+= [(v-m)/s for v in y[b]]
        p+= pred[b]              # predictions identical in both arms
    return pearson(average_ranks(t),average_ranks(p))

a1=pooled(src); a2=pooled(tgt)
print(f"Spearman poolé A1 = {a1:.12f}")
print(f"Spearman poolé A2 = {a2:.12f}")
print(f"écart A1-A2       = {a1-a2:+.3e}")

# Control: identical scalers across blocks -> must be exactly equal.
flat_s=[(4.12,1.0)]*3; flat_t=[(0.74,0.73)]*3
print(f"contrôle (scalers identiques par bloc) écart = {pooled(flat_s)-pooled(flat_t):+.3e}")
