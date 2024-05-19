# syntax=docker/dockerfile:1

#  -- Common stage to build rust stuff 

ARG RUST_VERSION=1.78.0

FROM rust:${RUST_VERSION}-alpine AS rust-build-base
WORKDIR /app

# Install host build dependencies.
RUN apk add --no-cache clang lld musl-dev git

# ----  BACKEND ---

################################################################################
# Create a stage for building the application.

FROM rust-build-base AS backend-build

# Build the application.
#
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies, a cache mount to /usr/local/cargo/git/db
# for git repository dependencies, and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
#
# Rebuild a skeleton of the workspace to have access to both common_types
# and test_chat_server
#
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. 
#
# Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=test_chat_server/src,target=test_chat_server/src \
    --mount=type=bind,source=test_chat_server/Cargo.toml,target=test_chat_server/Cargo.toml \
    --mount=type=bind,source=common_types/src,target=common_types/src \
    --mount=type=bind,source=common_types/Cargo.toml,target=common_types/Cargo.toml \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --package test_chat_server --bin test_chat_server && \
    cp ./target/release/test_chat_server /bin/server

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.

FROM alpine:3.18 AS backend-final


# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=backend-build /bin/server /bin/

# Expose the port that the application listens on.
EXPOSE 3000

ENTRYPOINT [ "/bin/server", "--addr", "0.0.0.0:3000" ]

# ----  FRONTEND ---

FROM node:18-alpine AS frontend-base

# Install dependencies only when needed
FROM frontend-base AS frontend-deps

# Check https://github.com/nodejs/docker-node/tree/b4117f9333da4138b03a546ec926ef50a31506c3#nodealpine to understand why libc6-compat might be needed.
RUN apk add --no-cache libc6-compat

WORKDIR /app

# Install dependencies
RUN --mount=type=bind,source=test_chat_client/package.json,target=package.json \
    --mount=type=bind,source=test_chat_client/package-lock.json,target=package-lock.json \
    npm ci

# Bindgen: compile the crate, then generate the bindgens
FROM rust-build-base AS frontend-bindgen

# Bind the whole project again and build the bindings.
# The whole workspace is needed so Cargo.lock can be reused, and consistency between the bindings and the backend is ensured
#
# Once built, run the bindgen and store the bindings.d.ts generated
RUN --mount=type=bind,source=test_chat_server/src,target=test_chat_server/src \
    --mount=type=bind,source=test_chat_server/Cargo.toml,target=test_chat_server/Cargo.toml \
    --mount=type=bind,source=common_types/src,target=common_types/src \
    --mount=type=bind,source=common_types/Cargo.toml,target=common_types/Cargo.toml \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release --package common_types --features bindgen --bin bindgen && \
    ./target/release/bindgen > /opt/bindings.json

# Rebuild the source code only when needed
FROM frontend-base AS frontend-build

WORKDIR /app
COPY --from=frontend-deps /app/node_modules ./node_modules
COPY test_chat_client .
# recover generated bindings
COPY --from=frontend-bindgen /opt/bindings.json /opt/bindings.json

# generate typescript bindings
RUN ./node_modules/.bin/json2ts /opt/bindings.json ./app/bindings.d.ts

# Next.js collects completely anonymous telemetry data about general usage.
# Learn more here: https://nextjs.org/telemetry
# Uncomment the following line in case you want to disable telemetry during the build.
# ENV NEXT_TELEMETRY_DISABLED 1

RUN npm run build

# Production image, copy all the files and run next
FROM frontend-base AS frontend-final
WORKDIR /app

ENV NODE_ENV production
# Uncomment the following line in case you want to disable telemetry during runtime.
ENV NEXT_TELEMETRY_DISABLED 1

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=frontend-build /app/public ./public

# Set the correct permission for prerender cache
RUN mkdir .next
RUN chown nextjs:nodejs .next

# Automatically leverage output traces to reduce image size
# https://nextjs.org/docs/advanced-features/output-file-tracing
COPY --from=frontend-build --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=frontend-build --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000

ENV PORT 3000

# server.js is created by next build from the standalone output
# https://nextjs.org/docs/pages/api-reference/next-config-js/output
CMD HOSTNAME="0.0.0.0" node server.js
