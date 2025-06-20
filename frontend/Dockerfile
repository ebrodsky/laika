# --- Stage 1: Build Stage ---
# Use the official Node.js image as a builder
FROM node:22-alpine as builder

WORKDIR /app

# Copy dependency manifests
COPY package.json ./
COPY package-lock.json ./

# Install dependencies
RUN npm install

# Copy the rest of the application source code
COPY . .

# Build the React app for production
# Pass in the API_URL as a build-time argument
ARG API_URL
ENV API_URL=$API_URL
RUN npm run build

# --- Stage 2: Final Image ---
# Use a lightweight Nginx image to serve the static files
FROM nginx:stable-alpine

# Copy the built static files from the builder stage
COPY --from=builder /app/build /usr/share/nginx/html

# Copy the custom Nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# The default Nginx command will start the server
CMD ["nginx", "-g", "daemon off;"]

