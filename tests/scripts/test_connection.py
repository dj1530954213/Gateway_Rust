#!/usr/bin/env python3
import psycopg2
import requests
import sys

def test_postgresql():
    try:
        conn = psycopg2.connect(
            host="localhost",
            port=5432,
            database="iot",
            user="postgres",
            password="postgres"
        )
        cursor = conn.cursor()
        cursor.execute("SELECT 1;")
        result = cursor.fetchone()
        cursor.close()
        conn.close()
        print("OK PostgreSQL connection successful")
        return True
    except Exception as e:
        print(f"FAIL PostgreSQL connection failed: {e}")
        return False

def test_influxdb():
    try:
        response = requests.get("http://localhost:8086/health", timeout=5)
        if response.status_code == 200:
            print("OK InfluxDB connection successful")
            return True
        else:
            print(f"FAIL InfluxDB connection failed: HTTP {response.status_code}")
            return False
    except Exception as e:
        print(f"FAIL InfluxDB connection failed: {e}")
        return False

if __name__ == "__main__":
    print("Testing database connections...")
    pg_ok = test_postgresql()
    influx_ok = test_influxdb()
    
    if pg_ok and influx_ok:
        print("All connections successful!")
        sys.exit(0)
    else:
        print("Some connections failed!")
        sys.exit(1)