#!/bin/bash

echo "=== DNS Debug Script for Fly.io ==="
echo ""

echo "1. Testing basic DNS resolution:"
nslookup cluster0.hrmevk3.mongodb.net

echo ""
echo "2. Testing SRV record lookup:"
nslookup -type=SRV _mongodb._tcp.cluster0.hrmevk3.mongodb.net

echo ""
echo "3. Testing with dig:"
dig cluster0.hrmevk3.mongodb.net

echo ""
echo "4. Testing connectivity:"
nc -zv cluster0.hrmevk3.mongodb.net 27017

echo ""
echo "5. Environment variables:"
echo "MONGODB_URL: $MONGODB_URL"
