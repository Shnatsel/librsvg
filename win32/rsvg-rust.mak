!ifndef VCVER
!include detectenv-msvc.mak
!ifndef LIBDIR
LIBDIR=$(PREFIX)\lib
!endif
!endif

!if "$(CARGO)" == ""
CARGO = cargo
!endif

!if "$(RUSTUP)" == ""
RUSTUP = rustup
!endif

!if "$(PLAT)" == "x64"
RUST_TARGET = x86_64
!else
RUST_TARGET = i686
!endif

!if "$(VALID_CFGSET)" == "TRUE"
BUILD_RUST = 1
!else
BUILD_RUST = 0
!endif

!if "$(BUILD_RUST)" == "1"

CARGO_TARGET = --target $(RUST_TARGET)-pc-windows-msvc
DEFAULT_TARGET = stable-$(RUST_TARGET)-pc-windows-msvc
RUSTUP_CMD = $(RUSTUP) default $(DEFAULT_TARGET)

!if "$(CFG)" == "release" || "$(CFG)" == "Release"
CARGO_CMD = $(CARGO) build $(CARGO_TARGET) --release
!else
CARGO_CMD = $(CARGO) build $(CARGO_TARGET)
!endif

vs$(VSVER)\$(CFG)\$(PLAT)\obj\rsvg_c_api\$(RUST_TARGET)-pc-windows-msvc\$(CFG)\rsvg_c_api.lib:
	@set PATH=%PATH%;%HOMEPATH%\.cargo\bin
	@set CARGO_TARGET_DIR=win32\vs$(VSVER)\$(CFG)\$(PLAT)\obj\rsvg_c_api
	@set GTK_LIB_DIR=$(LIBDIR);$(LIB)
	$(RUSTUP_CMD)
	@cd ..
	$(CARGO_CMD) --verbose
	@cd win32
	@set GTK_LIB_DIR=
	@set CARGO_TARGET_DIR=

cargo-clean:
	@set PATH=%PATH%;%HOMEPATH%\.cargo\bin
	@set CARGO_TARGET_DIR=win32\vs$(VSVER)\$(CFG)\$(PLAT)\obj\rsvg_c_api
	@cd ..
	@$(CARGO) clean
	@cd win32
	@set CARGO_TARGET_DIR=
	
!else
!if "$(VALID_CFGSET)" == "FALSE"
!error You need to specify an appropriate config for your build, using CFG=Release|Debug
!endif
!endif
