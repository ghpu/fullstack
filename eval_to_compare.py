#!/usr/bin/env python3
# coding: utf-8

import sys
import os
import json

mapping = sys.argv[1]
input = sys.argv[2]
tsv = sys.argv[3]
out = sys.argv[4]

# ./eval_to_compare.py /media/ghpu/EA6A-0BC9/20190513_fut-dispatcher_20190318-20190331_DB-v4.0-v313_NOOKD_Post-Re-Annotation/intent_mapping.txt /media/ghpu/EA6A-0BC9/20190513_fut-dispatcher_20190318-20190331_DB-v4.0-v313_NOOKD_Post-Re-Annotation/input.txt /media/ghpu/EA6A-0BC9/20190513_fut-dispatcher_20190318-20190331_DB-v4.0-v313_NOOKD_Post-Re-Annotation/result.tsv result.json

    
def decode(annotation):
    annots = [x.split("#",maxsplit=2) for x in annotation.split("|")]
    ANNOT = {"intent":"", "values":[]}
    for annot in annots:
        ANNOT["intent"] = annot[0]
        if annot[1]!="":
            ANNOT["values"].append([annot[1],annot[2]])
    return [ANNOT]


f = open(mapping,"r",encoding="utf-8")
MAPPING={}
for line in f:
    line=line.strip()
    if not line:
        continue
    skill, intent = line.split()
    MAPPING[intent] = skill


f = open(tsv,"r",encoding="utf-8")
ANNOTATIONS={}
for line in f:
    line=line.strip()
    if not line:
        continue
    fields = line.split("\t")
    ref = fields[0].split("#")[1]
    annotation = fields[4]
    ANNOTATIONS[ref] = decode(annotation)



f = open(input,"r",encoding="utf-8")
TEXTS={}
for line in f:
    line=line.strip()
    if not line:
        continue
    fields = line.split("\t")
    ref = fields[0].split("#")[1]
    text = fields[3]
    gold = decode(fields[5])
    print(gold)
    count = fields[6]
    TEXTS[ref] = {"text": text, "gold": gold, "count": count, "annotation": ANNOTATIONS[ref]}

CASES=[]

for k,v in TEXTS.items():
    case = { "text":v["text"], "reference":int(k), "count":int(v["count"]), "gold":v["gold"], "left":v["annotation"], "right":[]}
    CASES.append(case)

corpus = {"intent_mapping":{"val":MAPPING}, "cases": CASES}
g = open(out,"w",encoding="utf-8")
g.write(json.dumps(corpus,indent=4, sort_keys=True, ensure_ascii=False))


