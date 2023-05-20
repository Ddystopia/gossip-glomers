```json
{
  "src": "c1",
  "dest": "n1",
  "body": {
    "type": "init",
    "msg_id": 1,
    "node_id": "n1",
    "node_ids": ["n1", "n2", "n3"]
  }
}
```

```
{ "src": "c1", "dest": "n1", "body": { "type": "init", "msg_id": 1, "node_id": "n1", "node_ids": ["n1", "n2", "n3"] } }
```

```json
{
  "src": "c1",
  "dest": "n1",
  "body": {
    "type": "topology",
    "topology": {
      "n1": ["n2"]
    }
  }
}
```

```
{ "src": "c1", "dest": "n1", "body": { "type": "topology", "topology": { "n1": ["n2"] } } }
```

```json
{
  "src": "c1",
  "dest": "n1",
  "body": {
    "type": "broadcast",
    "message": 1000
  }
}
```

```
{ "src": "c1", "dest": "n1", "body": { "type": "broadcast", "message": 1000 } }
```



```json
{
  "src": "n2",
  "dest": "n1",
  "body": {
    "type": "gossip_ok",
    "messages": [1000]
  }
}

```

```
{ "src": "n2", "dest": "n1", "body": { "type": "gossip_ok", "messages": [1000] } }
```

```json
{
  "src": "c1",
  "dest": "n1",
  "body": {
    "type": "read"
  }
}
```

```
{ "src": "c1", "dest": "n1", "body": { "type": "read" } }
```
