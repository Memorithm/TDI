"""Bounded standard-library client for the scirust-hub HTTP/v1 surface.

Hub remains the scheduler and artifact store. This client never retries a
mutation, forwards credentials to a redirect, or treats admission as a
scientific authorization. HTTP is limited to explicitly selected loopback IPs.
"""
from __future__ import annotations

import hashlib
import http.client
import ipaddress
import math
import re
import ssl
import urllib.error
import urllib.parse
import urllib.request
import uuid

import tdi_artifact_contract as artifacts
import tdi_experiment_supervisor as durable
import tdi_hub_edge_contract as edge

MAX_TRANSFER_BYTES = 16 * 1024 * 1024


class HubClientError(durable.ContractError):
    """A peer, response or client configuration violated the transport contract."""


class HubTransportUnknown(HubClientError):
    """No reliable response was received; a submitted operation may have happened."""


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def entity_id(value):
    """Validate one canonical UUID before interpolation into a resource path."""
    try:
        if not isinstance(value, str) or str(uuid.UUID(value)) != value:
            raise ValueError()
    except (ValueError, AttributeError):
        raise HubClientError("invalid canonical Hub entity id") from None
    return value


class HubClient:
    """Connect to one explicitly selected Hub, without automatic mutation retries.

    Example: ``HubClient('http://127.0.0.1:8477', allow_loopback_http=True)``.
    HTTPS validates certificates against the platform trust store. A remote
    HTTPS deployment also requires a token. Never place tokens in the URL.
    """

    def __init__(self, endpoint, *, token=None, allow_loopback_http=False,
                 timeout=30, max_bytes=MAX_TRANSFER_BYTES):
        try:
            parsed = urllib.parse.urlsplit(endpoint)
            port = parsed.port
        except ValueError:
            raise HubClientError("invalid Hub endpoint") from None
        if (parsed.scheme not in ("https", "http") or not parsed.hostname
                or parsed.username or parsed.password or parsed.query or parsed.fragment
                or parsed.path not in ("", "/") or any(c.isspace() for c in endpoint)):
            raise HubClientError("Hub endpoint must be an origin without credentials or path")
        try:
            loopback = ipaddress.ip_address(parsed.hostname).is_loopback
        except ValueError:
            loopback = False
        if parsed.scheme == "http" and not (loopback and allow_loopback_http is True):
            raise HubClientError("plaintext Hub transport requires explicit loopback selection")
        if not loopback and not token:
            raise HubClientError("remote Hub requires an authentication token")
        if token is not None and (not isinstance(token, str) or not 1 <= len(token) <= 4096
                                  or any(not 33 <= ord(c) <= 126 for c in token)):
            raise HubClientError("invalid Hub token format")
        if (type(timeout) not in (float, int) or not math.isfinite(timeout)
                or not 0 < timeout <= 300):
            raise HubClientError("transport timeout must be in (0, 300] seconds")
        if type(max_bytes) is not int or not 0 < max_bytes <= MAX_TRANSFER_BYTES:
            raise HubClientError("transfer budget must be between 1 byte and 16 MiB")
        self.endpoint = urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, "", "", ""))
        self.timeout, self.max_bytes, self._token = timeout, max_bytes, token
        self._opener = urllib.request.build_opener(
            urllib.request.ProxyHandler({}), _NoRedirect(),
            urllib.request.HTTPSHandler(context=ssl.create_default_context()),
        )

    def request(self, method, path, *, value=None, payload=None, headers=None, binary=False):
        """Perform one bounded request; malformed/ambiguous mutations are never retried.

        JSON errors contain only the HTTP status, not the remote body, URL,
        credentials or worker diagnostics. ``binary=True`` returns exact bytes.
        """
        if method not in ("GET", "POST") or not path.startswith("/api/v1/"):
            raise HubClientError("unsupported Hub operation")
        if "#" in path or "\\" in path or ".." in path:
            raise HubClientError("invalid Hub resource path")
        if value is not None and payload is not None:
            raise HubClientError("only one request body is allowed")
        if value is not None:
            payload = durable.canonical(value).encode("utf-8")
        if payload is not None and (not isinstance(payload, bytes) or len(payload) > self.max_bytes):
            raise HubClientError("request exceeds transfer budget")
        request_headers = {"Accept": "application/octet-stream" if binary else "application/json"}
        if value is not None:
            request_headers["Content-Type"] = "application/json"
        if headers:
            if set(headers) - {"Content-Type", "x-scirust-artifact-name"}:
                raise HubClientError("unsupported request headers")
            request_headers.update(headers)
        if self._token:
            request_headers["Authorization"] = "Bearer " + self._token
        request = urllib.request.Request(self.endpoint + path, data=payload,
                                         method=method, headers=request_headers)
        try:
            with self._opener.open(request, timeout=self.timeout) as response:
                if response.geturl() != self.endpoint + path:
                    raise HubClientError("Hub redirect is forbidden")
                length = response.headers.get("Content-Length")
                if length is not None and (not length.isdecimal() or int(length) > self.max_bytes):
                    raise HubClientError("response exceeds transfer budget")
                raw = response.read(self.max_bytes + 1)
                if len(raw) > self.max_bytes:
                    raise HubClientError("response exceeds transfer budget")
                if length is not None and len(raw) != int(length):
                    raise HubClientError("truncated Hub response")
                if binary:
                    return raw
                value = durable.strict_json(raw, max_bytes=self.max_bytes)
                if not isinstance(value, dict):
                    raise HubClientError("Hub response must be an object")
                return value
        except urllib.error.HTTPError as error:
            error.close()
            raise HubClientError(f"Hub HTTP status {error.code}; mutation was not retried") from None
        except (OSError, ValueError, http.client.HTTPException) as error:
            if method != "GET":
                raise HubTransportUnknown("Hub mutation response unavailable; reconcile before continuing") from None
            raise HubClientError("Hub response unavailable or invalid") from None

    def upload(self, descriptor, payload):
        """Upload an explicitly selected permitted artifact and verify Hub's digest bridge."""
        descriptor = artifacts.verify_artifact_bytes(descriptor, payload)
        if descriptor["access_class"] == "restricted-reference":
            raise HubClientError("restricted payload upload is not supported by this client")
        if not re.fullmatch(r"[\x21-\x7e]{1,128}", descriptor["name"]):
            raise HubClientError("upload name must be a bounded ASCII header label")
        meta = self.request("POST", "/api/v1/artifacts", payload=bytes(payload), headers={
            "x-scirust-artifact-name": descriptor["name"],
            "Content-Type": descriptor["media_type"],
        })
        identity = entity_id(meta["id"])
        portable = self.request("GET", f"/api/v1/artifacts/{identity}/portable-digest")
        if portable.get("id") != identity or meta.get("digest") != portable.get("hub_digest"):
            raise HubClientError("uploaded artifact identity mismatch")
        return edge.bind_portable_artifact(descriptor, portable)

    def download(self, artifact_id, descriptor=None):
        """Read and verify one exact bounded payload; reject restricted descriptors first."""
        identity = entity_id(artifact_id)
        if descriptor is not None:
            descriptor = artifacts.canonical_artifact(descriptor)
            if descriptor["access_class"] == "restricted-reference":
                raise HubClientError("restricted payload download is not supported")
            if descriptor["size_bytes"] > self.max_bytes:
                raise HubClientError("artifact exceeds transfer budget")
        portable = self.request("GET", f"/api/v1/artifacts/{identity}/portable-digest")
        if portable.get("id") != identity or type(portable.get("size")) is not int:
            raise HubClientError("download identity mismatch")
        if not 0 <= portable["size"] <= self.max_bytes:
            raise HubClientError("artifact exceeds transfer budget")
        raw = self.request("GET", f"/api/v1/artifacts/{identity}/content", binary=True)
        if len(raw) != portable["size"] or hashlib.sha256(raw).hexdigest() != portable.get("raw_sha256"):
            raise HubClientError("download content digest mismatch")
        if descriptor is not None:
            artifacts.verify_artifact_bytes(descriptor, raw)
            edge.bind_portable_artifact(descriptor, portable)
        return raw, portable
