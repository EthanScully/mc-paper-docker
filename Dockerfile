FROM --platform=$BUILDPLATFORM rust AS build
ARG TARGETPLATFORM
ARG BUILDPLATFORM
WORKDIR /build/
COPY . /build/
RUN bash docker-build.sh $TARGETPLATFORM $BUILDPLATFORM
FROM debian:stable-slim
COPY --from=build /target/root/bin/mc-update /bin
WORKDIR /minecraft
ENTRYPOINT ["mc-update"]