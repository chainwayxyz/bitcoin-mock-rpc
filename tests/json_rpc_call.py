#!/usr/bin/python
#
# JSON RPC call example.

import requests

url = "http://127.0.0.1:1024"
method = "sendrawtransaction"
params = {"tx": "dummy"} 

payload = {
    "jsonrpc": "2.0",
    "method": method,
    "params": params,
    "id": 1,
}

response = requests.post(url, json=payload)
print(response.json())
