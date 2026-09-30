# Build and install Torvo. `make && sudo make install` puts it in /usr/local;
# the PKGBUILD uses `make install PREFIX=/usr DESTDIR="$pkgdir"`.

PREFIX  ?= /usr/local
DESTDIR ?=
BINDIR   = $(DESTDIR)$(PREFIX)/bin
DATADIR  = $(DESTDIR)$(PREFIX)/share
CARGO   ?= cargo

.PHONY: all build test install uninstall clean

all: build

build:
	$(CARGO) build --release --locked

test:
	tests/e2e/run.sh

install:
	install -Dm755 target/release/torvo "$(BINDIR)/torvo"
	install -Dm644 data/org.torvo.Torvo.desktop "$(DATADIR)/applications/org.torvo.Torvo.desktop"
	install -Dm644 data/org.torvo.Torvo.metainfo.xml "$(DATADIR)/metainfo/org.torvo.Torvo.metainfo.xml"
	install -Dm644 data/icons/org.torvo.Torvo.svg "$(DATADIR)/icons/hicolor/scalable/apps/org.torvo.Torvo.svg"
	install -Dm644 LICENSE "$(DATADIR)/licenses/torvo/LICENSE"

uninstall:
	rm -f "$(BINDIR)/torvo" \
	      "$(DATADIR)/applications/org.torvo.Torvo.desktop" \
	      "$(DATADIR)/metainfo/org.torvo.Torvo.metainfo.xml" \
	      "$(DATADIR)/icons/hicolor/scalable/apps/org.torvo.Torvo.svg"
	rm -rf "$(DATADIR)/licenses/torvo"

clean:
	$(CARGO) clean
