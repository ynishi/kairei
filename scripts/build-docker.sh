#!/bin/bash
set -e

# Build the Docker image
docker build -t kairei-http:latest .

echo "Docker image built successfully: kairei-http:latest"
echo "To run locally: docker run -p 3000:3000 kairei-http:latest"