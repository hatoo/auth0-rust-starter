# auth0-rust-starter

# Usage

## 1. Signup Auth0 and cretae some App
## 2. Run backend

    cd backend
    Authority=${YOUR_AUTH0_DOMAIN} cargo run

## 3. Run frontend

    cd frontend
    vim auth_config.json # Set domain and client_id
    cargo make build
    cargo make serve

## 4. Open http://localhost:8000
