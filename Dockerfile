# Runtime image built from the release binaries: no compilation here, the binary
# for the platform is copied from dist/<arch>/ prepared by the release workflow.
FROM debian:bookworm-slim
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl procps git && rm -rf /var/lib/apt/lists/*
COPY dist/${TARGETARCH}/souvenance /usr/local/bin/souvenance
ENV SOUVENANCE_ROOT=/notes
VOLUME ["/notes", "/root/.souvenance"]
ENTRYPOINT ["souvenance"]
CMD ["help"]
