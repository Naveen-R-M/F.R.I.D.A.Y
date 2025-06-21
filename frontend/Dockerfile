FROM nginx:alpine

# Copy static files
COPY . /usr/share/nginx/html

# Remove the index.html file if it comes from the static files
RUN rm -f /usr/share/nginx/html/nginx.conf

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
