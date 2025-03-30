import json
import random
import os
import sys
from datetime import datetime, timedelta

def create_json_log(count, filename):
    print(f"Creating {count} JSON log entries in {filename}")
    services = ["api", "auth", "payment", "database", "frontend", "cache", "search", "notification"]
    levels = ["INFO", "WARN", "ERROR", "DEBUG"]
    messages = [
        "Request processed successfully",
        "Database query completed",
        "Authentication successful",
        "Failed login attempt",
        "Payment processed",
        "Connection timeout",
        "Cache miss",
        "User profile updated",
        "NullPointerException in WebController",
        "Rate limit exceeded",
        "Slow database query detected",
        "Authentication token expired"
    ]
    status_codes = [200, 201, 204, 400, 401, 403, 404, 500, 503]

    # Generate random but deterministic log entries
    random.seed(42)  # For reproducible results
    base_time = datetime.now() - timedelta(days=1)

    with open(filename, 'w') as f:
        for i in range(count):
            timestamp = (base_time + timedelta(seconds=i)).isoformat()
            service = random.choice(services)

            # Make ERROR level appear in about 15% of logs
            level = "ERROR" if random.random() < 0.15 else random.choice(levels)

            # Make status 500 appear in about 5% of logs
            status = 500 if random.random() < 0.05 else random.choice(status_codes)

            response_time = random.randint(10, 2000)  # 10ms to 2s

            # Create nested fields for some services
            if service == "api":
                request = {
                    "method": random.choice(["GET", "POST", "PUT", "DELETE"]),
                    "path": f"/api/v1/{random.choice(['users', 'orders', 'products', 'carts'])}",
                    "headers": {
                        "content-type": random.choice(["application/json", "text/html", "application/xml"]),
                        "user-agent": "Mozilla/5.0"
                    }
                }
            else:
                request = None

            # Add user info for auth service
            if service == "auth":
                user = {
                    "id": f"user_{random.randint(1000, 9999)}",
                    "role": random.choice(["admin", "user", "guest"])
                }
            else:
                user = None

            # Basic log structure
            log_entry = {
                "timestamp": timestamp,
                "service": service,
                "level": level,
                "message": random.choice(messages),
                "request_id": f"req-{random.randint(10000, 99999)}",
                "status": status,
                "response_time": response_time
            }

            # Add nested fields if present
            if request:
                log_entry["request"] = request
            if user:
                log_entry["user"] = user

            # Add error details for ERROR level
            if level == "ERROR":
                log_entry["error"] = {
                    "type": random.choice(["NullPointerException", "ConnectionTimeout", "AuthenticationFailure", "DatabaseError"]),
                    "code": random.randint(1000, 9999)
                }

            # Write the JSON object
            f.write(json.dumps(log_entry) + "\n")

bench_dir = sys.argv[1]
with_large = (len(sys.argv) > 2 and sys.argv[2].lower() == 'true')

# Create datasets of different sizes
create_json_log(10000, os.path.join(bench_dir, "bench_json_10k.json"))
create_json_log(100000, os.path.join(bench_dir, "bench_json_100k.json"))
create_json_log(1000000, os.path.join(bench_dir, "bench_json_1m.json"))

if with_large:
    create_json_log(10000000, os.path.join(bench_dir, "bench_json_10m.json"))