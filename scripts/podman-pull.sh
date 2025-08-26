#!/bin/sh

podman pull --creds=public:public reg.stratal.systems/minicycle-rs
podman tag reg.stratal.systems/minicycle-rs minicycle-rs

