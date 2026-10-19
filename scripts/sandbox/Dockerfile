# Grimoire Algorithmica: Air-Gapped Sandbox Runner
# Multi-runtime isolated container for executing untrusted algorithmic candidates.

FROM debian:bookworm-slim

LABEL maintainer="CodeMemoryPrime <kichiku.beihei@gmail.com>"
LABEL description="Air-Gapped Gauntlet Execution Environment"

# Avoid interactive prompts during apt install
ENV DEBIAN_FRONTEND=noninteractive

# Install minimal, hardened language runtimes: Python 3, Node.js, Go, Dart
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    python3 \
    python3-minimal \
    nodejs \
    npm \
    golang-go \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged runner user
RUN useradd -u 10001 -m -s /bin/sh -d /home/sandbox sandbox

# Workspace setup (read-only mount target)
WORKDIR /code
RUN chown -R sandbox:sandbox /code

# Drop to unprivileged user
USER sandbox

# Environment lockdown
ENV PATH="/usr/local/bin:/usr/bin:/bin"
ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

ENTRYPOINT ["/usr/bin/python3"]
