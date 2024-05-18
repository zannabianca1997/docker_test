# Frontend
This is a basic frontend written in `Next.js`

## Running
You can directly run a dev server with `APIURL='...' npm run dev`, substituting the ellipsis with the right URL.

You can also build and run:
```bash
npm run build
APIURL='...' node .next/standalone/server.js
```

### Running inside Docker
The project is ready to be containerized. Just run
```bash
docker build . -t frontend
docker run -p3000:3000 -eAPI_URL='...' -it frontend
```
substituting the running server URL to the ellipsis
You can customize the port:
```bash
docker run -p${PORT}:3000 -eAPI_URL='...' -it frontend
```
