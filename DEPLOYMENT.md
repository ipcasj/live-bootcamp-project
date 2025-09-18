# Deployment Guide

## Prerequisites for DigitalOcean Deployment

### GitHub Secrets Configuration
Before deploying, configure these secrets in your GitHub repository:

1. **DOCKER_USERNAME** - Your Docker Hub username
2. **DOCKER_PASSWORD** - Your Docker Hub password/token
3. **JWT_SECRET** - A cryptographically secure random string (generate with: `openssl rand -base64 64`)
4. **POSTGRES_PASSWORD** - A strong PostgreSQL password
5. **DROPLET_PASSWORD** - Your DigitalOcean droplet root password

### GitHub Variables Configuration
Configure these variables in your GitHub repository:

1. **DROPLET_IP** - Your DigitalOcean droplet IP address

### Environment Setup

The `.env` file is included in the repository for simplicity with these default values:
- `POSTGRES_PASSWORD=SecurePass2024!`
- `JWT_SECRET=g4iNvB23GraeR2d1SsIDL9lxqynITs/8c9JOSL0BvY5aR6a1Lv69gl1Gq0N6vJLY5ntgpRg3WOvzqXVojUGdBA==`
- `AUTH_SERVICE_IP=auth-service`

**Note:** These values work for development and demo purposes. For production, you may want to use the same values or update them via GitHub Secrets.

### DigitalOcean Droplet Setup

1. Create a Ubuntu droplet on DigitalOcean
2. Install Docker and Docker Compose:
   ```bash
   curl -fsSL https://get.docker.com -o get-docker.sh
   sh get-docker.sh
   sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
   sudo chmod +x /usr/local/bin/docker-compose
   ```

### Database Persistence

The application uses a named Docker volume `db` for PostgreSQL data persistence. Data will survive container restarts and updates.

### Deployment Process

1. Push to main branch
2. GitHub Actions will:
   - Build and test both services
   - Create Docker images
   - Deploy to DigitalOcean droplet
   - Start services with Docker Compose

### Health Checks

- PostgreSQL: Health check ensures database is ready before starting auth-service
- Services: All services have restart policies for high availability

### Security Notes

- Never commit `.env` files with real secrets
- Use GitHub Secrets for sensitive values
- Regularly rotate JWT and database passwords
- Monitor logs for security events