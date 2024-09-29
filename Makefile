#!/usr/bin/env -S make -f
# SPDX-FileCopyrightText: 2024 Philipp Micheel <bbx0+borgreport@bitdevs.de>
# SPDX-License-Identifier: GPL-3.0-or-later

# Makefile to manage release artifacts for borgreport
#
# Usage:
# 	Build and Install:
# 		make prepare
# 		make build
# 		make PREFIX=/usr install
#
#   Make all release packages in target/dist/$(version):
# 		make assets
# 		make dist
#
#   Make source tarball:
# 		make crate
#
#   Make static binaries:
# 		make static
#
#   Make static debian packages:
# 		make assets
# 		make debian
#
##

.SUFFIXES:
SHELL		= /bin/sh
HELP2MAN	:= help2man
PREFIX		:= /usr/local

# Common prefix for installation directories.
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
sd_tmpfiles_dir 		:= $(libdir)/tmpfiles.d

# Generated and static assets
shell_completions := _borgreport borgreport.bash borgreport.elv borgreport.fish
generated_assets := $(addprefix assets/man/,borgreport.1 borgreport.html) $(addprefix assets/shell_completions/,$(shell_completions))
static_assets := $(addprefix assets/systemd/,borgreport.service borgreport.timer borgreport.tmpfile)

# cargo get package.version
version ?= $(shell grep --perl-regexp --only-matching --max-count 1 -e '(?<=(^version = "))(.*)(?=("$$))' Cargo.toml)
locked_src := Cargo.toml Cargo.lock build.rs src/**
native_target_triple := $(shell rustc -vV | sed -n 's/host: //p')

# Generate a native release build
.PHONY: build
build: prepare target/release/borgreport ;
target/release/borgreport: $(locked_src)
	cargo build --frozen --release

# Pull the native sources only
.PHONY: prepare
prepare:
	@cargo fetch --locked --target $(native_target_triple)

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
	install -Dm644 assets/systemd/borgreport.tmpfile ${DESTDIR}${sd_tmpfiles_dir}/borgreport.conf

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
	-rm ${DESTDIR}${sd_tmpfiles_dir}/borgreport.conf

.PHONY: clean
clean:
	cargo clean

#### Development and Release Helper ####
# Note for assets:
# The non-PHONY asset targets do not directly depend on target/release/borgreport.
# This avoids a rebuild when `make install` is run after a `make`.
# The asset targets are supposed to be pre-generated and part of a release tarball.
#

# Collect all third-party licenses
LICENSE-THIRD-PARTY.md: about.hbs about.toml Cargo.lock
	cargo about generate --fail --threshold 1.0 --output-file $@ $<
#   cargo bundle-licenses --format toml --output $@ --previous $@ --check-previous

# Checks on source files and dependencies
.PHONY: lint
lint:
	cargo clippy --frozen --no-deps --target $(native_target_triple)
	reuse lint -l
	cargo deny check --hide-inclusion-graph

# Update generated assets like manpages and shell_completions from last release build
.PHONY: assets
## Generate a manpage from the `--help-man` command of the release build
assets/man/borgreport.1: Cargo.lock src/cli.rs
	@mkdir -p $(dir $@)
	$(HELP2MAN) --no-info --help-option='--help-man' --output=$@ target/release/borgreport
assets/man/borgreport.html: assets/man/borgreport.1
	SOURCE_DATE_EPOCH=0 groff -mandoc -Thtml $< > $@
assets/shell_completions/%: Cargo.lock src/cli.rs
	@mkdir -p $(dir $@)
	@cp -v -t $(dir $@) target/release/assets/shell_completions/$*
assets: target/release/borgreport $(generated_assets);

# Generate compressed static release binaries
.PHONY: static
target/%/static/borgreport: $(locked_src)
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --locked --profile static --target $*
	upx --no-backup --lzma --best --preserve-build-id $@
static: target/x86_64-unknown-linux-gnu/static/borgreport \
		target/aarch64-unknown-linux-gnu/static/borgreport;

# Generate static Debian packages
.PHONY: debian
target/%/debian/borgreport_$(version)-1_amd64.deb: $(locked_src) $(generated_assets) ${static_assets} LICENSE-THIRD-PARTY.md
	RUSTFLAGS='-C target-feature=+crt-static' cargo deb --locked --profile debian-build --target $*
target/%/debian/borgreport_$(version)-1_arm64.deb: $(locked_src) $(generated_assets) ${static_assets} LICENSE-THIRD-PARTY.md
	RUSTFLAGS='-C target-feature=+crt-static' cargo deb --locked --profile debian-build --target $*
debian:	target/x86_64-unknown-linux-gnu/debian/borgreport_$(version)-1_amd64.deb \
		target/aarch64-unknown-linux-gnu/debian/borgreport_$(version)-1_arm64.deb;

# Generate a source tarball
.PHONY: crate
target/package/borgreport-$(version).crate: $(locked_src) $(generated_assets) ${static_assets} LICENSE-THIRD-PARTY.md
	cargo package --no-verify --allow-dirty
crate: target/package/borgreport-$(version).crate;

# Collect all release artifacts in target/dist
.PHONY: dist
dist_artifacts := borgreport-$(version).tar.gz borgreport-x86_64-linux borgreport-aarch64-linux borgreport_$(version)-1_amd64.deb borgreport_$(version)-1_arm64.deb
target/dist/v$(version):
	@mkdir -p $@
# A crate file is a tar.gz: https://github.com/rust-lang/cargo/blob/master/src/cargo/ops/cargo_package.rs
target/dist/v$(version)/borgreport-$(version).tar.gz:		target/package/borgreport-$(version).crate |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport-x86_64-linux:			target/x86_64-unknown-linux-gnu/static/borgreport |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport-aarch64-linux:			target/aarch64-unknown-linux-gnu/static/borgreport |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport_$(version)-1_amd64.deb:	target/x86_64-unknown-linux-gnu/debian/borgreport_$(version)-1_amd64.deb |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/borgreport_$(version)-1_arm64.deb:	target/aarch64-unknown-linux-gnu/debian/borgreport_$(version)-1_arm64.deb |target/dist/v$(version)
	@cp -v $< $@
target/dist/v$(version)/SHA256SUMS: $(addprefix target/dist/v$(version)/, $(dist_artifacts))
	@env -C $(dir $@) -S sha256sum --binary $(notdir $^) > $@
dist: $(addprefix target/dist/v$(version)/, $(dist_artifacts) SHA256SUMS)
