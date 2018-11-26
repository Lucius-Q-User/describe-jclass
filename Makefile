all: ClassQuickLook.qlgenerator

install: all
	cp -r ClassQuickLook.qlgenerator ~/Library/QuickLook
	qlmanage -r

target/release/libdescribe_jclass.dylib: Cargo.toml src/lib.rs src/describe.rs
	cargo build --release

ClassQuickLook.qlgenerator: Info.plist target/release/libdescribe_jclass.dylib
	mkdir -p ClassQuickLook.qlgenerator/Contents/MacOS
	cp target/release/libdescribe_jclass.dylib ClassQuickLook.qlgenerator/Contents/MacOS/ClassQuickLook
	cp Info.plist ClassQuickLook.qlgenerator/Contents/

clean:
	-rm -rf ClassQuickLook.qlgenerator
	cargo clean

.PHONY: clean all
