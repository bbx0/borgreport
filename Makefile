#!/usr/bin/env -S make -f
# SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
# SPDX-License-Identifier: GPL-3.0-or-later

# Makefile to manage release artifacts for borgreport
#
# Usage:
#	Build and Install:
#		make prepare
#		make build
#		make PREFIX=/usr install
#
#	Run the linter:
#		make lint
#
#	Run all tests:
#		make test
#
#	Make all release packages in target/dist/$(version):
#		make assets
#		make dist
#
#	> Make source tarball:
#		make assets
#		make crate
#
#	> Make static binaries:
#		make static
#
#	> Make static debian packages:
#		make assets
#		make debian
#
##

.SUFFIXES:
SHELL		= /bin/sh

# External Binaries (except cargo built-in commands and GNU core utilities)
ASCIIDOCTOR		:= asciidoctor
CARGO			:= cargo
CARGO-ABOUT		:= cargo about
CARGO-CLIPPY	:= cargo clippy
CARGO-DEB 		:= cargo deb
CARGO-DENY		:= cargo deny
CARGO-MSRV		:= cargo msrv
GREP 			:= grep
MINISIGN		:= minisign
REUSE 			:= reuse
RUSTC			:= rustc
SED				:= sed
TAR				:= tar
UPX				:= upx

# Common prefix for installation directories
PREFIX		:= /usr/local
prefix		:= ${PREFIX}
exec_prefix	:= $(prefix)
bindir		:= $(exec_prefix)/bin
datarootdir	:= $(prefix)/share
libdir		:= $(prefix)/lib
mandir		:= $(datarootdir)/man
man1dir		:= $(mandir)/man1

# Shell completion directories
bash_comp_dir	:= $(datarootdir)/bash-completion/completions
elv_comp_dir	:= $(datarootdir)/elvish/lib
fish_comp_dir	:= $(datarootdir)/fish/vendor_completions.d
zsh_comp_dir	:= $(datarootdir)/zsh/site-functions

# Systemd directories
sd_system_service_dir	:= $(libdir)/systemd/system
sd_user_service_dir		:= $(libdir)/systemd/user

# Generated and static assets
shell_completions := _borgreport borgreport.bash borgreport.elv borgreport.fish
generated_assets := $(addprefix assets/man/,borgreport.1) $(addprefix assets/shell_completions/,$(shell_completions))
static_assets := $(addprefix assets/systemd/,borgreport.service borgreport.timer) LICENSE LICENSE-THIRD-PARTY.md README.md CHANGELOG.md

