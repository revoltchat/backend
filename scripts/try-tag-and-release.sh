#!/usr/bin/env bash
date=$(date +'%Y%m%d')
incr=1

while [ $(git tag -l "$date-$incr") ]; do
    incr=$((incr+1))
done

tag=$date-$incr
echo About to tag and push $tag in 3 seconds...
sleep 3s

git tag $tag
git push --atomic origin $(git rev-parse --abbrev-ref HEAD) $tag
