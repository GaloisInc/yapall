#!/bin/bash

for i in src/*.rs src/*/*.rs
do
  cat copyright.txt $i >$i.new && mv $i.new $i
done

