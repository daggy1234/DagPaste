FROM  rustlang/rust:nightly as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/paste
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10
WORKDIR /usr/local/bin
COPY --from=build /usr/local/cargo/bin/paste .
#COPY --from=build /usr/src/paste/Rocket.toml ./Rocket.toml
#COPY --from=build /usr/src/paste/static ./static
#COPY --from=build /usr/src/paste/private ./private

EXPOSE 8000
CMD ["paste"]