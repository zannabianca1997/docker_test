FROM postgres:16.3-alpine3.19

USER postgres

COPY ./init.sql /docker-entrypoint-initdb.d/init.sql

ENV POSTGRES_USER=postgres
ENV POSTGRES_DB=messages