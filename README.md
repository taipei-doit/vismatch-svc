# vismatch-svc

相似影像辨識服務。

`vismatch-svc` is a Rust-based microservice for image similarity matching.

## TL;DR

```bash
docker compose build
docker compose up -d
```

[Check this guide!](https://github.com/h-alice/vismatch-api-guide)

## Features

- **Image comparison**: Compare an uploaded image against a database of images to find the most similar matches.
- **Image retrieval**: Want more evidences? We provided a way to retrieve closest images to let you double check.
- **Image upload**: Add new images to a project's database.
- **Docker ready**: One-click™ docker compose deployment.

## Getting Started

### Prerequisites

- Docker needs to be installed on system.
- Since we need to build container on-site, network connection is needed.

### Deployment

1. Clone the repository.
2. Build container
  ```bash
  docker compose build
  ```
3. Start the service using Docker Compose:

    ```bash
    docker compose up -d
    ```

    The service will be available at `http://localhost:3000`.

3.  The service mounts the `./image_root` directory. Images are organized by project subdirectories within `image_root`.

    Example structure:
    ```
    image_root/
    ├── my_project/
    │   ├── image1.jpg
    │   ├── image2.png
    │   └── ...
    └── another_project/
        ├── photo.webp
        └── ...
    ```

## API Reference

Again, [Check this guide](https://github.com/h-alice/vismatch-api-guide), I'm too lazy to write API documentation.

Talk is cheap, so I show you the code.

## Project Structure

- `src/`: Rust source code.
- `image_root/`: Directory where images are stored (mounted volume in Docker).
- `Dockerfile`: Multi-stage Docker build.
- `compose.yml`: Docker Compose configuration.
