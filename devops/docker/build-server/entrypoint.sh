#!/bin/bash

# Start PostgreSQL service
service postgresql start

# Start Mosquitto service
service mosquitto start

# Start Supervisor to manage all services
exec supervisord -c /etc/supervisor/supervisord.conf
