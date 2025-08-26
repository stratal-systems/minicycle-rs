#!/bin/sh

set -e

if [ ! -r ./minicycle.toml ]
then
	echo "No minicycle.toml found, copying from sample file!"
	cp ./minicycle.example.toml ./minicycle.toml
fi

if [ ! -d ./repos ]
then
	echo "No repos dir found, making"
	mkdir -p ./repos
fi

if [ ! -d ./reports ]
then
	echo "No reports dir found, making"
	mkdir -p ./reports
fi


podman run \
	--hostname minicycle-rs \
	--name minicycle-rs \
	--init \
	--rm \
	--publish 3030:3030 \
	-v ./minicycle.toml:/app/minicycle.toml:ro \
	-v ./repos:/app/repos:rw \
	-v ./reports:/app/reports:rw \
	minicycle-rs


