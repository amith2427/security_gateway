# 3D-ULPIN: Trust, Security & Compliance Gateway

This repository houses the Trust, Security & Compliance layer for the **3D-ULPIN** architecture. It functions as a zero-trust API Gateway and security middleware, ensuring that all interactions with the 3D land registry are authenticated, rate-limited, and dynamically sanitized to protect Sensitive Personal Information (SPI).

##  System Architecture & Request Lifecycle

The system implements a two-tier microservice architecture using Docker Compose:

1. **Client Request** → Initiates an API call to access spatial/legal property data.
2. **Tier 1: Kong API Gateway (Port 8000)** → Acts as the ingress controller.
   * Enforces rate limiting (DDoS protection).
   * Injects a unique `X-Request-ID` for end-to-end traceability.
   * Forwards valid traffic to the internal Docker network.
3. **Tier 2: Rust Security Middleware (Port 8082)** → The core security engine.
   * **Authentication & RBAC:** Evaluates the user's role (e.g., `Citizen`, `GovAdmin`).
   * **Dynamic Payload Masking:** Intercepts outgoing data and actively redacts fields (like `owner_name` and `financial_liens`) based on the authorization level.
   * **Audit Logging:** Logs the transaction against the injected `X-Request-ID` for tamper-evident compliance tracking.
4. **Response** → The client receives a secure, sanitized JSON payload.

##  Core Technical Features

* **DB-Less Declarative Routing:** Kong is configured using a declarative `kong.yml` file rather than a database. This ensures rapid startup times, version-controlled infrastructure, and immutable deployments.
* **Memory-Safe Middleware:** The security logic is written in **Rust** (using the `Axum` web framework and `Tokio` runtime). This guarantees memory safety, prevents buffer overflows, and provides sub-millisecond response times.
* **Multi-Stage Docker Builds:** The Rust `Dockerfile` utilizes a multi-stage build. It compiles the source code in a heavy Rust environment, but copies only the final binary into a lightweight Debian Slim image, reducing the deployment size by over 90%.
* **Zero-Trust Networking:** The Rust middleware does not expose its ports to the host machine. It is only accessible internally by the Kong Gateway via the isolated `security_gateway_ulpin-net` Docker network.

##  Technology Stack

* **Systems Language:** Rust (Axum, Tokio, Serde)
* **API Gateway:** Kong Gateway (DB-less mode)
* **Containerization:** Docker & Docker Compose
* **Version Control:** Git

##  How to Run Locally

Ensure you have **Docker Desktop** installed and running on your machine.

 **Clone the repository:**
   ```bash
   git clone <your-private-repo-url>
   cd security_gateway
