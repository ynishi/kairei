# KAIREI GCP Integration Guide

This guide provides instructions for deploying KAIREI to Google Cloud Platform (GCP) using Cloud Build and Cloud Run with Secret Manager integration.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Repository Setup](#repository-setup)
3. [Secret Manager Setup](#secret-manager-setup)
4. [Cloud Build Configuration](#cloud-build-configuration)
5. [Dockerfile Overview](#dockerfile-overview)
6. [Deployment Process](#deployment-process)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

Before deploying KAIREI to GCP, ensure you have:

- A Google Cloud Platform account with billing enabled
- The following APIs enabled in your GCP project:
  - Cloud Build API
  - Cloud Run API
  - Secret Manager API
  - Container Registry API
- Google Cloud SDK (gcloud) installed and configured
- Appropriate IAM permissions:
  - Secret Manager Admin (`roles/secretmanager.admin`)
  - Cloud Build Editor (`roles/cloudbuild.builds.editor`)
  - Cloud Run Admin (`roles/run.admin`)
  - Service Account User (`roles/iam.serviceAccountUser`)

## Repository Setup

1. Clone the KAIREI repository:
   ```bash
   git clone https://github.com/ynishi/kairei.git
   cd kairei
   ```

2. Ensure the repository contains the required deployment files:
   - `cloudbuild.yaml` - Cloud Build configuration
   - `Dockerfile` - Container image definition
   - `scripts/setup-secret.sh` - Secret Manager setup script

## Secret Manager Setup

KAIREI uses Google Cloud Secret Manager to securely store sensitive information like API keys and service credentials.

### Creating Secrets

1. Create a JSON file containing your secrets:
   ```json
   {
     "admin_service_key": "your-admin-key",
     "user_service_key": "your-user-key"
   }
   ```

2. Use the provided script to create a secret in Secret Manager:
   ```bash
   export SECRET_NAME="KAIREI_HTTP_DEV"
   export SECRET_FILE_PATH="/path/to/your/secret.json"
   bash scripts/setup-secret.sh
   ```

   Alternatively, use gcloud commands directly:
   ```bash
   # Create the secret
   gcloud secrets create KAIREI_HTTP_DEV --replication-policy="automatic"
   
   # Add a version with your secret data
   gcloud secrets versions add KAIREI_HTTP_DEV --data-file=/path/to/your/secret.json
   ```

3. Grant access to the Cloud Run service account:
   ```bash
   # Get your project number
   PROJECT_NUMBER=$(gcloud projects describe $(gcloud config get-value project) --format='value(projectNumber)')
   
   # Grant access to the Cloud Run service account
   gcloud secrets add-iam-policy-binding KAIREI_HTTP_DEV \
     --member="serviceAccount:service-${PROJECT_NUMBER}@serverless-robot-prod.iam.gserviceaccount.com" \
     --role="roles/secretmanager.secretAccessor"
   ```

## Cloud Build Configuration

The `cloudbuild.yaml` file defines the build and deployment process:

```yaml
steps:
  - name: gcr.io/cloud-builders/docker
    args:
      - build
      - "-t"
      - gcr.io/$PROJECT_ID/kairei-http
      - .
  - name: gcr.io/cloud-builders/docker
    args:
      - push
      - gcr.io/$PROJECT_ID/kairei-http
  - name: gcr.io/cloud-builders/gcloud
    args:
      - run
      - deploy
      - kairei-http
      - "--image"
      - gcr.io/$PROJECT_ID/kairei-http
      - "--region"
      - $_REGION
      - "--set-secrets=/etc/secrets/kairei-secret.json=KAIREI_HTTP_DEV:latest"
options:
  logging: CLOUD_LOGGING_ONLY
substitutions:
  _REGION: asia-northeast1
```

This configuration:
1. Builds a Docker image using the Dockerfile
2. Pushes the image to Container Registry
3. Deploys the image to Cloud Run
4. Mounts the secret at `/etc/secrets/kairei-secret.json`

### Customization Options

- `$_REGION`: Deployment region (default: asia-northeast1)
- `kairei-http`: Service name
- `KAIREI_HTTP_DEV`: Secret name

## Dockerfile Overview

The Dockerfile uses a multi-stage build process to create a minimal and secure container:

```dockerfile
# Stage 1: Build the application
FROM --platform=linux/amd64 rust:1.85-slim-bookworm as builder

WORKDIR /usr/src/kairei
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY kairei-core ./kairei-core
COPY kairei-http ./kairei-http
COPY kairei-cli  ./kairei-cli

RUN cargo build --release --bin kairei-http

# Stage 2: Create a minimal runtime image
FROM --platform=linux/amd64 debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Create a non-root user for running the application
RUN useradd -ms /bin/bash kairei
USER kairei

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/kairei/target/release/kairei-http /app/

# Expose the API port
EXPOSE 8080

ENV RUST_LOG=info

# Run the server
CMD ["/app/kairei-http", "--host", "0.0.0.0", "--port", "8080"]
```

Key features:
- Multi-stage build to minimize image size
- Non-root user for improved security
- Minimal dependencies in the final image
- Proper port exposure (8080)

## Deployment Process

### Manual Deployment

1. Build and push the Docker image:
   ```bash
   docker build -t gcr.io/[PROJECT_ID]/kairei-http .
   docker push gcr.io/[PROJECT_ID]/kairei-http
   ```

2. Deploy to Cloud Run:
   ```bash
   gcloud run deploy kairei-http \
     --image gcr.io/[PROJECT_ID]/kairei-http \
     --region asia-northeast1 \
     --platform managed \
     --set-secrets=/etc/secrets/kairei-secret.json=KAIREI_HTTP_DEV:latest
   ```

### Automated Deployment with Cloud Build

1. Configure a Cloud Build trigger:
   ```bash
   gcloud builds triggers create github \
     --repo=ynishi/kairei \
     --branch-pattern=main \
     --build-config=cloudbuild.yaml
   ```

2. Push changes to the main branch to trigger a build and deployment.

## Troubleshooting

### Secret Access Issues

If the application cannot access secrets:

1. Verify the secret exists:
   ```bash
   gcloud secrets list
   ```

2. Check IAM permissions:
   ```bash
   gcloud secrets get-iam-policy KAIREI_HTTP_DEV
   ```

3. Ensure the secret is mounted correctly in Cloud Run:
   ```bash
   gcloud run services describe kairei-http
   ```

### Container Build Failures

1. Check Cloud Build logs:
   ```bash
   gcloud builds list
   gcloud builds log [BUILD_ID]
   ```

2. Verify Dockerfile syntax and dependencies.

### Cloud Run Deployment Issues

1. Check service status:
   ```bash
   gcloud run services describe kairei-http
   ```

2. View service logs:
   ```bash
   gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=kairei-http"
   ```

3. Verify the container starts correctly:
   ```bash
   docker run -p 8080:8080 gcr.io/[PROJECT_ID]/kairei-http
   ```

## Secret Management Best Practices

When working with secrets in GCP and KAIREI:

1. **Principle of Least Privilege**:
   - Grant minimal necessary permissions to service accounts
   - Use IAM Conditions to restrict access by time, resource attributes, etc.

2. **Secret Rotation**:
   - Implement regular secret rotation
   - Use secret versions to manage rotation without downtime

3. **Monitoring and Auditing**:
   - Enable audit logging for Secret Manager
   - Set up alerts for suspicious access patterns

4. **Development Practices**:
   - Never commit secrets to source control
   - Use different secrets for development, staging, and production
   - Consider using Secret Manager's automatic replication for multi-region deployments

5. **Application Integration**:
   - Mount secrets as files in Cloud Run (as implemented in cloudbuild.yaml)
   - Handle missing secrets gracefully in the application
   - Implement proper error handling for secret access failures
