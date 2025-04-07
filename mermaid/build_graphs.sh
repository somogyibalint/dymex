#!/bin/bash

for file in ./*.mmd
do
  echo "$file -> ${file/.mmd/.pdf}"
  mmdc -i $file -o ${file/.mmd/.pdf} --pdfFit > /dev/null
done
