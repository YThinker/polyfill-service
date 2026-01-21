<h1 align="center">
    <a href="https://cdnjs.com"><img src="https://raw.githubusercontent.com/cdnjs/brand/master/logo/standard/dark-512.png" width="175px" alt="< cdnjs >"></a>
</h1>

<h3 align="center">The #1 free and open source CDN built to make life easier for developers.</h3>

---

## Polyfill.io now available on cdnjs

Start using the service: <https://cdnjs.cloudflare.com/polyfill>

See the announcements from Cloudflare: <https://blog.cloudflare.com/polyfill-io-now-available-on-cdnjs-reduce-your-supply-chain-risk> + <https://blog.cloudflare.com/automatically-replacing-polyfill-io-links-with-cloudflares-mirror-for-a-safer-internet>.

## Running the Service

### Environment Variables

- `POLYFILL_BASE`: Path to polyfill libraries directory (default: `polyfill-libraries`)
- `PORT`: Server port (default: `8787`)
- `CACHE_DIR`: Optional cache directory for generated polyfill bundles. If set, generated bundles will be cached on disk to avoid regeneration for identical requests.

### Example

```bash
# Without cache
POLYFILL_BASE=./polyfill-libraries PORT=8787 cargo run -p service

# With cache
POLYFILL_BASE=./polyfill-libraries PORT=8787 CACHE_DIR=./cache cargo run -p service
```

### Cache Feature

When `CACHE_DIR` is set, the service will:
- Generate a SHA256 hash key based on all request parameters (version, features, UA string, minify flag, etc.)
- Check if a cached file exists before generating a new bundle
- Return cached content immediately if found
- Write generated bundles to cache for future requests

Cache files are stored as `{hash}.js` in the cache directory. The cache key includes all parameters that affect the output, ensuring correctness.

**Docker Deployment**:
- Cache directory is automatically cleared on each container start
- Default cache directory is `/app/cache-dir` (can be overridden with `CACHE_DIR` env var)