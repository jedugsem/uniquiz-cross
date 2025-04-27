#!/bin/sh
rm data/sol/FG_1.txt
rm data/sol/FG_2.txt
rm data/sol/FG_3.txt
rm data/sol/FG_4.txt
rm data/sol/FG_5.txt
n=1
for i in data/images/* 
do
  for f in $i/*; do ocrad ${f} -F utf8 >> data/sol/FG_${n}.txt ; done;
  n=$((n + 1))
done
