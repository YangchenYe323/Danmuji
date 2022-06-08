# build & package frontend code
FROM node:16 as frontend-builder
WORKDIR ./frontend
ADD ./frontend .
RUN npm install
RUN npm run build

# build rust code
FROM rust:1.59 as server-builder
# create an empty rust project(with no source code)
# and compile dependencies in a separate layer
# this can reduce compilation time when we make changes to
# the source code but not the dependencies
RUN USER=root cargo new --bin Danmuji
WORKDIR ./Danmuji
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
# add source code and re-build, this time
# with source code
ADD . ./
RUN rm ./target/release/deps/Danmuji*
RUN cargo build --release

# final container image that serves
# the binaries
FROM debian:buster-slim
# configurable application root directory
ARG APP_ROOT=/usr/src/app
# application port
# alert: this should match the port exposed
# in ./src/main.rs
ARG APP_PORT=9000

# install openssl packages
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE $APP_PORT

# create a non-root user
# to run our binary
ENV APP_USER=appuser
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP_ROOT}

# copy rust binary
COPY --from=server-builder /Danmuji/target/release/Danmuji ${APP_ROOT}/Danmuji
# copy frontend package
COPY --from=frontend-builder /frontend/dist ${APP_ROOT}/frontend/dist

# make APP_USER own the application root directory
RUN chown -R $APP_USER:$APP_USER ${APP_ROOT}

# execution environment
USER $APP_USER
WORKDIR ${APP_ROOT}

CMD ["./Danmuji"]

