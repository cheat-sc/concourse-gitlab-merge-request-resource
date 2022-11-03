FROM archlinux AS builder
RUN --mount=type=cache,target=/var/cache/pacman pacman -Syu --noconfirm rust pkgconf
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/src/target \
	--mount=type=cache,target=/usr/local/cargo/registry \
	cargo build --release && cargo install --path .

FROM archlinux
COPY --from=builder /root/.cargo/bin/check /opt/resource/
COPY --from=builder /root/.cargo/bin/in /opt/resource/
COPY --from=builder /root/.cargo/bin/out /opt/resource/