# cargo get package.version
version ?= $(shell $(GREP) --perl-regexp --only-matching --max-count 1 -e '(?<=(^version = "))(.*)(?=("$$))' Cargo.toml)
locked_src := Cargo.toml Cargo.lock build.rs src/**
native_target_triple := $(shell $(RUSTC) -vV | sed -n 's/host: //p')

# Generate a native release build
.PHONY: build
build: prepare target/release/borgreport ;
target/release/borgreport: $(locked_src)
	$(CARGO) build --frozen --release

# Pull the native sources only
.PHONY: prepare
prepare:
	@$(CARGO) fetch --locked --target $(native_target_triple)

# Install the native releases
.PHONY: install
install: target/release/borgreport $(generated_assets) ${static_assets}
	install -Dm755 -t $(DESTDIR)$(bindir) target/release/borgreport
	install -Dm644 -t $(DESTDIR)$(man1dir) assets/man/borgreport.1
	install -Dm644 -t $(DESTDIR)$(bash_comp_dir) assets/shell_completions/borgreport.bash
	install -Dm644 -t $(DESTDIR)$(elv_comp_dir) assets/shell_completions/borgreport.elv
	install -Dm644 -t $(DESTDIR)$(fish_comp_dir) assets/shell_completions/borgreport.fish
	install -Dm644 -t $(DESTDIR)$(zsh_comp_dir) assets/shell_completions/_borgreport
	install -Dm644 -t ${DESTDIR}$(sd_system_service_dir) assets/systemd/borgreport.service
	install -Dm644 -t ${DESTDIR}$(sd_system_service_dir) assets/systemd/borgreport.timer
	install -Dm644 -t ${DESTDIR}$(sd_user_service_dir) assets/systemd/borgreport.service
	install -Dm644 -t ${DESTDIR}$(sd_user_service_dir) assets/systemd/borgreport.timer

.PHONY: uninstall
uninstall:
	-rm $(DESTDIR)$(bindir)/borgreport
	-rm $(DESTDIR)$(man1dir)/borgreport.1
	-rm $(DESTDIR)$(bash_comp_dir)/borgreport.bash
	-rm $(DESTDIR)$(elv_comp_dir)/borgreport.elv
	-rm $(DESTDIR)$(fish_comp_dir)/borgreport.fish
	-rm $(DESTDIR)$(zsh_comp_dir)/_borgreport
	-rm ${DESTDIR}$(sd_system_service_dir)/borgreport.service
	-rm ${DESTDIR}$(sd_system_service_dir)/borgreport.timer
	-rm ${DESTDIR}$(sd_user_service_dir)/borgreport.service
	-rm ${DESTDIR}$(sd_user_service_dir)/borgreport.timer

.PHONY: clean
clean:
	$(CARGO) clean

#### Development and Release Helper ####
# Note for assets:
# The non-PHONY asset targets do not directly depend on target/release/borgreport.
# This avoids a rebuild when `make install` is run after a `make`.
# The asset targets are supposed to be pre-generated and part of a release tarball.
#

# Collect all third-party licenses
LICENSE-THIRD-PARTY.md: about.hbs about.toml Cargo.lock
	$(CARGO-ABOUT) generate --fail --threshold 1.0 --output-file $@ $<

# Checks on source files and dependencies
.PHONY: lint
lint:
	$(CARGO) check --locked --target $(native_target_triple)
	$(CARGO-CLIPPY) --locked --no-deps --target $(native_target_triple)
	$(CARGO-DENY) --locked check --hide-inclusion-graph
	$(CARGO-MSRV) verify --target $(native_target_triple)
	$(REUSE) lint -l

# Run the test suites
.PHONY: test test-all
test: $(locked_src)
	$(CARGO) test --locked
test-all: $(locked_src)
	$(CARGO) test --locked -- --include-ignored

# Update generated assets like man pages and shell_completions from last release build
.PHONY: assets
assets/man/borgreport.1: assets/man/borgreport.1.adoc Cargo.lock
	$(ASCIIDOCTOR) --safe-mode secure --backend manpage --attribute release-version=$(version) --out-file $@ $<
assets/shell_completions/%: Cargo.lock src/cli.rs
	@mkdir -p $(dir $@)
	@cp -v -t $(dir $@) target/release/assets/shell_completions/$*
assets: target/release/borgreport $(generated_assets) $(static_assets);

# Generate compressed static release binaries
.PHONY: static
target/%/static/borgreport: $(locked_src)
	RUSTFLAGS='-C target-feature=+crt-static' $(CARGO) build --locked --profile static --target $*
	$(UPX) --no-backup --lzma --best --preserve-build-id $@
static: target/x86_64-unknown-linux-gnu/static/borgreport \
		target/aarch64-unknown-linux-gnu/static/borgreport;

# Generate static Debian packages
.PHONY: debian
target/%/debian/borgreport_$(version)-1_amd64.deb: $(locked_src) $(generated_assets) ${static_assets}
	RUSTFLAGS='-C target-feature=+crt-static' $(CARGO-DEB) --locked --profile debian-build --target $*
target/%/debian/borgreport_$(version)-1_arm64.deb: $(locked_src) $(generated_assets) ${static_assets}
	RUSTFLAGS='-C target-feature=+crt-static' $(CARGO-DEB) --locked --profile debian-build --target $*
debian:	target/x86_64-unknown-linux-gnu/debian/borgreport_$(version)-1_amd64.deb \
		target/aarch64-unknown-linux-gnu/debian/borgreport_$(version)-1_arm64.deb;

# Generate static RPM packages
.PHONY: rpm
define rpm_template =
target/$(1)-unknown-linux-gnu/generate-rpm/borgreport-$$(version)-1.$(1).rpm: $$(locked_src) $$(generated_assets) $${static_assets}
	RUSTFLAGS='-C target-feature=+crt-static' $(CARGO) build --locked --profile rpm-build --target $(1)-unknown-linux-gnu
	cargo generate-rpm --profile rpm-build --target $(1)-unknown-linux-gnu --payload-compress gzip
endef
$(foreach arch,x86_64 aarch64,$(eval $(call rpm_template,$(arch))))
rpm: target/x86_64-unknown-linux-gnu/generate-rpm/borgreport-$(version)-1.x86_64.rpm \
	 target/aarch64-unknown-linux-gnu/generate-rpm/borgreport-$(version)-1.aarch64.rpm;

# Generate a source tarball
.PHONY: crate
target/package/borgreport-$(version).crate: $(locked_src) $(generated_assets) ${static_assets}
	$(CARGO) package --no-verify
crate: target/package/borgreport-$(version).crate;

# Generate binary tarballs for static binaries
# https://www.gnu.org/software/tar/manual/html_node/Reproducibility.html
tar_create := $(TAR) --create --auto-compress --sort=name --format=posix --pax-option='exthdr.name=%d/PaxHeaders/%f' --pax-option='delete=atime,delete=ctime' --mtime='1970-01-01T00:00:00Z' --numeric-owner --owner=0 --group=0 --mode='go+u,go-w'
tar_create_bin = $(tar_create) --file=$(abspath $@) --transform 's|^|borgreport-$(version)/|' --transform 's|/assets/|/|' $(generated_assets) ${static_assets} --directory=$(<D) $(<F)

# Collect all release artifacts in target/dist
.PHONY: dist
dist_artifacts := borgreport-$(version).tar.gz borgreport-$(version)-linux-x86_64.tar.gz borgreport-$(version)-linux-aarch64.tar.gz borgreport_$(version)-1_amd64.deb borgreport_$(version)-1_arm64.deb borgreport-$(version)-1.x86_64.rpm borgreport-$(version)-1.aarch64.rpm
target/dist/v$(version):
	@mkdir -p $@
# A crate file is a tar.gz: https://github.com/rust-lang/cargo/blob/master/src/cargo/ops/cargo_package.rs
target/dist/v$(version)/borgreport-$(version).tar.gz: target/package/borgreport-$(version).crate |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport-$(version)-linux-x86_64.tar.gz:  target/x86_64-unknown-linux-gnu/static/borgreport  $(generated_assets) ${static_assets} |target/dist/v$(version)
	$(tar_create_bin)
target/dist/v$(version)/borgreport-$(version)-linux-aarch64.tar.gz: target/aarch64-unknown-linux-gnu/static/borgreport $(generated_assets) ${static_assets} |target/dist/v$(version)
	$(tar_create_bin)
target/dist/v$(version)/borgreport_$(version)-1_amd64.deb: target/x86_64-unknown-linux-gnu/debian/borgreport_$(version)-1_amd64.deb  |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport_$(version)-1_arm64.deb: target/aarch64-unknown-linux-gnu/debian/borgreport_$(version)-1_arm64.deb |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport-$(version)-1.x86_64.rpm:  target/x86_64-unknown-linux-gnu/generate-rpm/borgreport-$(version)-1.x86_64.rpm   |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport-$(version)-1.aarch64.rpm: target/aarch64-unknown-linux-gnu/generate-rpm/borgreport-$(version)-1.aarch64.rpm |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/SHA256SUMS: $(addprefix target/dist/v$(version)/, $(dist_artifacts))
	@env -C $(dir $@) -S sha256sum --binary $(notdir $^) > $@
dist: $(addprefix target/dist/v$(version)/, $(dist_artifacts) SHA256SUMS);

# Minisign all dist artifacts
# https://github.com/cargo-bins/cargo-binstall/blob/main/SIGNING.md
.PHONY: minisign
dist_artifacts_minisig :=  $(addsuffix .minisig,$(dist_artifacts))
$(addprefix target/dist/v$(version)/, $(dist_artifacts_minisig)): $(addprefix target/dist/v$(version)/, $(dist_artifacts)) |minisign/borgreport.key target/dist/v$(version)
	$(MINISIGN) -S -s minisign/borgreport.key -x $@ -m $^
minisign: $(addprefix target/dist/v$(version)/, $(dist_artifacts_minisig));
