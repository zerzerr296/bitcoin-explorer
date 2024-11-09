# Bitcoin Explorer

Bitcoin Explorer is a real-time data visualization tool that displays on-chain and off-chain metrics of Bitcoin using WebSockets and API integrations. The project includes both backend and frontend components packaged in Docker.

## Features

- Displays real-time Bitcoin block height, transaction count, and price.
- Visualizes data using dynamic line charts.
- Fetches and updates data automatically every 60 seconds.
- Built using Rust for the backend, React for the frontend, and MySQL for database storage.

## Technologies

- **Frontend**: React, TypeScript, Recharts (for charts)
- **Backend**: Rust, Warp, MySQL, WebSockets, API calls to external Bitcoin services
- **Database**: MySQL
- **CI/CD**: GitHub Actions, DockerHub

## Setup

1. **Clone the repository**:
    ```bash
    git clone git@github.com:zerzerr296/bitcoin-explorer.git
    cd bitcoin-explorer
    ```
aad
2. **Configure Docker and MySQL**:
    - Set up your `.env` file for MySQL configuration.
    - Run `docker-compose up` to start the services.

3. **Run the app**:
    - Frontend will be available at `http://localhost:80`.
    - Backend runs at `http://localhost:3030/ws`.

4. **Trigger CI/CD**:
    - Pushing changes to the `main` branch triggers GitHub Actions for CI/CD.
