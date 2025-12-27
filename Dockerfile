# Use the official Rust image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /kprs/src/backend/

# Copy your project files into the container
COPY . .

# Build and run your Rust program
CMD ["cargo", "run", "--release"]
