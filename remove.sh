#!/bin/bash
echo "Removing Revolt container."
docker kill revolt
docker rm revolt
