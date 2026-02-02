# JaLM Host ABI (V0)

This document defines the host interface for JaLM when targeting WASM.

## Goals
- Use WASI for generic platform facilities (clocks, randomness).
- Use custom imports for web host features: HTTP, sockets, logging, cancellation.
- Keep the ABI stable and easy to bind from multiple hosts.

## WASI vs Custom Imports
**WASI** is used for:
- Monotonic + wall clock (`wasi:clocks/*`) for time measurements.
- Randomness (`wasi:random/random`) for IDs and jitter.

**Custom imports** are used for:
- HTTP requests/responses.
- TCP sockets (stream abstraction).
- Logging.
- Cancellation and timeouts.

Reasoning: Web service I/O is highly host-specific and needs richer semantics
than WASI provides today. The custom interface is narrow, explicit, and stable.

## Canonical Types
- `string` and `bytes` are passed as `(ptr, len)` in the raw ABI.
- `bytes` is `list<u8>` in the logical WIT layer.
- Handles are `u32`.

## Custom Import ABI (Raw)
All custom imports live in the `jalm.host` module at link time.

### Logging
```
log(level: u8, msg_ptr: u32, msg_len: u32) -> void
```

### Time / Cancellation
```
sleep_ms(ms: u64) -> u32   // returns cancellation token
cancel(token: u32) -> bool
```

### HTTP
```
http_request(
  method_ptr: u32, method_len: u32,
  url_ptr: u32, url_len: u32,
  headers_ptr: u32, headers_len: u32, // list of (k,v) pairs
  body_ptr: u32, body_len: u32,
  timeout_ms: u64,
) -> u32 // handle to response

http_response_status(handle: u32) -> u16
http_response_headers(handle: u32, out_ptr: u32, out_len: u32) -> u32
http_response_body(handle: u32, out_ptr: u32, out_len: u32) -> u32
http_response_free(handle: u32)
```

### Net (TCP)
```
net_connect(host_ptr: u32, host_len: u32, port: u16) -> u32
net_close(handle: u32)
net_read(handle: u32, out_ptr: u32, out_len: u32) -> u32
net_write(handle: u32, data_ptr: u32, data_len: u32) -> u32
```

## WIT Layer
A logical WIT description lives at `abi/host_abi_v0.wit`. This is the
recommended interface for generating bindings and hosts.

## Edge Cases / Notes
- `http_response_*` uses a handle so responses can be streamed.
- `headers_ptr` is encoded as a flat list of `(key, value)` pairs.
- Cancellation tokens are opaque and may be host-specific.

## Performance Notes
The interface minimizes allocations by allowing the host to write into
caller-provided buffers (out_ptr/out_len). The host may return a required size
when the buffer is too small.
