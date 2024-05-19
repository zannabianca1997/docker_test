# Frontend
This is a basic frontend written in `Next.js`

## Running
Before running, you must generate the bindings with `npm run bindgen`.

You can then directly run a dev server with `APIURL='...' npm run dev`, substituting the ellipsis with the right URL.
You can also build and run the production version:
```bash
npm run build
APIURL='...' node .next/standalone/server.js
```