#!/bin/bash
mkdir -p cert && cd cert
rm -rf *
/bin/bash -c "openssl req -newkey rsa:4096 -new -x509 -days 365 -nodes -out certificate.crt -keyout private.key"
cd ..