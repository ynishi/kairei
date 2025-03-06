# Kairei HTTP API

REST API for the Kairei language runtime with support for system operations, agents, and events.

## Local Development

```bash
# Run the HTTP server locally
cd kairei-http
cargo run
```

## Docker Container

### Building the Docker Image

```bash
# From the project root
make docker-build
```

### Running the Container Locally

```bash
# From the project root
make docker-run

# Or manually
docker run -p 3000:3000 kairei-http:latest
```

### Testing the API

```bash
# Check health endpoint
curl http://localhost:3000/health

# List systems (requires authentication)
curl -H "Authorization: Bearer admin-key" http://localhost:3000/api/v1/systems
```

## Cloud Run Deployment

### Manual Deployment (Pre-CI/CD)

```bash
# Set GCP project
export GCP_PROJECT_ID=your-gcp-project-id

# Configure Docker to use Google Container Registry
gcloud auth configure-docker

# Build and tag the image
docker build -t gcr.io/$GCP_PROJECT_ID/kairei-http:latest .

# Push the image to Container Registry
docker push gcr.io/$GCP_PROJECT_ID/kairei-http:latest

# Deploy to Cloud Run
gcloud run deploy kairei-http \
  --image gcr.io/$GCP_PROJECT_ID/kairei-http:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --memory 512Mi \
  --cpu 1 \
  --port 3000
```

## API Endpoints

### System Management

- `GET /api/v1/systems` - List all systems
- `POST /api/v1/systems` - Create a new system
- `GET /api/v1/systems/:id` - Get system details
- `POST /api/v1/systems/:id/start` - Start a system
- `POST /api/v1/systems/:id/stop` - Stop a system
- `DELETE /api/v1/systems/:id` - Delete a system

### Additional Information

The HTTP API uses in-memory storage for sessions and user data in its initial phase.
Authentication is implemented using API keys.

Default API keys for testing:
- Admin: `admin-key`
- Regular users: `user1-key`, `user2-key`