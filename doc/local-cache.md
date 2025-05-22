To set up a **local binary cache** for Nix (similar in function to Cachix, but self-hosted on your network), you can use the built-in `nix-serve` service. This allows you to share build artifacts between machines, speeding up rebuilds and reducing redundant compilation. Below are detailed instructions for a typical NixOS setup:

---

## 1. Generate Signing Keys

Nix binary caches require signing keys to ensure integrity.

```bash
nix-store --generate-binary-cache-key /var/cache-priv-key.pem /var/cache-pub-key.pem
```
- `/var/cache-priv-key.pem`: Private key (keep secure, used for signing)
- `/var/cache-pub-key.pem`: Public key (shared with clients)

---

## 2. Enable and Configure `nix-serve` on the Cache Server

Add the following to your server's `/etc/nixos/configuration.nix`:

```nix
services.nix-serve = {
  enable = true;
  secretKeyFile = "/var/cache-priv-key.pem";
};
```

- This will serve the cache over HTTP (default port 5000).
- Optionally, you can use Nginx as a reverse proxy for HTTPS support[3].

Then, rebuild your system:

```bash
sudo nixos-rebuild switch
```

---

## 3. Share the Public Key

Copy `/var/cache-pub-key.pem` to your client machines. You’ll need the contents for configuration.

---

## 4. Configure Clients to Use the Local Cache

On each client, add the server as a substituter and trust its public key. Edit `/etc/nixos/configuration.nix` or `/etc/nix/nix.conf`:

```nix
nix.settings.substituters = [
  "http://:5000"
  "https://cache.nixos.org/"
];
nix.settings.trusted-public-keys = [
  "-1:"
  "cache.nixos.org-1:..."
];
```
- Replace `` and `` with your actual values.
- The public key string is the base64-encoded value from `/var/cache-pub-key.pem`.

Rebuild the client system:

```bash
sudo nixos-rebuild switch
```

---

## 5. Use the Cache

Now, when you build on one machine, the results are uploaded to the cache server. Other machines will fetch build results from the cache if available, greatly speeding up builds[2][3][1].

---

## Optional: Use Nginx as a Proxy (for HTTPS)

If you want to serve the cache over HTTPS (recommended for larger networks), set up Nginx as a reverse proxy in front of `nix-serve`[3]:

```nix
services.nginx = {
  enable = true;
  recommendedProxySettings = true;
  virtualHosts."cache.local" = {
    locations."/" = {
      proxyPass = "http://localhost:5000";
    };
  };
};
networking.firewall.allowedTCPPorts = [ 80 ];
```

---

## Summary Table

| Step                   | Server Action                               | Client Action                 |
| ---------------------- | ------------------------------------------- | ----------------------------- |
| Generate keys          | `nix-store --generate-binary-cache-key ...` | Copy public key               |
| Enable nix-serve       | Add to `configuration.nix`, rebuild         |                               |
| Configure substituters |                                             | Add server URL and public key |
| (Optional) Nginx proxy | Configure Nginx, open ports                 | Use HTTPS substituter         |

---

**Note:**  
If you want a more "Cachix-like" experience with push/pull and S3 support, consider self-hosted solutions like [Attic](9) or [Distributed Nix Cache Server with Cachix compatibility](8), but for most local setups, `nix-serve` is the simplest and most robust[2][3][1].

Citations:
[1] https://discourse.nixos.org/t/nix-store-cache-in-local-network/28972
[2] https://nixos.wiki/wiki/Binary_Cache
[3] https://nix.dev/tutorials/nixos/binary-cache-setup.html
[4] https://www.reddit.com/r/NixOS/comments/1i32171/using_nix_cache_with_cachix_or_nixserve/
[5] https://www.channable.com/tech/setting-up-a-private-nix-cache-for-fun-and-profit
[6] https://scrive.github.io/nix-workshop/06-infrastructure/01-caching-nix.html
[7] https://docs.cachix.org/getting-started
[8] https://discourse.nixos.org/t/distributed-nix-cache-server-with-cachix-compatibility/63837
[9] https://discourse.nixos.org/t/introducing-attic-a-self-hostable-nix-binary-cache-server/24343
[10] https://discourse.nixos.org/t/caching-nixos-files-for-offline-builds-and-bandwidth-reduction/48162
