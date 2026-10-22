# Deploying Fala

Fala runs on Fly as one small machine at **https://fala.veesh.me**. The SQLite database lives
on a Fly volume and Litestream backs it up continuously to a Tigris bucket. After the one-time
setup below, every merge to `master` deploys itself once CI passes.

## One-time setup

You need the [`fly` CLI](https://fly.io/docs/flyctl/install/), signed in (`fly auth login`).
Run these from the repo root.

1. **Create the app and its volume.** The volume holds the database. Pick the region nearest
   you and change `primary_region` in `fly.toml` to match (`fly platform regions` lists them).

   ```sh
   fly apps create fala-veesh
   fly volumes create fala_data --size 1 --region fra --app fala-veesh
   ```

2. **Create the backup bucket.** This sets `BUCKET_NAME` and the `AWS_*` credentials as
   secrets on the app, which Litestream picks up.

   ```sh
   fly storage create --app fala-veesh
   ```

3. **Set the secrets.** `SESSION_KEY` encrypts the sign-in cookie. `SETUP_TOKEN` lets you
   create the first passkey; write it down for step 6.

   ```sh
   fly secrets set --app fala-veesh --stage \
     SESSION_KEY="$(openssl rand -hex 64)" \
     SETUP_TOKEN="$(openssl rand -hex 16 | tee /dev/stderr)"
   ```

4. **Deploy once by hand.**

   ```sh
   fly deploy --ha=false
   ```

5. **Point the domain at it.** Ask Fly for a certificate, then add the DNS record it shows at
   wherever veesh.me's DNS lives: a `CNAME` from `fala` to `fala-veesh.fly.dev`.

   ```sh
   fly certs add fala.veesh.me --app fala-veesh
   fly certs show fala.veesh.me --app fala-veesh   # repeat until it says the certificate is issued
   ```

6. **Create your passkey.** On your phone, open
   `https://fala.veesh.me/setup?token=<SETUP_TOKEN>` and tap "Create passkey". After that the
   token no longer works; add more devices from the Account page while signed in.

7. **Turn on deploy-on-merge.** Create a deploy token and add it to the GitHub repo as the
   `FLY_API_TOKEN` secret (Settings → Secrets and variables → Actions).

   ```sh
   fly tokens create deploy --app fala-veesh
   ```

## Day to day

- Merging to `master` runs CI, then the Deploy workflow. Without `FLY_API_TOKEN` it skips.
- The machine stops when idle and starts on the next request (a second or two).
- Logs: `fly logs --app fala-veesh`.

## If something goes wrong

- **Lost the volume?** Create a new one (step 1). On start, Litestream restores the database
  from the bucket before the server opens it.
- **Locked out (lost every passkey)?** Delete the passkeys and set a new setup token:

  ```sh
  fly ssh console --app fala-veesh -C "sqlite3 /data/fala.db 'DELETE FROM passkeys'"
  fly secrets set --app fala-veesh SETUP_TOKEN="$(openssl rand -hex 16 | tee /dev/stderr)"
  ```
