# Rust Real-Time Chat Application

A modern, real-time chat application built with Rust and WebSocket technology. This application allows users to create chat rooms and communicate in real-time with features like typing indicators and room sharing.

![Rust Chat App](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![WebSocket](https://img.shields.io/badge/WebSocket-000000?style=for-the-badge&logo=websocket&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-2496ED?style=for-the-badge&logo=docker&logoColor=white)

## Features

- 🚀 Real-time messaging using WebSocket
- 🎯 Create and join chat rooms
- 👥 Multiple users per room
- 📝 Typing indicators
- 🔗 Shareable room links
- 🎨 Modern and responsive UI
- 🔒 Secure WebSocket connections (WSS)
- 🐳 Docker containerization
- 🔄 Automatic room creation
- 📱 Mobile-friendly design

## Tech Stack

- **Backend**: Rust with Actix-web framework
- **Frontend**: HTML, CSS, JavaScript
- **Real-time Communication**: WebSocket
- **Containerization**: Docker
- **Reverse Proxy**: Nginx

## Prerequisites

- Rust (1.82 or higher)
- Docker
- Nginx (for production deployment)
- SSL certificates (for HTTPS)

## Quick Start

### Using Docker

1. Build the Docker image:
```bash
docker build -t rust-chat-app .
```

2. Run the container:
```bash
docker run -d -p 8087:8087 rust-chat-app
```

The application will be available at `http://localhost:8087`

### Manual Setup

1. Clone the repository:
```bash
git clone https://github.com/yourusername/rust-chat-app.git
cd rust-chat-app
```

2. Build the application:
```bash
cargo build --release
```

3. Run the application:
```bash
./target/release/rust-chat-app
```

## Production Deployment

### Nginx Configuration

For production deployment with HTTPS and WebSocket support, use this Nginx configuration:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com www.your-domain.com;
    
    ssl_certificate     /path/to/your/fullchain.pem;
    ssl_certificate_key /path/to/your/privkey.pem;

    # WebSocket proxy settings
    location /ws/ {
        proxy_pass http://localhost:8087;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Regular HTTP proxy settings
    location / {
        proxy_pass http://localhost:8087;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Docker Deployment

1. Build the production image:
```bash
docker build -t rust-chat-app:prod .
```

2. Run the container:
```bash
docker run -d -p 8087:8087 rust-chat-app:prod
```

## Development

### Project Structure

```
rust-chat-app/
├── src/
│   └── main.rs          # Main application code
├── static/
│   └── index.html       # Frontend code
├── Dockerfile           # Docker configuration
├── nginx.conf          # Nginx configuration
└── Cargo.toml          # Rust dependencies
```

### Adding Features

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Actix-web](https://actix.rs/) - Web framework
- [WebSocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket) - Real-time communication
- [Docker](https://www.docker.com/) - Containerization

## Support

If you encounter any issues or have questions, please open an issue in the GitHub repository.
