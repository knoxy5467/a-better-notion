# Start from the official PostgreSQL image
FROM postgres

# Set environment variables
ENV POSTGRES_USER=abn
ENV POSTGRES_PASSWORD=abn
ENV POSTGRES_DB=abn

# Copy the SQL script into the Docker image
COPY createTable.sql /docker-entrypoint-initdb.d/