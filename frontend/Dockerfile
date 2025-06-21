FROM nginx:alpine

# Copy static files
COPY . /usr/share/nginx/html

# Remove config file from static files to avoid confusion
RUN rm -f /usr/share/nginx/html/default.conf
RUN rm -f /usr/share/nginx/html/nginx.conf

# Copy config
COPY default.conf /etc/nginx/conf.d/default.conf

# Expose port
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
