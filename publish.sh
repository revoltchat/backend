#!/bin/bash
source set_version.sh

                  docker build -t revoltchat/server:${version} . &&
docker tag revoltchat/server:${version} revoltchat/server:latest &&
                        docker push revoltchat/server:${version} &&
                            docker push revoltchat/server:latest
