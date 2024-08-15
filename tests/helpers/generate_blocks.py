#!/usr/bin/python
#
# Gets a new address and generates blocks to that address.

from requests.auth import HTTPBasicAuth
import requests

url = "http://127.0.0.1:1024"
auth = HTTPBasicAuth("admin", "admin")

method = "getnewaddress"
params = {}
payload = {
    "jsonrpc": "2.0",
    "method": method,
    "params": params,
    "id": 1,
}
response = requests.post(url, json=payload, auth=auth).json()
address = response["result"]
print("Address:", response)

method = "getblockcount"
params = {}
payload = {
    "jsonrpc": "2.0",
    "method": method,
    "params": params,
    "id": 1,
}
response = requests.post(url, json=payload, auth=auth).json()
print("Block count:", response)

method = "generatetoaddress"
params = {"nblocks": 2, "address": address}
payload = {
    "jsonrpc": "2.0",
    "method": method,
    "params": params,
    "id": 1,
}
response = requests.post(url, json=payload, auth=auth).json()
print("Generate to address:", response)

method = "getblockcount"
params = {}
payload = {
    "jsonrpc": "2.0",
    "method": method,
    "params": params,
    "id": 1,
}
response = requests.post(url, json=payload, auth=auth).json()
print("Block count:", response)
