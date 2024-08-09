#!/usr/bin/python
#
# JSON RPC call example.

from requests.auth import HTTPBasicAuth
import requests

url = "http://127.0.0.1:1024"
# url = "http://127.0.0.1:18443"

auth = HTTPBasicAuth("admin", "admin")

method = "getrawtransaction"
params = {"txid": "8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87"}

payload = {
    "jsonrpc": "2.0",
    "method": method,
    "params": params,
    "id": 1,
}

response = requests.post(url, json=payload, auth=auth)

print(response.json())
